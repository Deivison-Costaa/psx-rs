mod support;

use psx_core::bus::{Bus, BusRead, BusWrite};
use psx_core::cdrom_bin_cue::{DiscLayout, TrackInfo, TrackType};
use support::asm;

const CD_BASE: u32 = 0x1F80_1800;
const ESPERA_PRIMEIRA_RESPOSTA: u32 = 0x1_4000;
const ESPERA_PRIMEIRO_SETOR: u32 = 0x6000;
// § INT1 Rate (06-cdrom.md L2093-2101): SystemClock*930h/4/44100Hz em velocidade normal.
const CADENCIA: u32 = 451_584;
const FRAMES: usize = 64;

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

fn hclrctl_write(bus: &mut Bus, val: u8) {
    set_bank(bus, 1);
    cd_write(bus, 3, val);
    set_bank(bus, 0);
}

fn hintsts(bus: &mut Bus) -> u8 {
    set_bank(bus, 1);
    let val = cd_read(bus, 3);
    set_bank(bus, 0);
    val & 0x07
}

fn result_read(bus: &mut Bus) -> u8 {
    set_bank(bus, 0);
    cd_read(bus, 1)
}

fn layout() -> DiscLayout {
    DiscLayout {
        bin_path: "test.bin".to_string(),
        tracks: vec![TrackInfo {
            number: 1,
            file: "test.bin".to_string(),
            start_lba: 150,
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

fn int_to_bcd(v: u8) -> u8 {
    ((v / 10) << 4) | (v % 10)
}

/// Cada frame carrega o proprio MSF no cabecalho (010h-013h do setor cru sao 00Ch-00Fh
/// contando do sync), que e' exatamente o que o GetlocL devolve.
fn disco_com_cabecalho_por_setor() -> Vec<u8> {
    let mut bin = vec![0u8; FRAMES * 2352];
    for frame in 0..FRAMES {
        let base = frame * 2352;
        bin[base] = 0x00;
        for b in bin.iter_mut().skip(base + 1).take(10) {
            *b = 0xFF;
        }
        let ff = (frame % 75) as u8;
        let ss = 2 + (frame / 75) as u8;
        bin[base + 0x0C] = 0x00;
        bin[base + 0x0D] = int_to_bcd(ss);
        bin[base + 0x0E] = int_to_bcd(ff);
        bin[base + 0x0F] = 0x01;
        for (i, b) in bin.iter_mut().skip(base + 0x10).take(4).enumerate() {
            *b = 0x50 + i as u8;
        }
    }
    bin
}

fn bus_com_disco() -> Bus {
    let mut bus = asm::bus_with_bios_empty();
    bus.inject_disc(layout(), disco_com_cabecalho_por_setor());
    bus.cdrom_mut().insert_disc();
    bus
}

/// Setloc(0:2:0) + ReadN, com o INT3 do ReadN ja reconhecido.
fn setloc_0_2_0_e_read(bus: &mut Bus) {
    param_write(bus, 0x00);
    param_write(bus, 0x02);
    param_write(bus, 0x00);
    send_command(bus, 0x02);
    let _ = result_read(bus);
    hclrctl_write(bus, 0x07);

    send_command(bus, 0x06);
    let _ = result_read(bus);
    hclrctl_write(bus, 0x07);
}

/// GetlocL (comando 10h) apos o ack do INT1 corrente: devolve (amm, ass, asect) do setor
/// de dado mais NOVO recebido pelo controlador (06-cdrom.md L1057-1059).
fn getloc_l_apos_ack(bus: &mut Bus) -> (u8, u8, u8) {
    hclrctl_write(bus, 0x07);
    // § Sector Buffer (06-cdrom.md L2112-2117): reconhecer o INT1 antigo faz o controlador
    // "jump directly to INT1 for the newest sector" na hora — com setor em buffer, o ack
    // nao deixa a linha livre, levanta o INT1 seguinte. Reconhece esse tambem, senao o
    // GetlocL fica retido e responde o stat do INT1 em vez do cabecalho.
    if hintsts(bus) == 1 {
        let _ = result_read(bus);
        hclrctl_write(bus, 0x07);
    }
    send_command(bus, 0x10);
    let amm = result_read(bus);
    let ass = result_read(bus);
    let asect = result_read(bus);
    assert_eq!(
        hintsts(bus),
        3,
        "GetlocL tem que responder INT3 com o cabecalho, nao INT5 de erro"
    );
    hclrctl_write(bus, 0x07);
    (amm, ass, asect)
}

/// Fluxo normal da spec (06-cdrom.md L2128-2135): processando cada INT1 na hora, os
/// setores saem em sequencia 0:2:0, 0:2:1, ...
#[test]
fn sem_atraso_a_posicao_avanca_um_setor_por_int1() {
    let mut bus = bus_com_disco();
    setloc_0_2_0_e_read(&mut bus);

    bus.tick_timers(ESPERA_PRIMEIRO_SETOR);
    assert_eq!(hintsts(&mut bus), 1, "primeiro setor tem que levantar INT1");
    assert_eq!(
        getloc_l_apos_ack(&mut bus),
        (0x00, 0x02, 0x00),
        "o primeiro setor lido depois de Setloc(0:2:0) e' 0:2:0"
    );

    bus.tick_timers(CADENCIA);
    assert_eq!(hintsts(&mut bus), 1, "segundo setor tem que levantar INT1");
    assert_eq!(
        getloc_l_apos_ack(&mut bus),
        (0x00, 0x02, 0x01),
        "processando INT1 na hora, o setor seguinte e' 0:2:1"
    );
}

/// § Sector Buffer (06-cdrom.md L2118-2126): "one should process INT1's as soon as
/// possible (ie. before the cdrom controller receives and skips further sectors).
/// Otherwise sectors would be lost without notice". O drive fisico nao para de girar
/// enquanto a CPU nao da ack — os casos medidos em hardware (L2136-2168) mostram a
/// posicao pulando de 0:2:1 pra 0:2:6 depois de um delay curto.
#[test]
fn cinco_intervalos_sem_ack_avancam_cinco_setores() {
    let mut bus = bus_com_disco();
    setloc_0_2_0_e_read(&mut bus);

    bus.tick_timers(ESPERA_PRIMEIRO_SETOR);
    assert_eq!(hintsts(&mut bus), 1, "primeiro setor tem que levantar INT1");

    bus.tick_timers(5 * CADENCIA);

    assert_eq!(
        getloc_l_apos_ack(&mut bus),
        (0x00, 0x02, 0x05),
        "sem ack da CPU o drive continua girando: 5 intervalos de setor depois, o setor \
         mais novo recebido pelo controlador e' 0:2:5, nao 0:2:0 — a posicao de leitura \
         nao pode ficar parada esperando o ack (06-cdrom.md L2118-2126)"
    );
}

/// Mesmo efeito com atraso maior (06-cdrom.md L2147-2155: delay(2) leva a posicao pra
/// 0:2:9/0:2:11) — o avanco e' proporcional ao tempo, nao ao numero de acks.
#[test]
fn atraso_longo_sem_ack_avanca_proporcional_ao_tempo() {
    let mut bus = bus_com_disco();
    setloc_0_2_0_e_read(&mut bus);

    bus.tick_timers(ESPERA_PRIMEIRO_SETOR);
    assert_eq!(hintsts(&mut bus), 1, "primeiro setor tem que levantar INT1");

    bus.tick_timers(12 * CADENCIA);

    assert_eq!(
        getloc_l_apos_ack(&mut bus),
        (0x00, 0x02, 0x12),
        "12 intervalos de setor sem ack avancam 12 setores (0:2:12 em BCD), como os casos \
         medidos em hardware real (06-cdrom.md L2136-2168)"
    );
}

/// Nenhum INT1 novo nasce enquanto a anterior nao foi reconhecida — o setor antigo e'
/// perdido SILENCIOSAMENTE, sem flag de overrun nem interrupcao de erro
/// (06-cdrom.md L2123-2126).
#[test]
fn sem_ack_nao_ha_flag_de_overrun_nem_int_extra() {
    let mut bus = bus_com_disco();
    setloc_0_2_0_e_read(&mut bus);

    bus.tick_timers(ESPERA_PRIMEIRO_SETOR);
    assert_eq!(hintsts(&mut bus), 1, "primeiro setor tem que levantar INT1");

    bus.tick_timers(5 * CADENCIA);

    assert_eq!(
        hintsts(&mut bus),
        1,
        "os setores pulados nao geram INT5 nem qualquer outro codigo — a INT1 antiga \
         continua ali, sozinha, ate a CPU reconhecer"
    );
}
