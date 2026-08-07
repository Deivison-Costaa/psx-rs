use std::cell::{Cell, RefCell};
use std::collections::VecDeque;

use crate::cdrom_bin_cue::DiscLayout;
use crate::cdrom_xa::{self, XaState};

const PAUSE_READING_CYCLES: u64 = 0x021_181C;
const PAUSE_IDLE_CYCLES: u64 = 0x1DF2;
const STOP_MOTOR_CYCLES: u64 = 0x0D3_8ACA;
const STOP_STOPPED_CYCLES: u64 = 0x1D7B;
const GETID_CYCLES: u64 = 0x4A00;

// APROXIMACAO DECLARADA. A spec mede GetID/Pause/Stop (06-cdrom.md L2069-2076) mas diz do
// seek: "The seek timings are still unknown, and they are probably quite complicated"
// (L2079). O que ela afirma e' que o tempo depende da distancia (L2077-2078, L2081-2086) e
// que toda medida do drive tem FAIXA, nao valor unico. Modelo adotado: custo fixo de
// assentamento + termo linear na distancia em quadros, calibrado para ~13,5 ms no seek
// curto e ~200 ms na varredura quase completa do disco. Nao ha medida de hardware por tras
// destes dois numeros.
const SEEK_SETTLE_CYCLES: u64 = 0x0007_0000;
const SEEK_CYCLES_PER_FRAME: u64 = 24;
const SPINUP_CYCLES: u64 = STOP_MOTOR_CYCLES;
const SEEK_JITTER_DIVISOR: u64 = 64;
const SEEK_RNG_SEED: u64 = 0x0123_4567_89AB_CDEF;

// § Sector Buffer (06-cdrom.md L2109-2111): "The buffer is apparently divided into 8 slots".
const SLOTS: usize = 8;
const SLOT_BYTES: usize = 2340;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Cdrom {
    bank: Cell<u8>,
    param_count: Cell<u8>,
    param_buf: Cell<[u8; 16]>,
    result_count: Cell<u8>,
    result_head: Cell<u8>,
    result_buf: Cell<[u8; 16]>,
    intsts: Cell<u8>,
    intmsk: Cell<u8>,
    busy: Cell<bool>,
    pending_second: Cell<u8>,
    second_request: Cell<bool>,
    disc_inserted: Cell<bool>,
    motor_on: Cell<bool>,
    seeking: Cell<bool>,
    reading: Cell<bool>,
    shell_open: Cell<bool>,
    seek_min: Cell<u8>,
    seek_sec: Cell<u8>,
    seek_sect: Cell<u8>,
    #[serde(with = "crate::serde_grande::em_cell")]
    data_buffer: Cell<[u8; 2340]>,
    data_pos: Cell<usize>,
    data_len: Cell<usize>,
    read_mode: Cell<u8>,
    hchpctl: Cell<u8>,
    irq_line: Cell<bool>,
    pending_cmd: Cell<Option<u8>>,
    pending_params: Cell<[u8; 16]>,
    pending_param_count: Cell<u8>,
    issued_cmd: Cell<Option<u8>>,
    int2_pending: Cell<bool>,
    int1_pending: Cell<bool>,
    second_cycles: Cell<u64>,
    read_pos_mm: Cell<u8>,
    read_pos_ss: Cell<u8>,
    read_pos_ff: Cell<u8>,
    mode: Cell<u8>,
    second_dirty: Cell<bool>,
    playing: Cell<bool>,
    play_track: Cell<u8>,
    audio_fifo: RefCell<VecDeque<(i16, i16)>>,
    xa_state: Cell<XaState>,
    filter_file: Cell<u8>,
    filter_channel: Cell<u8>,
    // § GetlocL (06-cdrom.md L1057-1059): "the GetlocL command returns the header and
    // subheader of the <newest> buffered sector" — o mais NOVO completo, que diverge do
    // setor que o INT1 esta entregando.
    last_data_sector: Cell<Option<(u8, u8, u8)>>,
    #[serde(with = "crate::serde_grande::em_cell")]
    sector_slots: Cell<[u8; SLOTS * SLOT_BYTES]>,
    write_slot: Cell<u8>,
    newest_slot: Cell<u8>,
    int1_slot: Cell<u8>,
    sector_ready: Cell<bool>,
    seek_rng: Cell<u64>,
}

/// Quatro setores de CD-DA. Se o jogo le mais rapido do que o SPU consome, o excedente
/// e descartado em vez de virar vazamento.
const AUDIO_FIFO_MAX: usize = 4 * cdrom_xa::CDDA_FRAMES;

impl Cdrom {
    pub fn new() -> Self {
        Cdrom {
            bank: Cell::new(0),
            param_count: Cell::new(0),
            param_buf: Cell::new([0u8; 16]),
            result_count: Cell::new(0),
            result_head: Cell::new(0),
            result_buf: Cell::new([0u8; 16]),
            intsts: Cell::new(0),
            intmsk: Cell::new(0),
            busy: Cell::new(false),
            pending_second: Cell::new(0),
            second_request: Cell::new(false),
            disc_inserted: Cell::new(false),
            motor_on: Cell::new(false),
            seeking: Cell::new(false),
            reading: Cell::new(false),
            shell_open: Cell::new(false),
            seek_min: Cell::new(0),
            seek_sec: Cell::new(0),
            seek_sect: Cell::new(0),
            data_buffer: Cell::new([0u8; 2340]),
            data_pos: Cell::new(0),
            data_len: Cell::new(2048),
            read_mode: Cell::new(0),
            hchpctl: Cell::new(0),
            irq_line: Cell::new(false),
            pending_cmd: Cell::new(None),
            pending_params: Cell::new([0u8; 16]),
            pending_param_count: Cell::new(0),
            issued_cmd: Cell::new(None),
            int2_pending: Cell::new(false),
            int1_pending: Cell::new(false),
            second_cycles: Cell::new(0x4A00),
            read_pos_mm: Cell::new(0),
            read_pos_ss: Cell::new(0),
            read_pos_ff: Cell::new(0),
            mode: Cell::new(0),
            second_dirty: Cell::new(false),
            playing: Cell::new(false),
            play_track: Cell::new(0),
            audio_fifo: RefCell::new(VecDeque::new()),
            xa_state: Cell::new(XaState::default()),
            filter_file: Cell::new(0),
            filter_channel: Cell::new(0),
            last_data_sector: Cell::new(None),
            sector_slots: Cell::new([0u8; SLOTS * SLOT_BYTES]),
            write_slot: Cell::new(0),
            newest_slot: Cell::new(0),
            int1_slot: Cell::new(0),
            sector_ready: Cell::new(false),
            seek_rng: Cell::new(SEEK_RNG_SEED),
        }
    }

