mod support;

use psx_core::bus::{Bus, BusRead, BusWrite};
use support::asm;

const JOY_DATA: u32 = 0x1F80_1040;
const JOY_STAT: u32 = 0x1F80_1044;
const JOY_CTRL: u32 = 0x1F80_104A;
const I_STAT: u32 = 0x1F80_1070;
const CTRL_BIOS: u16 = 0x1003;

// § Controller and Memory Card Signals (L386) de docs/reference/10-controllers-memcards.md:
// o driver do kernel IGNORA pulsos de /ACK nos primeiros 2-3 us (100 ciclos) depois do SCK,
// e (L385) desiste se o /ACK nao chegar em 100 us — 3386 ciclos a 33,8688 MHz.
const JANELA_IGNORADA: u32 = 100;
const TIMEOUT_DO_KERNEL: u32 = 3386;

fn porta(pad: bool) -> Bus {
    let mut bus = asm::bus_with_bios_empty();
    bus.sio_mut().connect_digital_pad(pad);
    bus.write16::<BusWrite>(JOY_CTRL, CTRL_BIOS);
    bus
}

fn stat(bus: &Bus) -> u16 {
    bus.read16::<BusRead>(JOY_STAT)
}

fn istat_bit7(bus: &Bus) -> u32 {
    bus.read32::<BusRead>(I_STAT) & 0x80
}

#[test]
fn ack_nao_chega_no_mesmo_ciclo_do_byte() {
    let mut bus = porta(true);

    bus.write8::<BusWrite>(JOY_DATA, 0x01);

    assert_eq!(
        stat(&bus) & 0x0200,
        0,
        "JOY_STAT.9 no mesmo ciclo do byte e o defeito que a spec manda emuladores nao terem"
    );
    assert_eq!(istat_bit7(&bus), 0, "IRQ7 tambem nao pode subir no ato");
}

#[test]
fn ack_nao_chega_dentro_da_janela_ignorada_pelo_kernel() {
    let mut bus = porta(true);

    bus.write8::<BusWrite>(JOY_DATA, 0x01);
    bus.tick_timers(JANELA_IGNORADA);

    assert_eq!(
        stat(&bus) & 0x0200,
        0,
        "pulso dentro dos primeiros 100 ciclos e ignorado pelo driver: nao pode ser o nosso"
    );
}

#[test]
fn ack_chega_antes_do_timeout_do_kernel() {
    let mut bus = porta(true);

    bus.write8::<BusWrite>(JOY_DATA, 0x01);
    bus.tick_timers(TIMEOUT_DO_KERNEL);

    assert_ne!(
        stat(&bus) & 0x0200,
        0,
        "o /ACK tem de chegar antes dos 100 us em que o driver do kernel desiste"
    );
    assert_ne!(istat_bit7(&bus), 0, "e o IRQ7 correspondente tem de subir");
}

#[test]
fn sem_periferico_nenhum_ack_e_agendado() {
    let mut bus = porta(false);

    bus.write8::<BusWrite>(JOY_DATA, 0x01);
    bus.tick_timers(TIMEOUT_DO_KERNEL);

    assert_eq!(
        stat(&bus) & 0x0200,
        0,
        "slot vazio nao pulsa /ACK nem depois do timeout"
    );
    assert_eq!(istat_bit7(&bus), 0);
}

#[test]
fn dois_bytes_geram_dois_acks() {
    let mut bus = porta(true);

    bus.write8::<BusWrite>(JOY_DATA, 0x01);
    bus.tick_timers(TIMEOUT_DO_KERNEL);
    assert_ne!(stat(&bus) & 0x0200, 0, "pre-condicao: primeiro /ACK chegou");

    bus.write16::<BusWrite>(JOY_CTRL, CTRL_BIOS | 0x0010);
    bus.read8::<BusRead>(JOY_DATA);
    bus.write32::<BusWrite>(I_STAT, !0x80);
    assert_eq!(stat(&bus) & 0x0200, 0, "pre-condicao: IRQ reconhecida");

    bus.write8::<BusWrite>(JOY_DATA, 0x42);
    bus.tick_timers(TIMEOUT_DO_KERNEL);

    assert_ne!(
        stat(&bus) & 0x0200,
        0,
        "cada byte reconhecido pelo periferico gera o seu proprio pulso de /ACK"
    );
}

#[test]
fn soltar_o_cs_cancela_o_ack_ainda_nao_entregue() {
    let mut bus = porta(true);

    bus.write8::<BusWrite>(JOY_DATA, 0x01);
    bus.write16::<BusWrite>(JOY_CTRL, 0x0000);
    bus.tick_timers(TIMEOUT_DO_KERNEL);

    assert_eq!(
        stat(&bus) & 0x0200,
        0,
        "com /CS solto a transferencia acabou; o pulso pendente nao pode chegar depois"
    );
}
