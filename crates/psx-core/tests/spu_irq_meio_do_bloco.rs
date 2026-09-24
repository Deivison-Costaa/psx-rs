use psx_core::spu::Spu;

const V0_PITCH: u32 = 0x1F80_1C04;
const V0_START: u32 = 0x1F80_1C06;
const KON_LO: u32 = 0x1F80_1D88;
const IRQ_ADDR: u32 = 0x1F80_1DA4;
const TRANSFER_ADDR: u32 = 0x1F80_1DA6;
const FIFO: u32 = 0x1F80_1DA8;
const CNT: u32 = 0x1F80_1DAA;
const STAT: u32 = 0x1F80_1DAE;

const CNT_ENABLE_IRQ: u16 = 0x8000 | 0x4000 | 0x0040;
const PITCH_1X: u16 = 0x1000;
const AMOSTRAS_POR_BLOCO: usize = 28;

fn carrega(spu: &mut Spu, endereco_em_oitavos: u16, bytes: &[u8]) {
    spu.write16(TRANSFER_ADDR, endereco_em_oitavos);
    for par in bytes.chunks(2) {
        spu.write16(FIFO, u16::from_le_bytes([par[0], par[1]]));
    }
    spu.write16(CNT, 1 << 4);
    spu.write16(CNT, 0);
}

fn blocos_sem_flag(quantos: usize) -> Vec<u8> {
    let mut v = vec![0u8; 16 * quantos];
    for bloco in v.chunks_mut(16) {
        bloco[0] = 0x0C;
        for b in &mut bloco[2..] {
            *b = 0x13;
        }
    }
    v
}

/// Voz 0 tocando a partir do byte 0x1000 em 1x, com a IRQ9 apontada para `irq_em_oitavos`.
fn voz_tocando_com_irq(irq_em_oitavos: u16) -> Spu {
    let mut spu = Spu::new();
    carrega(&mut spu, 0x1000 / 8, &blocos_sem_flag(8));
    spu.write16(V0_PITCH, PITCH_1X);
    spu.write16(V0_START, 0x1000 / 8);
    spu.write16(IRQ_ADDR, irq_em_oitavos);
    spu.write16(CNT, CNT_ENABLE_IRQ);
    spu.write16(KON_LO, 1);
    let _ = spu.take_irq9();
    spu
}

fn irq_ate(spu: &mut Spu, amostras: usize) -> Option<usize> {
    (0..amostras).find(|_| {
        spu.tick();
        spu.read16(STAT) & (1 << 6) != 0
    })
}

#[test]
fn irq_no_meio_do_bloco_dispara_quando_a_voz_le_esse_bloco() {
    let mut spu = voz_tocando_com_irq((0x1010 + 8) / 8);
    let quando = irq_ate(&mut spu, 3 * AMOSTRAS_POR_BLOCO);
    assert!(
        quando.is_some(),
        "IRQ em 0x1018 fica dentro do bloco 0x1010..0x101F: a voz le esses 16 bytes \
         (8 meias-palavras) ao entrar no bloco, entao a IRQ9 tem de disparar (FF9 \
         arma a IRQ assim nas FMVs e so reabastece o anel quando ela chega)"
    );
    assert!(
        quando.is_some_and(|n| n >= AMOSTRAS_POR_BLOCO - 1),
        "nao pode disparar antes da voz sair do primeiro bloco (0x1000..0x100F); disparou em {quando:?}"
    );
    assert!(
        spu.take_irq9(),
        "a flag vai para o controlador de interrupcao"
    );
}

#[test]
fn irq_alinhada_no_inicio_do_bloco_continua_disparando() {
    let mut spu = voz_tocando_com_irq(0x1020 / 8);
    assert!(
        irq_ate(&mut spu, 3 * AMOSTRAS_POR_BLOCO).is_some(),
        "a voz entra no bloco 0x1020 na 57a amostra e le o cabecalho no endereco da IRQ"
    );
}

#[test]
fn irq_num_bloco_que_a_voz_nao_le_nao_dispara() {
    let mut spu = voz_tocando_com_irq((0x1400 + 8) / 8);
    assert_eq!(
        irq_ate(&mut spu, 3 * AMOSTRAS_POR_BLOCO),
        None,
        "0x1408 esta longe dos blocos 0x1000..0x102F que a voz leu em 84 amostras"
    );
}

#[test]
fn irq_no_meio_do_bloco_do_key_on_dispara_na_hora() {
    let mut spu = Spu::new();
    carrega(&mut spu, 0x1000 / 8, &blocos_sem_flag(2));
    spu.write16(V0_PITCH, PITCH_1X);
    spu.write16(V0_START, 0x1000 / 8);
    spu.write16(IRQ_ADDR, (0x1000 + 8) / 8);
    spu.write16(CNT, CNT_ENABLE_IRQ);
    spu.write16(KON_LO, 1);
    assert!(
        spu.read16(STAT) & (1 << 6) != 0,
        "o key on le o bloco 0x1000..0x100F inteiro, que contem 0x1008"
    );
}

const V0_REPEAT: u32 = 0x1F80_1C0E;
const ENDX_LO: u32 = 0x1F80_1D9C;

fn bloco_com_flags(flags: u8) -> Vec<u8> {
    let mut b = blocos_sem_flag(1);
    b[1] = flags;
    b
}

#[test]
fn key_on_nao_sobrescreve_o_repeat_address_escrito_antes() {
    let mut spu = Spu::new();
    carrega(&mut spu, 0x200, &bloco_com_flags(0b011));
    carrega(&mut spu, 0x420, &blocos_sem_flag(2));
    spu.write16(V0_PITCH, PITCH_1X);
    spu.write16(V0_START, 0x200);
    spu.write16(V0_REPEAT, 0x420);
    spu.write16(KON_LO, 1);
    assert_eq!(
        spu.read16(V0_REPEAT),
        0x420,
        "o key on copia o start para o endereco CORRENTE; o repeat fica com o que o jogo escreveu"
    );
}

#[test]
fn anel_como_o_do_ff9_salta_para_o_repeat_e_dispara_a_irq_da_outra_metade() {
    let mut spu = Spu::new();
    let mut metade_a = blocos_sem_flag(4);
    metade_a[3 * 16 + 1] = 0b011;
    carrega(&mut spu, 0x220, &metade_a);
    carrega(&mut spu, 0x420, &blocos_sem_flag(4));
    spu.write16(V0_PITCH, PITCH_1X);
    spu.write16(V0_START, 0x220);
    spu.write16(V0_REPEAT, 0x420);
    spu.write16(IRQ_ADDR, 0x421);
    spu.write16(CNT, CNT_ENABLE_IRQ);
    spu.write16(KON_LO, 1);
    let quando = irq_ate(&mut spu, 6 * AMOSTRAS_POR_BLOCO);
    assert_eq!(spu.read16(ENDX_LO) & 1, 1, "o fim da metade A liga o ENDX");
    assert!(
        quando.is_some_and(|n| n >= 4 * AMOSTRAS_POR_BLOCO - 1),
        "depois dos 4 blocos da metade A (0x1100) a voz tem de saltar para o repeat 0x420 \
         (0x2100), escrito ANTES do key on, e ler 0x2108, onde esta a IRQ9; disparou em {quando:?}"
    );
}