    fn slot_escreve(&self, slot: u8, dados: &[u8; SLOT_BYTES]) {
        let mut todos = self.sector_slots.get();
        let inicio = (slot as usize % SLOTS) * SLOT_BYTES;
        todos[inicio..inicio + SLOT_BYTES].copy_from_slice(dados);
        self.sector_slots.set(todos);
    }

    fn slot_le(&self, slot: u8) -> [u8; SLOT_BYTES] {
        let todos = self.sector_slots.get();
        let inicio = (slot as usize % SLOTS) * SLOT_BYTES;
        let mut saida = [0u8; SLOT_BYTES];
        saida.copy_from_slice(&todos[inicio..inicio + SLOT_BYTES]);
        saida
    }

    // § Setmode (06-cdrom.md L685-703): bit5 troca entre 800h=DataOnly (2048) e
    // 924h=WholeSectorExceptSyncBytes (2340).
    fn sector_size(&self) -> usize {
        if self.mode.get() & 0x20 != 0 {
            SLOT_BYTES
        } else {
            2048
        }
    }

    /// Grava no slot corrente o setor que o drive esta recebendo agora. O slot recebe o
    /// conteudo assim que o setor comeca a chegar — e' o que faz o slot 1 mostrar o setor
    /// 17 enquanto o mais novo COMPLETO ainda e' o 16 (06-cdrom.md L2158-2168).
    fn grava_setor_em_voo(&self, disc_layout: Option<&DiscLayout>, disc_bin: Option<&[u8]>) {
        let tam = self.sector_size();
        let lido = match (disc_layout, disc_bin) {
            (Some(layout), Some(bin)) => read_sector_from_disc(
                layout,
                bin,
                self.read_pos_mm.get(),
                self.read_pos_ss.get(),
                self.read_pos_ff.get(),
                tam,
            ),
            _ => None,
        };
        let buf = lido.unwrap_or_else(|| {
            let mut stub = [0u8; SLOT_BYTES];
            for (i, b) in stub.iter_mut().enumerate().take(tam) {
                *b = (i as u8).wrapping_add(1);
            }
            stub
        });
        self.slot_escreve(self.write_slot.get(), &buf);
    }

    fn inicia_ring(&self, disc_layout: Option<&DiscLayout>, disc_bin: Option<&[u8]>) {
        self.write_slot.set(0);
        self.newest_slot.set(0);
        self.int1_slot.set(0);
        self.sector_ready.set(false);
        self.last_data_sector.set(None);
        self.grava_setor_em_voo(disc_layout, disc_bin);
    }

    /// § Buffer Overrun Timings (06-cdrom.md L758-782): o Data Request TRAVA o setor
    /// pedido ("the requested data is locked"), entao o conteudo do slot e' copiado no
    /// instante do pedido — nao no instante do INT1.
    fn carrega_do_slot(&self) {
        self.data_buffer.set(self.slot_le(self.int1_slot.get()));
        self.data_len.set(self.sector_size());
        self.data_pos.set(0);
    }

    fn levanta_int1(&self) {
        self.busy.set(false);
        self.result_clear();
        self.result_push(self.stat_byte());
        self.intsts.set(1);
        self.int1_slot.set(self.newest_slot.get());
        self.sector_ready.set(false);
        self.hchpctl.set(0);
        self.carrega_do_slot();
    }

    pub fn insert_disc(&self) {
        self.disc_inserted.set(true);
        self.motor_on.set(true);
    }

    fn stat_byte(&self) -> u8 {
        let mut s = 0u8;
        if self.playing.get() {
            s |= 1 << 7;
        }
        if self.seeking.get() {
            s |= 1 << 6;
        }
        if self.reading.get() {
            s |= 1 << 5;
        }
        if self.shell_open.get() {
            s |= 1 << 4;
        }
        if self.motor_on.get() {
            s |= 1 << 1;
        }
        s
    }

    fn param_push(&self, val: u8) {
        let count = self.param_count.get() as usize;
        if count < 16 {
            let mut buf = self.param_buf.get();
            buf[count] = val;
            self.param_buf.set(buf);
            self.param_count.set((count + 1) as u8);
        }
    }

    fn param_pop(&self) -> u8 {
        let count = self.param_count.get() as usize;
        if count == 0 {
            return 0;
        }
        let buf = self.param_buf.get();
        let val = buf[0];
        let mut new_buf = [0u8; 16];
        new_buf[..count - 1].copy_from_slice(&buf[1..count]);
        self.param_buf.set(new_buf);
        self.param_count.set((count - 1) as u8);
        val
    }

    fn param_is_empty(&self) -> bool {
        self.param_count.get() == 0
    }

    fn param_len(&self) -> u8 {
        self.param_count.get()
    }

    fn param_clear(&self) {
        self.param_count.set(0);
    }

    fn result_push(&self, val: u8) {
        let count = self.result_count.get() as usize;
        if count < 16 {
            let mut buf = self.result_buf.get();
            buf[count] = val;
            self.result_buf.set(buf);
            self.result_count.set((count + 1) as u8);
        }
    }

    fn result_pop(&self) -> u8 {
        let count = self.result_count.get() as usize;
        if count == 0 {
            return 0;
        }
        let head = self.result_head.get() as usize;
        let buf = self.result_buf.get();
        let val = buf[head];
        self.result_head.set(((head + 1) & 0xF) as u8);
        self.result_count.set((count - 1) as u8);
        val
    }

