mod support;

use psx_core::bus::{Bus, BusRead, BusWrite};
use psx_core::cdrom_bin_cue::{DiscLayout, TrackInfo, TrackType};
use support::asm;

const CD_BASE: u32 = 0x1F80_1800;
const ESPERA_PRIMEIRA_RESPOSTA: u32 = 0x1_4000;
// docs/cdrom-comandos.md § tabela de timing (06-cdrom.md L2058-2076): a 2a resposta
// tem tempo PROPRIO por comando. GetID: avg 4A00h, max 4C2Bh. Pause single-speed:
// avg 21181Ch, min 20EAEFh. Os goldens usam janelas com folga em torno desses valores.
const GETID_SEGUNDA_MIN: u32 = 0x1000;
const GETID_SEGUNDA_MAX: u32 = 0x8000;
const PAUSE_AINDA_NAO: u32 = 0x10_0000;
const PAUSE_SEGUNDA_MAX: u32 = 0x24_0000;
const ESPERA_GENEROSA: u32 = 0x24_0000;

fn bus() -> Bus {
    asm::bus_with_bios_empty()
}

fn cd_read(bus: &Bus, offset: u32) -> u8 {
    bus.read8::<BusRead>(CD_BASE + offset)
}

fn cd_write(bus: &mut Bus, offset: u32, val: u8) {
    bus.write8::<BusWrite>(CD_BASE + offset, val);
}

fn set_bank(bus: &mut Bus, b: u8) {
    cd_write(bus, 0, b);
}

fn send_command(bus: &mut Bus, cmd: u8) {
    set_bank(bus, 0);
    cd_write(bus, 1, cmd);
    bus.tick_timers(ESPERA_PRIMEIRA_RESPOSTA);
}

fn param_write(bus: &mut Bus, val: u8) {
    set_bank(bus, 0);
    cd_write(bus, 2, val);
}

fn hintsts(bus: &mut Bus) -> u8 {
    set_bank(bus, 1);
    let val = cd_read(bus, 3);
    set_bank(bus, 0);
    val & 0x7
}

fn ack(bus: &mut Bus) {
    set_bank(bus, 1);
    cd_write(bus, 3, 0x07);
    set_bank(bus, 0);
}

fn result_read(bus: &mut Bus) -> u8 {
    set_bank(bus, 0);
    cd_read(bus, 1)
}

fn rslrrdy(bus: &mut Bus) -> bool {
    set_bank(bus, 0);
    cd_read(bus, 0) & (1 << 5) != 0
}

fn rddata_read(bus: &mut Bus) -> u8 {
    set_bank(bus, 0);
    cd_read(bus, 2)
}

fn le_4_bytes_do_setor(bus: &mut Bus) -> [u8; 4] {
    set_bank(bus, 0);
    cd_write(bus, 3, 0x80);
    [
        rddata_read(bus),
        rddata_read(bus),
        rddata_read(bus),
        rddata_read(bus),
    ]
}

fn layout() -> DiscLayout {
    DiscLayout {
        bin_path: "test.bin".to_string(),
        tracks: vec![TrackInfo {
            number: 1,
            track_type: TrackType::Mode1_2048,
            index01_mm: 0,
            index01_ss: 2,
            index01_ff: 0,
            index00_mm: None,
            index00_ss: None,
            index00_ff: None,
            pregap_mm: None,
            pregap_ss: None,
            pregap_ff: None,
        }],
    }
}

fn grava_setor(bin: &mut [u8], frame: usize, marca: [u8; 4]) {
    let base = frame * 2352;
    bin[base] = 0x00;
    for b in bin.iter_mut().skip(base + 1).take(10) {
        *b = 0xFF;
    }
    bin[base + 0x0F] = 0x01;
    bin[base + 0x10..base + 0x14].copy_from_slice(&marca);
}

