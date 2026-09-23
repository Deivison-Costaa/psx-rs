mod support;

use psx_core::bus::{Bus, BusRead, BusWrite};
use support::asm;

const JOY_DATA: u32 = 0x1F80_1040;
const JOY_STAT: u32 = 0x1F80_1044;
const JOY_CTRL: u32 = 0x1F80_104A;
const JOY_MODE: u32 = 0x1F80_1048;
const JOY_BAUD: u32 = 0x1F80_104E;
const CTRL_BIOS: u16 = 0x1003;
const STAT_DSR: u16 = 1 << 7;
const STAT_IRQ: u16 = 1 << 9;
const CTRL_ACK: u16 = 1 << 4;

// 10-controllers-memcards.md, "Address byte (01h) being sent": o periferico segura /ACK baixo
// por pelo menos 2 us (68 ciclos) e depois solta; "DSR (/ACK) ... Interrupt": o LOW dura
// cerca de 100 ciclos. 17-sio.md SIO_STAT.7: bit 7 = 1 enquanto /ACK esta baixo.
const PULSO_MINIMO: u32 = 68;
const PULSO_FOLGADO: u32 = 400;
const PASSO: u32 = 4;
const LIMITE: u32 = 8 * 136 + 3386;

fn porta() -> Bus {
    let mut bus = asm::bus_with_bios_empty();
    bus.sio_mut().connect_digital_pad(true);
    bus.write16::<BusWrite>(JOY_BAUD, 0x0088);
    bus.write16::<BusWrite>(JOY_MODE, 0x000D);
    bus.write16::<BusWrite>(JOY_CTRL, CTRL_BIOS);
    bus
}

fn stat(bus: &Bus) -> u16 {
    bus.read16::<BusRead>(JOY_STAT)
}

fn ate_o_ack(bus: &mut Bus) {
    let mut decorrido = 0;
    while stat(bus) & STAT_DSR == 0 {
        assert!(decorrido < LIMITE, "o /ACK nunca chegou");
        bus.tick_timers(PASSO);
        decorrido += PASSO;
    }
}

#[test]
fn dsr_cai_sozinho_quando_o_pulso_de_ack_termina() {
    let mut bus = porta();
    bus.write8::<BusWrite>(JOY_DATA, 0x01);
    let _ = bus.read8::<BusRead>(JOY_DATA);

    ate_o_ack(&mut bus);
    bus.tick_timers(PULSO_FOLGADO);

    assert_eq!(
        stat(&bus) & STAT_DSR,
        0,
        "STAT.7 espelha a linha /ACK: o pulso acabou, mesmo sem ninguem ler o RX depois"
    );
    assert_eq!(
        stat(&bus) & STAT_IRQ,
        STAT_IRQ,
        "STAT.9 e sticky: continua ate o CTRL.4"
    );
}

#[test]
fn dsr_fica_baixo_pelo_menos_2us() {
    let mut bus = porta();
    bus.write8::<BusWrite>(JOY_DATA, 0x01);

    ate_o_ack(&mut bus);
    bus.tick_timers(PULSO_MINIMO - 2 * PASSO);

    assert_eq!(
        stat(&bus) & STAT_DSR,
        STAT_DSR,
        "o periferico segura /ACK baixo por no minimo 2 us"
    );
}

#[test]
fn ctrl4_nao_reconhece_stat9_com_ack_ainda_baixo() {
    let mut bus = porta();
    bus.write8::<BusWrite>(JOY_DATA, 0x01);
    ate_o_ack(&mut bus);

    bus.write16::<BusWrite>(JOY_CTRL, CTRL_BIOS | CTRL_ACK);
    assert_eq!(
        stat(&bus) & STAT_IRQ,
        STAT_IRQ,
        "SIO0_STAT.9 nao pode ser reconhecido enquanto /ACK ainda esta baixo"
    );

    bus.tick_timers(PULSO_FOLGADO);
    bus.write16::<BusWrite>(JOY_CTRL, CTRL_BIOS | CTRL_ACK);
    assert_eq!(
        stat(&bus) & STAT_IRQ,
        0,
        "com /ACK solto (STAT.7=0) o CTRL.4 limpa STAT.9"
    );
}

// Sequencia do libpad do FF9: le o RX do byte 01h antes do /ACK chegar, espera I_STAT.7, manda
// 42h e exige STAT.7=0 em ate 480 ciclos antes de reconhecer — senao aborta a leitura.
#[test]
fn ack_seguinte_encontra_dsr_solto() {
    let mut bus = porta();
    bus.write8::<BusWrite>(JOY_DATA, 0x01);
    let _ = bus.read8::<BusRead>(JOY_DATA);
    ate_o_ack(&mut bus);

    bus.tick_timers(300);
    bus.write8::<BusWrite>(JOY_DATA, 0x42);
    bus.tick_timers(200);

    assert_eq!(
        stat(&bus) & STAT_DSR,
        0,
        "o /ACK do byte anterior ja terminou e o do 42h ainda nao comecou"
    );
}