    fn result_is_empty(&self) -> bool {
        self.result_count.get() == 0
    }

    fn result_clear(&self) {
        self.result_count.set(0);
        self.result_head.set(0);
    }

    fn hsts(&self) -> u8 {
        let mut s = self.bank.get() & 0x3;
        if self.param_is_empty() {
            s |= 1 << 3;
        }
        if self.param_len() < 16 {
            s |= 1 << 4;
        }
        if !self.result_is_empty() {
            s |= 1 << 5;
        }
        if self.data_pos.get() < self.data_len.get()
            && self.read_mode.get() != 0
            && (self.hchpctl.get() & 0x80) != 0
        {
            s |= 1 << 6;
        }
        if self.busy.get() {
            s |= 1 << 7;
        }
        s
    }

    fn set_bank(&self, val: u8) {
        self.bank.set(val & 0x3);
    }

    // § Sending a new command while another is pending (06-cdrom.md L471-473): a spec
    // mede "ReadN/ReadS -> Wait for INT3 IRQ -> clear IRQ -> SetMode/SetLoc/..." e
    // conclui "Will not drop any of the two commands, thus execute sequentially". Ou
    // seja: a regra e' pela lista de quem ABORTA a leitura, nao por uma lista curta de
    // "passivos". Sao os que mexem em motor/posicao ou reiniciam a transferencia —
    // Play, ReadN, MotorOn, Stop, Pause, Init, SetSession, SeekL, SeekP, GetID, ReadS,
    // Reset, ReadTOC (§ Command Summary L546-601, coluna de completion). Todo o resto
    // (Setmode, Setloc, Setfilter, Mute/Demute, os Getloc/GetT*, Test, GetQ) responde
    // so' INT3 e deixa o drive girando.
    fn aborta_leitura(cmd: u8) -> bool {
        matches!(
            cmd,
            0x03 | 0x06
                | 0x07
                | 0x08
                | 0x09
                | 0x0A
                | 0x12
                | 0x15
                | 0x16
                | 0x1A
                | 0x1B
                | 0x1C
                | 0x1E
        )
    }

    // § "cancela resposta armada ao aceitar comando" (commit 7b57967) descarta um 2o
    // response OBSOLETO de um comando anterior ja concluido. Mas um comando passivo
    // (Nop, GetlocL, ...) ACEITO ou DESPACHADO enquanto ReadN/Play continua entregando
    // setores (reading/playing) nao pode contar como "a CPU aceitou um comando novo,
    // descarte a 2a resposta obsoleta" — senao a entrega ainda a caminho e cancelada e
    // a leitura nunca completa (GT2 fazia ReadN;ack;Nop e ficava preso num retry
    // infinito de Setloc/Setmode/ReadN/GetlocL sem nunca ver o setor chegar). Usado
    // tanto no latch (write8) quanto no dispatch de fato (deliver_first) — sao dois
    // pontos independentes que hoje descartam CDROM_SECOND.
    fn preserva_entrega_em_voo(&self, cmd: u8) -> bool {
        (self.reading.get() || self.playing.get()) && !Self::aborta_leitura(cmd)
    }

    fn latch_command(&self, cmd: u8) {
        self.pending_cmd.set(Some(cmd));
        self.pending_params.set(self.param_buf.get());
        self.pending_param_count.set(self.param_count.get());
        self.issued_cmd.set(Some(cmd));
        if self.intsts.get() == 0
            && !self.int2_pending.get()
            && !self.int1_pending.get()
            && !self.preserva_entrega_em_voo(cmd)
        {
            self.second_dirty.set(true);
        }
    }

    /// Um quadro de 44,1 kHz para o SPU. § SPU-ADPCM vs XA-ADPCM (L260) de
    /// docs/reference/08-spu.md: o XA nao ocupa voz nem RAM do SPU.
    pub fn take_audio_frame(&self) -> Option<(i16, i16)> {
        self.audio_fifo.borrow_mut().pop_front()
    }

    pub fn audio_pending(&self) -> usize {
        self.audio_fifo.borrow().len()
    }

    fn enfileira_audio(&self, quadros: Vec<(i16, i16)>) {
        let mut fifo = self.audio_fifo.borrow_mut();
        for q in quadros {
            if fifo.len() >= AUDIO_FIFO_MAX {
                break;
            }
            fifo.push_back(q);
        }
    }

    /// Setor cru do disco na posicao corrente de leitura.
    fn setor_cru(&self, bin: Option<&[u8]>) -> Option<Vec<u8>> {
        self.setor_cru_em(
            bin,
            self.read_pos_mm.get(),
            self.read_pos_ss.get(),
            self.read_pos_ff.get(),
        )
    }

    /// Setor cru do disco numa posicao MSF (BCD) arbitraria — usado por GetlocL (06-cdrom.md
    /// L1052-1071) pra reler o cabecalho/subcabecalho do ultimo setor de dado entregue, que
    /// nao e mais a posicao corrente (ja avancada por advance_read_pos).
    fn setor_cru_em(&self, bin: Option<&[u8]>, mm: u8, ss: u8, ff: u8) -> Option<Vec<u8>> {
        let bin = bin?;
        let abs = bcd_to_int(mm) * 60 * 75 + bcd_to_int(ss) * 75 + bcd_to_int(ff);
        let inicio = abs.checked_sub(150)? as usize * cdrom_xa::RAW_SECTOR_BYTES;
        let fim = inicio + cdrom_xa::RAW_SECTOR_BYTES;
        (fim <= bin.len()).then(|| bin[inicio..fim].to_vec())
    }

    fn decodifica_cru(&self, cru: &[u8]) {
        if !cdrom_xa::is_xa_audio_sector(cru) {
            return;
        }
        let coding = cru[0x13];
        let mut estado = self.xa_state.get();
        let quadros = cdrom_xa::decode_sector(cru, cdrom_xa::xa_is_stereo(coding), &mut estado);
        self.xa_state.set(estado);
        self.enfileira_audio(cdrom_xa::resample_to_44100(
            &quadros,
            cdrom_xa::xa_sample_rate(coding),
        ));
    }

