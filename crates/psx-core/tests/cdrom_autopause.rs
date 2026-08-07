mod support;

use psx_core::bus::{Bus, BusRead};
use psx_core::cdrom_bin_cue::{DiscLayout, TrackInfo, TrackType};
use support::asm;

const CD_BASE: u32 = 0x1F80_1800;
const ESPERA_PRIMEIRA_RESPOSTA: u32 = 0x1_4000;
// § INT1 Rate (06-cdrom.md L2093-2101): cadencia real de relatorio em velocidade normal
// (nenhum destes testes liga o bit7/Speed do Setmode) — 451584 ciclos, nao os 0x6000
// que valiam so pro modelo antigo de custo fixo.
const ESPERA_SEGUNDA_RESPOSTA: u32 = 451_584;

// § Setmode - Command 0Eh,mode (L685) de docs/reference/06-cdrom.md: bit1 = autopause,
// bit2 = report. O Rayman manda 07h, medido na iteracao 0180.
const REPORT: u8 = 0x04;
const AUTOPAUSE: u8 = 0x02;

fn cd_write(bus: &mut Bus, offset: u32, val: u8) {
    bus.write8::<BusRead>(CD_BASE + offset, val);
}

fn set_bank(bus: &mut Bus, b: u8) {
    cd_write(bus, 0, b);
}

fn hintsts(bus: &mut Bus) -> u8 {
    set_bank(bus, 1);
    let val = bus.read8::<BusRead>(CD_BASE + 3);
    set_bank(bus, 0);
    val
}

fn ack(bus: &mut Bus) {
    set_bank(bus, 1);
    cd_write(bus, 3, 0x07);
    set_bank(bus, 0);
    bus.tick_timers(ESPERA_SEGUNDA_RESPOSTA);
}

fn result_read(bus: &mut Bus) -> u8 {
    set_bank(bus, 0);
    bus.read8::<BusRead>(CD_BASE + 1)
}

fn send(bus: &mut Bus, cmd: u8, params: &[u8]) {
    set_bank(bus, 0);
    for p in params {
        cd_write(bus, 2, *p);
    }
    cd_write(bus, 1, cmd);
    bus.tick_timers(ESPERA_PRIMEIRA_RESPOSTA);
}

fn trilha(number: u8, mm: u8, ss: u8, ff: u8, tipo: TrackType) -> TrackInfo {
    TrackInfo {
        number,
        file: format!("t{number}.bin"),
        start_lba: mm as u32 * 60 * 75 + ss as u32 * 75 + ff as u32,
        track_type: tipo,
        index01_mm: mm,
        index01_ss: ss,
        index01_ff: ff,
        index00_mm: None,
        index00_ss: None,
        index00_ff: None,
        pregap_mm: None,
        pregap_ss: None,
        pregap_ff: None,
    }
}

// Trilha 1 no LBA 0 e trilha 2 no LBA 75 (um segundo adiante), perto o bastante para o teste
// cruzar a fronteira em poucos relatorios.
fn bus_com_duas_trilhas(mode: u8, mm: u8, ss: u8, ff: u8) -> Bus {
    let mut bus = asm::bus_with_bios_empty();
    let layout = DiscLayout {
        bin_path: "t1.bin".to_string(),
        tracks: vec![
            trilha(1, 0, 0, 0, TrackType::Mode2_2352),
            trilha(2, 0, 1, 0, TrackType::Audio),
        ],
    };
    bus.inject_disc(layout, vec![0u8; 2352 * 400]);
    bus.cdrom_mut().insert_disc();
    send(&mut bus, 0x0E, &[mode]);
    ack(&mut bus);
    send(&mut bus, 0x02, &[mm, ss, ff]);
    ack(&mut bus);
    bus
}

/// Toca ate sair um INT diferente de INT1, devolvendo (int, primeiro byte da resposta).
fn toca_ate_mudar_de_int(bus: &mut Bus, limite: usize) -> (u8, u8) {
    send(bus, 0x03, &[]);
    for _ in 0..limite {
        ack(bus);
        let int = hintsts(bus) & 0x7;
        if int != 1 {
            return (int, result_read(bus));
        }
        let _ = (0..8).map(|_| result_read(bus)).count();
    }
    (1, 0)
}

// § AutoPause --> INT4(stat) (L1267-1272) de docs/reference/06-cdrom.md:
// "Setmode.bit1=1: AutoPause=On --> Issue INT4(stat) and PAUSE at end of TRACK".
#[test]
fn com_autopause_o_fim_da_trilha_gera_int4() {
    let mut bus = bus_com_duas_trilhas(REPORT | AUTOPAUSE, 0x00, 0x02, 0x00);
    let (int, stat) = toca_ate_mudar_de_int(&mut bus, 40);
    assert_eq!(int, 4, "cruzar a fronteira da trilha tem de gerar INT4");
    assert_eq!(
        stat & 0x80,
        0,
        "o autopause pausa: stat.bit7 (Play) sai desligado"
    );
}

// § AutoPause (L1271): "Setmode.bit1=0: AutoPause=Off --> Issue INT4(stat) and STOP at end of
// DISC". Sem o bit, a fronteira de trilha nao interrompe nada.
#[test]
fn sem_autopause_a_fronteira_de_trilha_nao_interrompe() {
    let mut bus = bus_com_duas_trilhas(REPORT, 0x00, 0x02, 0x00);
    let (int, _) = toca_ate_mudar_de_int(&mut bus, 40);
    assert_eq!(
        int, 1,
        "sem autopause os relatorios seguem atravessando a trilha"
    );
}

// A pausa e de verdade: depois do INT4 nao sai mais relatorio.
#[test]
fn depois_do_autopause_nao_sai_mais_relatorio() {
    let mut bus = bus_com_duas_trilhas(REPORT | AUTOPAUSE, 0x00, 0x02, 0x00);
    let (int, _) = toca_ate_mudar_de_int(&mut bus, 40);
    assert_eq!(int, 4, "precondicao: o INT4 saiu");
    ack(&mut bus);
    assert_eq!(
        hintsts(&mut bus) & 0x7,
        0,
        "apos o autopause o Play acabou; nao ha segunda resposta pendente"
    );
}