fn bus_com_dois_setores() -> Bus {
    let mut bin = vec![0u8; 2 * 2352];
    grava_setor(&mut bin, 0, [0xAA, 0xBB, 0xCC, 0xDD]);
    grava_setor(&mut bin, 1, [0x11, 0x22, 0x33, 0x44]);
    let mut bus = bus();
    bus.inject_disc(layout(), bin);
    bus.cdrom_mut().insert_disc();
    bus
}

fn setloc(bus: &mut Bus, mm: u8, ss: u8, ff: u8) {
    param_write(bus, mm);
    param_write(bus, ss);
    param_write(bus, ff);
    send_command(bus, 0x02);
    let _ = result_read(bus);
    ack(bus);
}

#[test]
fn comando_com_int_pendente_so_executa_apos_ack() {
    let mut bus = bus();
    bus.cdrom_mut().insert_disc();
    send_command(&mut bus, 0x0A);
    assert_eq!(hintsts(&mut bus), 3, "INT3 do Init pendente");
    let stat = result_read(&mut bus);
    assert_eq!(stat, 0x02, "stat do Init: motor ligado (bit1), nada mais");

    param_write(&mut bus, 0x20);
    send_command(&mut bus, 0x19);
    assert_eq!(
        hintsts(&mut bus),
        3,
        "com INT pendente o mainloop NAO executa o comando (06-cdrom.md L1984-2000) — \
         o Test(19h,20h) fica retido; e a divida 10.53"
    );
    assert!(
        !rslrrdy(&mut bus),
        "comando retido nao produz resposta: result FIFO segue vazio"
    );

    ack(&mut bus);
    bus.tick_timers(GETID_SEGUNDA_MAX);
    assert_eq!(
        hintsts(&mut bus),
        2,
        "apos o ack, a 2a resposta pendente do Init (INT2) vem antes do comando retido \
         (06-cdrom.md L1997-1999)"
    );
    let _ = result_read(&mut bus);

    ack(&mut bus);
    bus.tick_timers(ESPERA_PRIMEIRA_RESPOSTA);
    assert_eq!(
        hintsts(&mut bus),
        3,
        "so depois do ack do INT2 o Test retido executa e responde INT3"
    );
    assert_eq!(
        result_read(&mut bus),
        0x97,
        "resposta do Test(19h,20h): ano BCD 97h — prova que quem respondeu foi o \
         comando retido, nao um INT requentado"
    );
}

#[test]
fn timing_da_segunda_resposta_e_por_comando() {
    let mut bus = bus();
    bus.cdrom_mut().insert_disc();

    send_command(&mut bus, 0x1A);
    let _ = result_read(&mut bus);
    ack(&mut bus);
    bus.tick_timers(GETID_SEGUNDA_MIN);
    assert_eq!(
        hintsts(&mut bus),
        0,
        "GetID: 1000h < min 4922h — INT2 ainda nao"
    );
    bus.tick_timers(GETID_SEGUNDA_MAX);
    assert_eq!(hintsts(&mut bus), 2, "GetID: INT2 ate max 4C2Bh + folga");
    for _ in 0..8 {
        let _ = result_read(&mut bus);
    }
    ack(&mut bus);

    send_command(&mut bus, 0x09);
    let _ = result_read(&mut bus);
    ack(&mut bus);
    bus.tick_timers(PAUSE_AINDA_NAO);
    assert_eq!(
        hintsts(&mut bus),
        0,
        "Pause: 100000h < min 20EAEFh — INT2 ainda nao; um unico atraso global para \
         todo comando reprova aqui"
    );
    bus.tick_timers(PAUSE_SEGUNDA_MAX);
    assert_eq!(hintsts(&mut bus), 2, "Pause: INT2 ate ~21181Ch + folga");
}