    pub fn take_issued_command(&self) -> Option<u8> {
        self.issued_cmd.take()
    }

    pub fn first_response_cycles(cmd: u8) -> u64 {
        match cmd {
            0x0A | 0x1E => 0x1_3CCE,
            _ => 0xC4E1,
        }
    }

    fn head_frame(&self) -> u32 {
        msf_para_quadros((
            self.read_pos_mm.get(),
            self.read_pos_ss.get(),
            self.read_pos_ff.get(),
        ))
    }

    fn seek_target_frame(&self) -> u32 {
        msf_para_quadros((
            self.seek_min.get(),
            self.seek_sec.get(),
            self.seek_sect.get(),
        ))
    }

    // LCG de 64 bits (Knuth/MMIX). Deterministico e parte do estado serializado: dois
    // saves do mesmo instante rendem a mesma sequencia.
    fn next_random(&self) -> u64 {
        let x = self
            .seek_rng
            .get()
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.seek_rng.set(x);
        x >> 33
    }

    fn with_jitter(&self, nominal: u64) -> u64 {
        let faixa = (nominal / SEEK_JITTER_DIVISOR).max(1);
        nominal - faixa / 2 + self.next_random() % faixa
    }

    fn seek_cycles_to(&self, alvo: u32) -> u64 {
        let distancia = alvo.abs_diff(self.head_frame()) as u64;
        let spinup = if self.motor_on.get() { 0 } else { SPINUP_CYCLES };
        self.with_jitter(SEEK_SETTLE_CYCLES + distancia * SEEK_CYCLES_PER_FRAME + spinup)
    }

    fn second_response_cycles_for(&self, cmd: u8) -> u64 {
        match cmd {
            0x1A => GETID_CYCLES,
            0x03 | 0x06 | 0x15 | 0x16 | 0x1B => self.seek_cycles_to(self.seek_target_frame()),
            0x0A | 0x1E => self.seek_cycles_to(0),
            _ => GETID_CYCLES,
        }
    }

    pub fn second_response_cycles(&self) -> u64 {
        self.second_cycles.get()
    }

    pub fn deliver_first(&self, disc_layout: Option<&DiscLayout>, disc_bin: Option<&[u8]>) -> bool {
        // § First Response (06-cdrom.md L1984): o mainloop so executa o comando se NAO
        // houver INT pendente — qualquer INT sem ack, nao so int1_pending/int2_pending
        // (essas flags marcam "resposta ainda devida", nao "intsts sem ack").
        let blocked = self.intsts.get() != 0;
        if blocked {
            return false;
        }
        let cmd = match self.pending_cmd.take() {
            Some(cmd) => cmd,
            None => return false,
        };
        let preserva = self.preserva_entrega_em_voo(cmd);
        self.param_buf.set(self.pending_params.get());
        self.param_count.set(self.pending_param_count.get());
        self.send_command(cmd, disc_layout, disc_bin);
        if !preserva {
            self.second_dirty.set(true);
        }
        true
    }

