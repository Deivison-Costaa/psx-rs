mod support;

use psx_core::bus::{Bus, BusRead, BusWrite};
use psx_core::cdrom_bin_cue::{DiscLayout, TrackInfo, TrackType};
use support::asm;

const CD_BASE: u32 = 0x1F80_1800;
const ESPERA_PRIMEIRA_RESPOSTA: u32 = 0x1_4000;
const FRAMES: usize = 32;
const DRQSTS: u8 = 1 << 6;
const PAUSE_COMPLETO: u32 = 0x0030_0000;

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

fn ack(bus: &mut Bus) {
    set_bank(bus, 1);
    cd_write(bus, 3, 0x07);
    set_bank(bus, 0);
}

fn hintsts(bus: &mut Bus) -> u8 {
    set_bank(bus, 1);
    let val = cd_read(bus, 3);
    set_bank(bus, 0);
    val & 0x07
}

fn hsts(bus: &Bus) -> u8 {
    cd_read(bus, 0)
}

fn result_read(bus: &mut Bus) -> u8 {
    set_bank(bus, 0);
    cd_read(bus, 1)
}

fn bfrd(bus: &mut Bus, val: u8) {
    set_bank(bus, 0);
    cd_write(bus, 3, val);
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

fn byte_do_setor(frame: usize, i: usize) -> u8 {
    (frame as u8).wrapping_mul(31).wrapping_add(i as u8)
}

/// Cada setor tem cabecalho com o proprio MSF e um padrao de dados que depende do quadro,
/// para distinguir "o mesmo setor" de "o setor seguinte".
fn disco() -> Vec<u8> {
    let mut bin = vec![0u8; FRAMES * 2352];
    for frame in 0..FRAMES {
        let base = frame * 2352;
        for b in bin.iter_mut().skip(base + 1).take(10) {
            *b = 0xFF;
        }
        bin[base + 0x0D] = 0x02;
        bin[base + 0x0E] = (((frame / 10) << 4) | (frame % 10)) as u8;
        bin[base + 0x0F] = 0x02;
        for i in 0x18..2352 {
            bin[base + i] = byte_do_setor(frame, i);
        }
    }
    bin
}

fn bus_lendo() -> Bus {
    let mut bus = asm::bus_with_bios_empty();
    bus.inject_disc(layout(), disco());
    bus.cdrom_mut().insert_disc();
    param_write(&mut bus, 0x20);
    send_command(&mut bus, 0x0E);
    let _ = result_read(&mut bus);
    ack(&mut bus);
    param_write(&mut bus, 0x00);
    param_write(&mut bus, 0x02);
    param_write(&mut bus, 0x00);
    send_command(&mut bus, 0x02);
    let _ = result_read(&mut bus);
    ack(&mut bus);
    send_command(&mut bus, 0x06);
    let _ = result_read(&mut bus);
    ack(&mut bus);
    let busca = bus.cdrom().second_response_cycles() as u32;
    bus.tick_timers(busca);
    assert_eq!(hintsts(&mut bus), 1, "o primeiro setor chega com INT1");
    let _ = result_read(&mut bus);
    bus
}

/// Aceita o setor do INT1 corrente e consome os 12 bytes de cabecalho+subcabecalho.
fn aceita_e_le_cabecalho(bus: &mut Bus) {
    ack(bus);
    bfrd(bus, 0x00);
    bfrd(bus, 0x80);
    for _ in 0..12 {
        let _ = cd_read(bus, 2);
    }
}

/// § HSTS (06-cdrom.md L67): DRQSTS = "one or more RDDATA reads ... pending". § Buffer
/// Overrun Timings (L783-785): o setor pedido fica "locked". Nada liga o bit ao drive estar
/// lendo: o Pause para o drive, nao esvazia o FIFO de dados ja aceito.
#[test]
fn pause_nao_esvazia_o_fifo_de_dados_ja_aceito() {
    let mut bus = bus_lendo();
    aceita_e_le_cabecalho(&mut bus);
    send_command(&mut bus, 0x09);
    assert_eq!(hintsts(&mut bus), 3, "Pause responde INT3 primeiro");
    let _ = result_read(&mut bus);
    ack(&mut bus);

    assert_ne!(
        hsts(&bus) & DRQSTS,
        0,
        "com 2328 bytes ainda no FIFO o DRQSTS continua ligado depois do Pause"
    );
    assert_eq!(
        cd_read(&bus, 2),
        byte_do_setor(0, 0x18),
        "o proximo byte e' o 13o do setor aceito (dado travado, 06-cdrom.md L783-785)"
    );
}

#[test]
fn fifo_continua_legivel_depois_do_segundo_response_do_pause() {
    let mut bus = bus_lendo();
    aceita_e_le_cabecalho(&mut bus);
    send_command(&mut bus, 0x09);
    let _ = result_read(&mut bus);
    ack(&mut bus);
    bus.tick_timers(PAUSE_COMPLETO);
    assert_eq!(hintsts(&mut bus), 2, "Pause completa com INT2");
    let _ = result_read(&mut bus);
    ack(&mut bus);

    assert_ne!(hsts(&bus) & DRQSTS, 0);
    let dados: Vec<u8> = (0..4).map(|_| cd_read(&bus, 2)).collect();
    let esperado: Vec<u8> = (0x18..0x1C).map(|i| byte_do_setor(0, i)).collect();
    assert_eq!(dados, esperado);
}

/// § HCHPCTL (06-cdrom.md L104-110): BFRD=0 descarta o FIFO; sem pedido nao ha DRQSTS.
#[test]
fn bfrd_zero_esvazia_o_fifo() {
    let mut bus = bus_lendo();
    aceita_e_le_cabecalho(&mut bus);
    bfrd(&mut bus, 0x00);
    assert_eq!(hsts(&bus) & DRQSTS, 0);
}