#[test]
fn read_n_avanca_de_setor_a_cada_int1() {
    let mut bus = bus_com_dois_setores();
    setloc(&mut bus, 0x00, 0x02, 0x00);

    send_command(&mut bus, 0x06);
    let _ = result_read(&mut bus);
    ack(&mut bus);
    bus.tick_timers(ESPERA_GENEROSA);
    assert_eq!(hintsts(&mut bus), 1, "INT1 do primeiro setor");
    let _ = result_read(&mut bus);
    let primeiro = le_4_bytes_do_setor(&mut bus);
    assert_eq!(
        primeiro,
        [0xAA, 0xBB, 0xCC, 0xDD],
        "dados do setor 00:02:00"
    );

    ack(&mut bus);
    bus.tick_timers(ESPERA_GENEROSA);
    assert_eq!(hintsts(&mut bus), 1, "INT1 do segundo setor");
    let _ = result_read(&mut bus);
    let segundo = le_4_bytes_do_setor(&mut bus);
    assert_eq!(
        segundo,
        [0x11, 0x22, 0x33, 0x44],
        "a leitura continua avanca sozinha para o setor seguinte (06-cdrom.md L925-926) \
         — reentregar o setor do Setloc corrompe qualquer arquivo > 2KB"
    );
}

#[test]
fn read_s_tambem_avanca_de_setor() {
    let mut bus = bus_com_dois_setores();
    setloc(&mut bus, 0x00, 0x02, 0x00);

    send_command(&mut bus, 0x1B);
    let _ = result_read(&mut bus);
    ack(&mut bus);
    bus.tick_timers(ESPERA_GENEROSA);
    assert_eq!(hintsts(&mut bus), 1, "INT1 do primeiro setor do ReadS");
    let _ = result_read(&mut bus);
    let primeiro = le_4_bytes_do_setor(&mut bus);
    assert_eq!(
        primeiro,
        [0xAA, 0xBB, 0xCC, 0xDD],
        "dados do setor 00:02:00"
    );

    ack(&mut bus);
    bus.tick_timers(ESPERA_GENEROSA);
    assert_eq!(
        hintsts(&mut bus),
        1,
        "ReadN e ReadS leem sequencialmente do mesmo jeito (06-cdrom.md L925-926); \
         ReadS parar apos 1 setor era espelho da implementacao, nao spec"
    );
    let _ = result_read(&mut bus);
    let segundo = le_4_bytes_do_setor(&mut bus);
    assert_eq!(segundo, [0x11, 0x22, 0x33, 0x44], "segundo setor do ReadS");
}

#[test]
fn setloc_nao_tem_segunda_resposta() {
    let mut bus = bus();
    bus.cdrom_mut().insert_disc();
    param_write(&mut bus, 0x00);
    param_write(&mut bus, 0x02);
    param_write(&mut bus, 0x00);
    send_command(&mut bus, 0x02);
    assert_eq!(hintsts(&mut bus), 3, "INT3 do Setloc");
    let _ = result_read(&mut bus);
    ack(&mut bus);
    bus.tick_timers(ESPERA_GENEROSA);
    assert_eq!(
        hintsts(&mut bus),
        0,
        "so 07h,08h,09h,0Ah,12h,15h,16h,1Ah,1Dh,1Eh tem 2a resposta \
         (06-cdrom.md L2004-2014); Setloc nao esta na lista"
    );
}

#[test]
fn int5_na_primeira_resposta_suprime_a_segunda() {
    let mut bus = bus();
    bus.cdrom_mut().insert_disc();
    param_write(&mut bus, 0x00);
    param_write(&mut bus, 0x60);
    param_write(&mut bus, 0x00);
    send_command(&mut bus, 0x02);
    assert_eq!(hintsts(&mut bus), 5, "INT5: ss=60h invalido em BCD");
    let _ = result_read(&mut bus);
    ack(&mut bus);
    bus.tick_timers(ESPERA_GENEROSA);
    assert_eq!(
        hintsts(&mut bus),
        0,
        "se o INT5 e a 1a resposta, a 2a NAO e enviada (06-cdrom.md L2022-2026)"
    );
}