    fn send_command(&self, cmd: u8, disc_layout: Option<&DiscLayout>, disc_bin: Option<&[u8]>) {
        self.busy.set(true);
        self.result_clear();
        match cmd {
            0x02 => {
                let mm = self.param_pop();
                let ss = self.param_pop();
                let ff = self.param_pop();
                self.param_clear();
                let bcd_ok = ss < 0x60 && (ss & 0x0F) < 0x0A && ff < 0x75 && (ff & 0x0F) < 0x0A;
                if !bcd_ok {
                    self.result_push(self.stat_byte() | 0x01);
                    self.result_push(0x10);
                    self.intsts.set(5);
                } else if !self.disc_inserted.get() {
                    self.result_push(self.stat_byte() | 0x01);
                    self.result_push(0x80);
                    self.intsts.set(5);
                } else {
                    self.seek_min.set(mm);
                    self.seek_sec.set(ss);
                    self.seek_sect.set(ff);
                    self.result_push(self.stat_byte());
                    self.intsts.set(3);
                }
                self.busy.set(false);
            }
            0x03 => {
                if !self.disc_inserted.get() {
                    self.result_push(self.stat_byte() | 0x01);
                    self.result_push(0x80);
                    self.intsts.set(5);
                    self.busy.set(false);
                } else {
                    // A busca conta da posicao ATUAL da cabeca — por isso o tempo sai antes
                    // de read_pos/motor_on virarem o alvo.
                    let busca = self.second_response_cycles_for(0x03);
                    self.playing.set(true);
                    self.motor_on.set(true);
                    self.play_track.set(0);
                    self.read_pos_mm.set(self.seek_min.get());
                    self.read_pos_ss.set(self.seek_sec.get());
                    self.read_pos_ff.set(self.seek_sect.get());
                    self.result_push(self.stat_byte());
                    self.intsts.set(3);
                    if self.mode.get() & 0x04 != 0 {
                        self.int1_pending.set(true);
                        self.pending_second.set(6);
                        self.second_cycles.set(busca);
                    } else {
                        self.busy.set(false);
                    }
                }
            }
            0x06 => {
                if !self.disc_inserted.get() {
                    self.result_push(self.stat_byte() | 0x01);
                    self.result_push(0x80);
                    self.intsts.set(5);
                    self.busy.set(false);
                } else {
                    let busca = self.second_response_cycles_for(0x06);
                    self.reading.set(true);
                    self.read_mode.set(1);
                    self.read_pos_mm.set(self.seek_min.get());
                    self.read_pos_ss.set(self.seek_sec.get());
                    self.read_pos_ff.set(self.seek_sect.get());
                    self.inicia_ring(disc_layout, disc_bin);
                    self.result_push(self.stat_byte());
                    self.intsts.set(3);
                    self.int1_pending.set(true);
                    self.pending_second.set(5);
                    self.second_cycles.set(busca);
                }
            }
            0x08 => {
                self.int1_pending.set(false);
                self.sector_ready.set(false);
                let stat = self.stat_byte();
                self.result_push(stat);
                self.intsts.set(3);
                self.int2_pending.set(true);
                self.pending_second.set(4);
                let timing = if self.motor_on.get() {
                    STOP_MOTOR_CYCLES
                } else {
                    STOP_STOPPED_CYCLES
                };
                self.second_cycles.set(timing);
                self.motor_on.set(false);
                self.playing.set(false);
            }
            0x09 => {
                self.int1_pending.set(false);
                self.sector_ready.set(false);
                let stat = self.stat_byte();
                self.result_push(stat);
                self.intsts.set(3);
                self.int2_pending.set(true);
                self.read_mode.set(0);
                self.playing.set(false);
                self.pending_second.set(4);
                let timing = if self.reading.get() {
                    PAUSE_READING_CYCLES
                } else {
                    PAUSE_IDLE_CYCLES
                };
                self.second_cycles.set(timing);
            }
            0x0D => {
                // § Setfilter (06-cdrom.md L676-683): 2 parametros (file, channel) que
                // passam a valer pro filtro XA-ADPCM (Setmode bit3) — precisa consumir os
                // params da fila como qualquer comando suportado, senao eles vazam pro
                // proximo comando (a rota generica de baixo nao le nem limpa param_buf;
                // Setfilter(file, channel) sobrava e virava prefixo do Setloc seguinte,
                // corrompendo mm/ss/ff dele).
                self.filter_file.set(self.param_pop());
                self.filter_channel.set(self.param_pop());
                self.param_clear();
                self.result_push(self.stat_byte());
                self.intsts.set(3);
                self.busy.set(false);
            }
            0x0A => {
                if self.int2_pending.get() && self.pending_second.get() == 1 {
                    self.busy.set(false);
                    return;
                }
                let busca = self.second_response_cycles_for(0x0A);
                if self.disc_inserted.get() {
                    self.motor_on.set(true);
                }
                self.result_push(self.stat_byte());
                self.intsts.set(3);
                self.int2_pending.set(true);
                self.pending_second.set(1);
                self.second_cycles.set(busca);
            }
            0x0E => {
                if !self.param_is_empty() {
                    self.mode.set(self.param_pop());
                }
                self.param_clear();
                self.result_push(self.stat_byte());
                self.intsts.set(3);
                self.busy.set(false);
            }
            // § GetlocL (06-cdrom.md L1052-1071): INT3(amm,ass,asect,mode,file,channel,
            // sm,ci) — cabecalho+subcabecalho do setor de DADO mais recente entregue (nao
            // funciona em Audio CD/trilha de audio, nem durante Seek — INT5(stat,80h) nos
            // dois casos, exatamente como falha quando nao ha nenhum setor de dado ainda
            // reportado).
            0x10 => {
                let cru = if self.seeking.get() {
                    None
                } else {
                    self.last_data_sector
                        .get()
                        .and_then(|(mm, ss, ff)| self.setor_cru_em(disc_bin, mm, ss, ff))
                };
                match cru {
                    Some(c) if c.len() >= 0x14 => {
                        self.result_push(c[0x0C]);
                        self.result_push(c[0x0D]);
                        self.result_push(c[0x0E]);
                        self.result_push(c[0x0F]);
                        self.result_push(c[0x10]);
                        self.result_push(c[0x11]);
                        self.result_push(c[0x12]);
                        self.result_push(c[0x13]);
                        self.intsts.set(3);
                    }
                    _ => {
                        self.result_push(self.stat_byte() | 0x01);
                        self.result_push(0x80);
                        self.intsts.set(5);
                    }
                }
                self.busy.set(false);
            }
            // § GetlocP (06-cdrom.md L1073-1088): INT3(track,index,mm,ss,sect,amm,ass,
            // asect) — posicao atual em BCD, relativa a trilha e absoluta no disco.
            // Funciona durante Seek (diferente de GetlocL).
            0x11 => {
                let amm = self.read_pos_mm.get();
                let ass = self.read_pos_ss.get();
                let asect = self.read_pos_ff.get();
                let (track, index, inicio) = self.trilha_em(disc_layout, amm, ass, asect);
                let (mm, ss, ff) = subtrai_msf((amm, ass, asect), inicio);
                self.result_push(track);
                self.result_push(index);
                self.result_push(mm);
                self.result_push(ss);
                self.result_push(ff);
                self.result_push(amm);
                self.result_push(ass);
                self.result_push(asect);
                self.intsts.set(3);
                self.busy.set(false);
            }
            0x1E => {
                self.result_push(self.stat_byte());
                self.intsts.set(3);
                self.int2_pending.set(true);
                self.pending_second.set(1);
                self.second_cycles
                    .set(self.second_response_cycles_for(0x1E));
            }
            0x15 => {
                if !self.disc_inserted.get() {
                    self.result_push(self.stat_byte() | 0x01);
                    self.result_push(0x80);
                    self.intsts.set(5);
                    self.busy.set(false);
                } else {
                    self.seeking.set(true);
                    self.sector_ready.set(false);
                    self.result_push(self.stat_byte());
                    self.intsts.set(3);
                    self.int2_pending.set(true);
                    self.pending_second.set(3);
                    self.second_cycles
                        .set(self.second_response_cycles_for(0x15));
                }
            }
            0x19 => {
                self.intsts.set(3);
                let sub = self.param_pop();
                match sub {
                    0x20 => {
                        self.result_push(0x97);
                        self.result_push(0x01);
                        self.result_push(0x10);
                        self.result_push(0xC2);
                    }
                    0x21 => {
                        self.result_push(0x01);
                    }
                    _ => {
                        self.result_push(self.stat_byte());
                    }
                }
                self.param_clear();
                self.busy.set(false);
            }
            0x1A => {
                self.result_push(self.stat_byte());
                self.intsts.set(3);
                self.int2_pending.set(true);
                self.pending_second.set(2);
                self.second_cycles
                    .set(self.second_response_cycles_for(0x1A));
            }
            0x1B => {
                if !self.disc_inserted.get() {
                    self.result_push(self.stat_byte() | 0x01);
                    self.result_push(0x80);
                    self.intsts.set(5);
                    self.busy.set(false);
                } else {
                    let busca = self.second_response_cycles_for(0x1B);
                    self.reading.set(true);
                    self.read_mode.set(2);
                    self.read_pos_mm.set(self.seek_min.get());
                    self.read_pos_ss.set(self.seek_sec.get());
                    self.read_pos_ff.set(self.seek_sect.get());
                    self.inicia_ring(disc_layout, disc_bin);
                    self.result_push(self.stat_byte());
                    self.intsts.set(3);
                    self.int1_pending.set(true);
                    self.pending_second.set(5);
                    self.second_cycles.set(busca);
                }
            }
            // § GetTN (06-cdrom.md L1090-1095): INT3(status,first,last) em BCD — o
            // despacho generico so empilhava 1 byte (status), entao o driver lia "first"
            // e "last" com o FIFO ja vazio (0 por padrao). Jogos que checam o numero de
            // trilhas (ex.: GT2 decidindo se ha trilha de audio CD-DA apos os dados)
            // travavam esperando um valor que nunca chegava direito.
            0x13 => {
                let (first, last) = match disc_layout {
                    Some(layout) if !layout.tracks.is_empty() => {
                        let first = layout.tracks.iter().map(|t| t.number).min().unwrap_or(1);
                        let last = layout.tracks.iter().map(|t| t.number).max().unwrap_or(1);
                        (first, last)
                    }
                    _ => (1, 1),
                };
                self.result_push(self.stat_byte());
                self.result_push(int_to_bcd(first as u32));
                self.result_push(int_to_bcd(last as u32));
                self.intsts.set(3);
                self.busy.set(false);
            }
            // § GetTD (06-cdrom.md L1096-1104): INT3(status,min,sec) em BCD. Parametro
            // track=00h pede o fim da ultima trilha (lead-out); 01h..NNh pede o inicio
            // da trilha N (relativo a Index=1); fora da faixa e' erro INT5(stat,10h).
            0x14 => {
                let track_bcd = self.param_pop();
                self.param_clear();
                let track = bcd_to_int(track_bcd);
                let alvo = disc_layout.and_then(|layout| {
                    if layout.tracks.is_empty() {
                        return None;
                    }
                    if track == 0 {
                        let bin = disc_bin?;
                        let total_quadros = (bin.len() / cdrom_xa::RAW_SECTOR_BYTES) as u32;
                        Some(quadros_para_msf(total_quadros + 150))
                    } else {
                        layout
                            .tracks
                            .iter()
                            .find(|t| t.number as u32 == track)
                            .map(|t| quadros_para_msf(t.start_lba + 150))
                    }
                });
                match alvo {
                    Some((mm, ss, _ff)) => {
                        self.result_push(self.stat_byte());
                        self.result_push(mm);
                        self.result_push(ss);
                        self.intsts.set(3);
                    }
                    None => {
                        self.result_push(self.stat_byte() | 0x01);
                        self.result_push(0x10);
                        self.intsts.set(5);
                    }
                }
                self.busy.set(false);
            }
            _ => {
                self.result_push(self.stat_byte());
                self.intsts.set(3);
                self.busy.set(false);
            }
        }
    }

