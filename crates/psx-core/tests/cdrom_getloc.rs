mod support;

use psx_core::bus::{Bus, BusRead, BusWrite};
use psx_core::cdrom_bin_cue::DiscLayout;
use support::asm;

fn layout_vazia() -> DiscLayout {
    DiscLayout {
        bin_path: "test.bin".to_string(),
        tracks: vec![],
    }
}

const CD_BASE: u32 = 0x1F80_1800;
const ESPERA_PRIMEIRA_RESPOSTA: u32 = 0x1_4000;
// 06-cdrom.md L333-337: 2a resposta so apos o ack, com atraso fisico (divida 10.53).
const ESPERA_SEGUNDA_RESPOSTA: u32 = 0x6000;

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

fn param_write(bus: &mut Bus, val: u8) {
    set_bank(bus, 0);
    cd_write(bus, 2, val);
}

fn manda_comando(bus: &mut Bus, cmd: u8) {
    set_bank(bus, 0);
    cd_write(bus, 1, cmd);
    bus.tick_timers(ESPERA_PRIMEIRA_RESPOSTA);
}

fn hintsts(bus: &mut Bus) -> u8 {
    set_bank(bus, 1);
    let val = cd_read(bus, 3) & 0x7;
    set_bank(bus, 0);
    val
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

fn le_resultado(bus: &mut Bus, n: usize) -> Vec<u8> {
    (0..n).map(|_| result_read(bus)).collect()
}

fn grava_setor_com_cabecalho(bin: &mut [u8], frame: usize, cabecalho: [u8; 8]) {
    let base = frame * 2352;
    bin[base] = 0x00;
    for b in bin.iter_mut().skip(base + 1).take(10) {
        *b = 0xFF;
    }
    bin[base + 0x0C..base + 0x14].copy_from_slice(&cabecalho);
}

// amm=0xAA/ass=0xBB/asect=0xCC deliberadamente NAO batem com a posicao real (00:02:00) do
// Setloc/ReadN: prova que o GetlocL le os 8 bytes crus do setor, e nao recalcula a partir do
// `read_pos` que o driver de leitura mantem.
const CABECALHO: [u8; 8] = [0xAA, 0xBB, 0xCC, 0x02, 0x11, 0x22, 0x33, 0x44];

fn disco_com_um_setor() -> Vec<u8> {
    let mut bin = vec![0u8; 2352];
    grava_setor_com_cabecalho(&mut bin, 0, CABECALHO);
    bin
}

// § GetlocL - Command 10h (L1052) de docs/reference/06-cdrom.md.
#[test]
fn getlocl_devolve_cabecalho_e_subheader_do_ultimo_setor_lido() {
    let mut bus = bus();
    bus.inject_disc(layout_vazia(), disco_com_um_setor());
    bus.cdrom_mut().insert_disc();

    param_write(&mut bus, 0x00);
    param_write(&mut bus, 0x02);
    param_write(&mut bus, 0x00);
    manda_comando(&mut bus, 0x02); // Setloc
    let _ = result_read(&mut bus);
    ack(&mut bus);

    manda_comando(&mut bus, 0x06); // ReadN
    let _ = result_read(&mut bus); // INT3
    ack(&mut bus);
    bus.tick_timers(ESPERA_SEGUNDA_RESPOSTA);
    let _ = result_read(&mut bus); // INT1
    ack(&mut bus);

    manda_comando(&mut bus, 0x10); // GetlocL
    assert_eq!(
        hintsts(&mut bus),
        3,
        "INT3: GetlocL responde de uma vez, sem 2a resposta"
    );
    assert_eq!(
        le_resultado(&mut bus, 8),
        CABECALHO,
        "spec § GetlocL: amm,ass,asect,mode,file,channel,sm,ci — os 8 bytes crus do cabecalho \
         mais sub-header do setor recem-lido (04h..0Bh do buffer do setor)"
    );
}

#[test]
fn getlocl_erro_80h_com_motor_parado() {
    let mut bus = bus();

    manda_comando(&mut bus, 0x10); // GetlocL, sem insert_disc: motor parado

    assert_eq!(
        hintsts(&mut bus),
        5,
        "INT5: spec § GetlocL — 'Error if disc is spun down'"
    );
    let stat = result_read(&mut bus);
    assert_eq!(stat & 0x01, 0x01, "stat.bit0 (erro) ligado");
    assert_eq!(result_read(&mut bus), 0x80, "byte de erro = 80h");
}

#[test]
fn getlocl_erro_80h_durante_play() {
    let mut bus = bus();
    bus.cdrom_mut().insert_disc();

    manda_comando(&mut bus, 0x03); // Play (sem report: sem 2a resposta)
    let _ = result_read(&mut bus);
    ack(&mut bus);

    manda_comando(&mut bus, 0x10); // GetlocL durante o play

    assert_eq!(
        hintsts(&mut bus),
        5,
        "INT5: spec § GetlocL (L1062) — 'fails ... when playing Audio CDs'"
    );
    let stat = result_read(&mut bus);
    assert_eq!(stat & 0x01, 0x01, "stat.bit0 (erro) ligado");
    assert_eq!(result_read(&mut bus), 0x80, "byte de erro = 80h");
}

// docs/iterations/0175-cdrom-oraculo.md mediu no hardware real: 'GetlocL failed, IRQ=5' no
// inicio da suite `cdrom/getloc`, antes de qualquer ReadN ter completado — mesmo com motor
// girando. A spec nao lista essa condicao por nome, mas o gabarito de hardware sim.
#[test]
fn getlocl_erro_80h_antes_de_qualquer_leitura() {
    let mut bus = bus();
    bus.cdrom_mut().insert_disc();

    param_write(&mut bus, 0x00);
    param_write(&mut bus, 0x02);
    param_write(&mut bus, 0x00);
    manda_comando(&mut bus, 0x02); // Setloc, sem ReadN
    let _ = result_read(&mut bus);
    ack(&mut bus);

    manda_comando(&mut bus, 0x10); // GetlocL sem nenhum setor jamais lido

    assert_eq!(
        hintsts(&mut bus),
        5,
        "docs/iterations/0175-cdrom-oraculo.md: GetlocL falha antes de qualquer leitura"
    );
    assert_eq!(
        result_read(&mut bus) & 0x01,
        0x01,
        "stat.bit0 (erro) ligado"
    );
    assert_eq!(result_read(&mut bus), 0x80, "byte de erro = 80h");
}

// § GetlocP - Command 11h (L1073) de docs/reference/06-cdrom.md.
#[test]
fn getlocp_devolve_trilha_index_posicao_relativa_e_absoluta() {
    let mut bus = bus();
    bus.cdrom_mut().insert_disc();

    param_write(&mut bus, 0x00);
    param_write(&mut bus, 0x05);
    param_write(&mut bus, 0x00);
    manda_comando(&mut bus, 0x02); // Setloc a 00:05:00
    let _ = result_read(&mut bus);
    ack(&mut bus);

    // ReadN grava read_pos = 00:05:00 na 1a resposta, antes de a 2a (que avanca 1 quadro)
    // sequer ser agendada. O truque de 2 acks entrega o GetlocP em linha, sem tick, antes que
    // o avanco do read_pos aconteca — ver docs/iterations/0207-cdrom-getloc.md.
    set_bank(&mut bus, 0);
    cd_write(&mut bus, 1, 0x06);
    bus.tick_timers(ESPERA_PRIMEIRA_RESPOSTA);
    let _ = result_read(&mut bus);
    ack(&mut bus);

    set_bank(&mut bus, 0);
    cd_write(&mut bus, 1, 0x11); // GetlocP
    ack(&mut bus);

    assert_eq!(hintsts(&mut bus), 3, "INT3: GetlocP responde de uma vez");
    assert_eq!(
        le_resultado(&mut bus, 8),
        vec![0x01, 0x01, 0x00, 0x03, 0x00, 0x00, 0x05, 0x00],
        "track=01h, index=01h, relativo=00:03:00 (5s - inicio padrao 00:02:00 sem TOC), \
         absoluto=00:05:00 — tudo em BCD"
    );
}

#[test]
fn getlocp_erro_80h_com_motor_parado() {
    let mut bus = bus();

    manda_comando(&mut bus, 0x11); // GetlocP, sem insert_disc: motor parado

    assert_eq!(
        hintsts(&mut bus),
        5,
        "INT5: spec § GetlocP — 'Error if disc is spun down'"
    );
    let stat = result_read(&mut bus);
    assert_eq!(stat & 0x01, 0x01, "stat.bit0 (erro) ligado");
    assert_eq!(result_read(&mut bus), 0x80, "byte de erro = 80h");
}
