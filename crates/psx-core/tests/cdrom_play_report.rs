mod support;

use psx_core::bus::{Bus, BusRead};
use support::asm;

const CD_BASE: u32 = 0x1F80_1800;
const ESPERA_PRIMEIRA_RESPOSTA: u32 = 0x1_4000;
const ESPERA_SEGUNDA_RESPOSTA: u32 = 0x6000;

// § Setmode - Command 0Eh,mode (L685) de docs/reference/06-cdrom.md: bit2 = report.
const MODE_REPORT: u8 = 0x04;

fn bus() -> Bus {
    asm::bus_with_bios_empty()
}

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

fn setup(mode: u8, mm: u8, ss: u8, ff: u8) -> Bus {
    let mut bus = bus();
    bus.cdrom_mut().insert_disc();
    send(&mut bus, 0x0E, &[mode]);
    ack(&mut bus);
    send(&mut bus, 0x02, &[mm, ss, ff]);
    ack(&mut bus);
    bus
}

fn le_resposta(bus: &mut Bus, n: usize) -> Vec<u8> {
    (0..n).map(|_| result_read(bus)).collect()
}

// § Error Codes (L1020) de docs/reference/06-cdrom.md: 80h aparece em 02h..09h — o Play (03h)
// esta nessa faixa — quando o disco esta ausente.
#[test]
fn play_sem_disco_devolve_int5_com_erro_80h() {
    let mut bus = bus();
    send(&mut bus, 0x03, &[]);
    assert_eq!(
        hintsts(&mut bus) & 0x7,
        5,
        "sem disco o Play tem de dar INT5"
    );
    let stat = result_read(&mut bus);
    assert_eq!(stat & 0x01, 0x01, "stat.bit0 (erro) ligado");
    assert_eq!(result_read(&mut bus), 0x80, "byte de erro = 80h");
}

// § Play - Command 03h (L1201) de docs/reference/06-cdrom.md: "--> INT3(stat)". § CDROM - Status
// (L996) do mesmo arquivo: bit7 = Play.
#[test]
fn play_com_disco_confirma_com_int3_e_stat_de_play() {
    let mut bus = setup(0, 0x00, 0x02, 0x00);
    send(&mut bus, 0x03, &[]);
    assert_eq!(hintsts(&mut bus) & 0x7, 3, "Play confirma com INT3");
    let stat = result_read(&mut bus);
    assert_eq!(stat & 0x80, 0x80, "stat.bit7 (Play) ligado durante o Play");
    assert_eq!(stat & 0x01, 0, "sem erro");
}

// § Setmode bits used for Play command (L1238-1245) de docs/reference/06-cdrom.md: sem o bit2 do
// Setmode o Play nao gera relatorio nenhum.
#[test]
fn sem_o_bit_de_report_o_play_nao_gera_int1() {
    let mut bus = setup(0, 0x00, 0x02, 0x00);
    send(&mut bus, 0x03, &[]);
    ack(&mut bus);
    assert_eq!(
        hintsts(&mut bus) & 0x7,
        0,
        "sem report o Play nao produz segunda resposta"
    );
}

// § Report (L1246-1256) de docs/reference/06-cdrom.md: INT1 com OITO bytes —
// stat,track,index,mm/amm,ss+80h/ass,sect/asect,peaklo,peakhi.
#[test]
fn com_report_o_play_gera_int1_de_oito_bytes() {
    let mut bus = setup(MODE_REPORT, 0x00, 0x02, 0x00);
    send(&mut bus, 0x03, &[]);
    ack(&mut bus);
    assert_eq!(hintsts(&mut bus) & 0x7, 1, "com report o Play gera INT1");
    let r = le_resposta(&mut bus, 8);
    assert_eq!(r.len(), 8, "relatorio tem oito bytes");
    assert_eq!(r[0] & 0x80, 0x80, "byte 0 = stat, com bit7 de Play");
    assert_eq!(r[1], 0x01, "byte 1 = numero da trilha");
    assert_eq!(r[2], 0x01, "byte 2 = index");
}

// § Report (L1254-1256): amm/ass/asect saem em asect=00h,20h,40h,60h (tempo absoluto);
// mm/ss+80h/sect saem em asect=10h,30h,50h,70h (dentro da trilha, marcado pelo bit7 de ss).
#[test]
fn relatorio_alterna_entre_tempo_absoluto_e_relativo_a_trilha() {
    let mut bus = setup(MODE_REPORT, 0x00, 0x02, 0x00);
    send(&mut bus, 0x03, &[]);

    let mut absolutos = 0;
    let mut relativos = 0;
    for _ in 0..8 {
        ack(&mut bus);
        if hintsts(&mut bus) & 0x7 != 1 {
            break;
        }
        let r = le_resposta(&mut bus, 8);
        if r[4] & 0x80 == 0 {
            absolutos += 1;
            assert_eq!(
                r[5] % 0x20,
                0,
                "relatorio absoluto sai em asect 00h/20h/40h/60h, achei {:02X}h",
                r[5]
            );
        } else {
            relativos += 1;
        }
    }
    assert!(
        absolutos >= 2,
        "esperava varios relatorios de tempo absoluto, vi {absolutos}"
    );
    assert!(
        relativos >= 2,
        "esperava varios relatorios relativos a trilha (ss com bit7), vi {relativos}"
    );
}

// § Report (L1246-1252): os relatorios sao repetidos enquanto o Play continua, e a posicao
// avanca — se ela nao avancar, quem usa o relatorio como contador de tempo espera para sempre.
#[test]
fn a_posicao_do_relatorio_avanca_entre_interrupcoes() {
    let mut bus = setup(MODE_REPORT, 0x00, 0x02, 0x00);
    send(&mut bus, 0x03, &[]);

    let mut absolutas = Vec::new();
    for _ in 0..6 {
        ack(&mut bus);
        if hintsts(&mut bus) & 0x7 != 1 {
            break;
        }
        let r = le_resposta(&mut bus, 8);
        if r[4] & 0x80 == 0 {
            absolutas.push((r[3], r[4], r[5]));
        }
    }
    assert!(
        absolutas.len() >= 2,
        "precisa de pelo menos dois relatorios absolutos para comparar"
    );
    assert!(
        absolutas[1] > absolutas[0],
        "a posicao absoluta tem de avancar: {:?} -> {:?}",
        absolutas[0],
        absolutas[1]
    );
}