    pub fn read8(&self, offset: u32) -> u8 {
        match offset & 0x3 {
            0 => self.hsts(),
            1 => self.result_pop(),
            2 => {
                let buf = self.data_buffer.get();
                let pos = self.data_pos.get();
                if pos < self.data_len.get() {
                    let val = buf[pos];
                    self.data_pos.set(pos + 1);
                    val
                } else {
                    0
                }
            }
            3 => {
                let base = self.intsts.get() & 0x7;
                if self.bank.get() == 1 || self.bank.get() == 3 {
                    base | 0xE0
                } else {
                    self.intmsk.get() | 0xE0
                }
            }
            _ => 0,
        }
    }

    pub fn write8(
        &self,
        offset: u32,
        val: u8,
        disc_layout: Option<&DiscLayout>,
        disc_bin: Option<&[u8]>,
    ) {
        match offset & 0x3 {
            0 => self.set_bank(val),
            1 if self.bank.get() == 0 => self.latch_command(val),
            2 if self.bank.get() == 0 => self.param_push(val),
            2 if self.bank.get() == 1 => {
                self.intmsk.set(val & 0x1F);
            }
            3 if self.bank.get() == 0 => {
                let antes = self.hchpctl.get();
                self.hchpctl.set(val);
                if val & 0x80 != 0 && antes & 0x80 == 0 {
                    self.carrega_do_slot();
                }
            }
            3 if self.bank.get() == 1 => {
                if val & 0x7 != 0 {
                    let new_intsts = self.intsts.get() & !(val & 0x07);
                    self.intsts.set(new_intsts);
                    self.irq_line.set(self.irq_pending());
                    if new_intsts == 0 {
                        if self.int2_pending.get() {
                            self.int2_pending.set(false);
                            self.second_request.set(true);
                            self.irq_line.set(false);
                        } else if self.int1_pending.get() {
                            self.int1_pending.set(false);
                            self.second_request.set(true);
                            self.irq_line.set(false);
                        } else if self.pending_cmd.get().is_some() {
                            self.deliver_first(disc_layout, disc_bin);
                        } else if self.sector_ready.get() && self.read_mode.get() != 0 {
                            // § Buffer Overrun Timings (06-cdrom.md L758-782): depois do
                            // acknowledge vem a proxima interrupcao. O setor anunciado e' o
                            // mais NOVO do buffer neste instante — o controlador "jumps
                            // directly to INT1 for the newest sector" (L2115-2117).
                            self.levanta_int1();
                        }
                    }
                }
                if val & 0x40 != 0 {
                    self.param_clear();
                }
            }
            _ => {}
        }
    }

    pub fn take_second_request(&self) -> bool {
        let v = self.second_request.get();
        self.second_request.set(false);
        v
    }

    pub fn take_second_dirty(&self) -> bool {
        let v = self.second_dirty.get();
        self.second_dirty.set(false);
        v
    }

    pub fn deliver_second_now(&self, disc_layout: Option<&DiscLayout>, disc_bin: Option<&[u8]>) {
        let pending = self.pending_second.get();
        if pending == 0 {
            return;
        }
        self.deliver_second(disc_layout, disc_bin);
        if pending == 6 && self.playing.get() && self.mode.get() & 0x04 != 0 {
            self.pending_second.set(6);
            self.int1_pending.set(true);
            self.second_cycles.set(self.sector_interval_cycles());
            return;
        }
        if pending == 5 && self.read_mode.get() != 0 {
            self.pending_second.set(5);
            self.second_cycles.set(self.sector_interval_cycles());
            // § Sector Buffer (06-cdrom.md L2118-2126): "one should process INT1's as soon
            // as possible (ie. before the cdrom controller receives and skips further
            // sectors). Otherwise sectors would be lost without notice". O drive gira
            // sozinho: o proximo setor chega no proximo intervalo tenha a CPU dado ack ou
            // nao (casos medidos em hardware, L2136-2168).
            self.second_request.set(true);
        }
    }

    // § INT1 Rate (06-cdrom.md L2093-2101): a cadencia de INT1 durante streaming
    // (CD-DA/CD-XA) precisa ser exata — SystemClock*930h/4/44100Hz em velocidade normal,
    // metade disso em dobro de velocidade (bit7 do Setmode). SystemClock/44100 e
    // CPU_CYCLES_PER_SAMPLE (spu.rs), ja usado pro tick do SPU.
    fn sector_interval_cycles(&self) -> u64 {
        const NORMAL: u64 = crate::spu::CPU_CYCLES_PER_SAMPLE * 0x930 / 4;
        if self.mode.get() & 0x80 != 0 {
            NORMAL / 2
        } else {
            NORMAL
        }
    }

    fn deliver_second(&self, disc_layout: Option<&DiscLayout>, disc_bin: Option<&[u8]>) {
        match self.pending_second.get() {
            1 => {
                self.busy.set(false);
                self.result_clear();
                self.result_push(self.stat_byte());
                self.intsts.set(2);
            }
            2 => {
                self.busy.set(false);
                self.result_clear();
                if self.disc_inserted.get() {
                    self.result_push(self.stat_byte());
                    self.result_push(0x00);
                    self.result_push(0x20);
                    self.result_push(0x00);
                    for letra in b"SCEA" {
                        self.result_push(*letra);
                    }
                    self.intsts.set(2);
                } else {
                    self.result_push(0x08);
                    self.result_push(0x40);
                    for _ in 0..6 {
                        self.result_push(0x00);
                    }
                    self.intsts.set(5);
                }
            }
            3 => {
                self.busy.set(false);
                self.seeking.set(false);
                // Terminado o Seek a cabeca esta no alvo: o proximo comando de busca conta
                // a distancia a partir daqui (06-cdrom.md L2077-2078).
                self.read_pos_mm.set(self.seek_min.get());
                self.read_pos_ss.set(self.seek_sec.get());
                self.read_pos_ff.set(self.seek_sect.get());
                self.result_clear();
                self.result_push(self.stat_byte());
                self.intsts.set(2);
            }
            4 => {
                self.busy.set(false);
                self.reading.set(false);
                self.result_clear();
                self.result_push(self.stat_byte());
                self.intsts.set(2);
            }
            5 => {
                let msf = (
                    self.read_pos_mm.get(),
                    self.read_pos_ss.get(),
                    self.read_pos_ff.get(),
                );
                let completo = self.write_slot.get();
                // O drive nunca para: o setor seguinte ja comeca a entrar no proximo slot
                // do ring (06-cdrom.md L2109-2111 + L2118-2126).
                advance_read_pos(&self.read_pos_mm, &self.read_pos_ss, &self.read_pos_ff);
                self.write_slot.set(((completo as usize + 1) % SLOTS) as u8);
                self.grava_setor_em_voo(disc_layout, disc_bin);

                // § Data/ADPCM Sector Filtering/Delivery (06-cdrom.md L760-782): um setor
                // Mode2 com submode Audio+RealTime (is_xa_audio_sector) e' entregue
                // EXCLUSIVAMENTE ao decoder XA-ADPCM quando o modo tem XA-ADPCM ligado —
                // "reject data-delivery if try_deliver_as_adpcm_sector did do
                // adpcm-delivery". Com o filtro ligado (Setmode bit3, § Setfilter
                // L676-683) so o file/channel pedidos contam; os demais audio+realtime nao
                // vao nem pro decoder nem pra CPU.
                let cru = self.setor_cru_em(disc_bin, msf.0, msf.1, msf.2);
                let audio_realtime = cru.as_deref().is_some_and(cdrom_xa::is_xa_audio_sector);
                let filtro_ligado = self.mode.get() & 0x08 != 0;
                let canal_bate = cru.as_deref().is_some_and(|c| {
                    c[0x10] == self.filter_file.get() && c[0x11] == self.filter_channel.get()
                });
                let xa_ligado = self.mode.get() & 0x40 != 0;
                let vai_pro_adpcm = xa_ligado && audio_realtime && (!filtro_ligado || canal_bate);
                let descartado_pelo_filtro = !vai_pro_adpcm && audio_realtime && filtro_ligado;
                if vai_pro_adpcm {
                    if let Some(c) = cru.as_deref() {
                        self.decodifica_cru(c);
                    }
                } else if !descartado_pelo_filtro {
                    self.last_data_sector.set(Some(msf));
                    self.newest_slot.set(completo);
                    self.sector_ready.set(true);
                    // § Sector Buffer (06-cdrom.md L2118-2126): com a INT1 anterior ainda
                    // sem ack nao ha como levantar outra — os setores pulados somem
                    // "without notice", sem flag de overrun e sem INT de erro.
                    if self.intsts.get() == 0 {
                        self.levanta_int1();
                    }
                }
            }
            6 => {
                self.busy.set(false);
                self.result_clear();
                self.intsts.set(1);
                loop {
                    advance_read_pos(&self.read_pos_mm, &self.read_pos_ss, &self.read_pos_ff);
                    if bcd_to_int(self.read_pos_ff.get()) % 10 == 0 {
                        break;
                    }
                }
                if let Some(cru) = self.setor_cru(disc_bin) {
                    self.enfileira_audio(cdrom_xa::cdda_frames(&cru));
                }
                let amm = self.read_pos_mm.get();
                let ass = self.read_pos_ss.get();
                let asect = self.read_pos_ff.get();
                let (track, index, inicio) = self.trilha_em(disc_layout, amm, ass, asect);
                if self.play_track.get() == 0 {
                    self.play_track.set(track);
                }
                if self.mode.get() & 0x02 != 0 && track != self.play_track.get() {
                    self.playing.set(false);
                    self.result_push(self.stat_byte());
                    self.intsts.set(4);
                    self.pending_second.set(0);
                    return;
                }
                let absoluto = (bcd_to_int(asect) / 10) % 2 == 0;
                self.result_push(self.stat_byte());
                self.result_push(track);
                self.result_push(index);
                if absoluto {
                    self.result_push(amm);
                    self.result_push(ass);
                    self.result_push(asect);
                } else {
                    let (mm, ss, ff) = subtrai_msf((amm, ass, asect), inicio);
                    self.result_push(mm);
                    self.result_push(ss | 0x80);
                    self.result_push(ff);
                }
                self.result_push(0x00);
                self.result_push(0x00);
            }
            _ => {}
        }
        self.pending_second.set(0);
    }

    // § Report (L1246-1256) de docs/reference/06-cdrom.md quer trilha, index e o inicio dela
    // para o tempo relativo. Sem TOC (disco stub dos testes) vale a unica coisa verdadeira de
    // qualquer disco: a trilha 1 comeca em 00:02:00, pela convencao MSF/LBA.
    fn trilha_em(&self, layout: Option<&DiscLayout>, mm: u8, ss: u8, ff: u8) -> (u8, u8, Msf) {
        let padrao = (1u8, 1u8, (0x00u8, 0x02u8, 0x00u8));
        let Some(layout) = layout else { return padrao };
        let agora_lba = msf_para_quadros((mm, ss, ff)).saturating_sub(150);
        let mut achada = padrao;
        for t in &layout.tracks {
            if t.start_lba <= agora_lba {
                achada = (t.number, 1, quadros_para_msf(t.start_lba + 150));
            }
        }
        achada
    }

    pub fn _hchpctl(&self) -> u8 {
        self.hchpctl.get()
    }

    pub fn drqsts_active(&self) -> bool {
        self.data_pos.get() < self.data_len.get()
            && self.read_mode.get() != 0
            && (self.hchpctl.get() & 0x80) != 0
    }

    pub fn irq_pending(&self) -> bool {
        (self.intsts.get() & self.intmsk.get() & 0x7) != 0
    }

    pub fn intsts(&self) -> u8 {
        self.intsts.get()
    }

    pub fn pending_cmd(&self) -> Option<u8> {
        self.pending_cmd.get()
    }

    pub fn take_irq2_edge(&self) -> bool {
        let nivel = self.irq_pending();
        let borda = nivel && !self.irq_line.get();
        self.irq_line.set(nivel);
        borda
    }
}

fn bcd_to_int(b: u8) -> u32 {
    ((b >> 4) as u32) * 10 + (b as u32 & 0xF)
}

fn int_to_bcd(n: u32) -> u8 {
    (((n / 10) << 4) | (n % 10)) as u8
}

type Msf = (u8, u8, u8);

fn msf_para_quadros((mm, ss, ff): Msf) -> u32 {
    (bcd_to_int(mm) * 60 + bcd_to_int(ss)) * 75 + bcd_to_int(ff)
}

fn quadros_para_msf(q: u32) -> Msf {
    (
        int_to_bcd(q / (60 * 75)),
        int_to_bcd((q / 75) % 60),
        int_to_bcd(q % 75),
    )
}

fn subtrai_msf(pos: Msf, inicio: Msf) -> Msf {
    let d = msf_para_quadros(pos).saturating_sub(msf_para_quadros(inicio));
    (
        int_to_bcd(d / (60 * 75)),
        int_to_bcd((d / 75) % 60),
        int_to_bcd(d % 75),
    )
}

fn advance_read_pos(mm: &Cell<u8>, ss: &Cell<u8>, ff: &Cell<u8>) {
    let f = bcd_to_int(ff.get()) + 1;
    if f >= 75 {
        ff.set(0);
        let s = bcd_to_int(ss.get()) + 1;
        if s >= 60 {
            ss.set(0);
            mm.set(int_to_bcd(bcd_to_int(mm.get()) + 1));
        } else {
            ss.set(int_to_bcd(s));
        }
    } else {
        ff.set(int_to_bcd(f));
    }
}

fn read_sector_from_disc(
    _layout: &DiscLayout,
    bin: &[u8],
    min_bcd: u8,
    sec_bcd: u8,
    sect_bcd: u8,
    sector_size: usize,
) -> Option<[u8; 2340]> {
    let abs_sector =
        bcd_to_int(min_bcd) * 60 * 75 + bcd_to_int(sec_bcd) * 75 + bcd_to_int(sect_bcd);
    let file_sector = abs_sector.checked_sub(150)?;
    let offset = file_sector as usize * 2352;
    if offset + 0x10 > bin.len() {
        return None;
    }
    let cabecalho = if bin[offset + 0x0F] == 0x02 {
        0x18
    } else {
        0x10
    };
    // § Setmode (06-cdrom.md L685-703): DataOnly comeca depois do cabecalho (Mode1=10h,
    // Mode2=18h); WholeSectorExceptSyncBytes comeca logo apos os 12 bytes de sync.
    let data_start = if sector_size == 2340 {
        offset + 0x0C
    } else {
        offset + cabecalho
    };
    let data_end = data_start + sector_size;
    if data_end > bin.len() {
        return None;
    }
    let mut buf = [0u8; 2340];
    buf[..sector_size].copy_from_slice(&bin[data_start..data_end]);
    Some(buf)
}

impl Default for Cdrom {
    fn default() -> Self {
        Self::new()
    }
}
