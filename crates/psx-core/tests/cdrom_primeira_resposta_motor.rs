mod support;

use psx_core::bus::{Bus, BusRead, BusWrite};
use support::asm;

const CD_BASE: u32 = 0x1F80_1800;
const I_STAT: u32 = 0x1F80_1070;
const IRQ2: u32 = 1 << 2;

// Achado legado 10.55 (06-cdrom.md L2047-2054): a primeira resposta demora menos se o
// motor ja estiver parado quando o comando chega — a tabela de timings da spec da dois
// valores pra Nop: "normal" (motor ligado) e "when stopped" (motor desligado).
const PRIMEIRA_RESPOSTA_NORMAL: u64 = 0xC4E1;
const PRIMEIRA_RESPOSTA_PARADO: u64 = 0x5CF4;

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

fn manda_comando(bus: &mut Bus, cmd: u8) {
    set_bank(bus, 0);
    cd_write(bus, 1, cmd);
}

fn hintsts(bus: &mut Bus) -> u8 {
    set_bank(bus, 1);
    let val = cd_read(bus, 3) & 0x7;
    set_bank(bus, 0);
    val
}

fn intmsk_write(bus: &mut Bus, val: u8) {
    set_bank(bus, 1);
    cd_write(bus, 2, val);
    set_bank(bus, 0);
}

fn i_stat_irq2(bus: &Bus) -> u32 {
    bus.read32::<BusRead>(I_STAT) & IRQ2
}

fn avanca(bus: &mut Bus, ciclos: u64) {
    bus.tick_timers(ciclos as u32);
}

#[test]
fn primeira_resposta_com_motor_parado_usa_o_atraso_menor() {
    let mut bus = bus();
    intmsk_write(&mut bus, 0x1F);
    // motor comeca desligado por padrao em Cdrom::new() — nao chamo insert_disc() aqui.

    manda_comando(&mut bus, 0x01);
    avanca(&mut bus, PRIMEIRA_RESPOSTA_PARADO - 1);
    assert_eq!(
        hintsts(&mut bus),
        0,
        "spec § First Response (L2054): 'Nop (when stopped)' = 0005cf4h — um ciclo antes \
         ainda e cedo"
    );

    avanca(&mut bus, 1);
    assert_eq!(
        i_stat_irq2(&bus),
        IRQ2,
        "com HINTMSK=1Fh a propria entrega levanta IRQ2"
    );
    assert_eq!(
        hintsts(&mut bus),
        3,
        "com o motor parado, a 1a resposta tem que chegar em 0005cf4h ciclos, nao nos \
         000c4e1h do motor ligado (achado 10.55: hoje o motor e ignorado)"
    );
}

#[test]
fn primeira_resposta_com_motor_ligado_usa_o_atraso_normal() {
    let mut bus = bus();
    intmsk_write(&mut bus, 0x1F);
    bus.cdrom_mut().insert_disc(); // liga o motor (e insere disco)

    manda_comando(&mut bus, 0x01);
    avanca(&mut bus, PRIMEIRA_RESPOSTA_NORMAL - 1);
    assert_eq!(
        hintsts(&mut bus),
        0,
        "spec § First Response (L2053): 'Nop (normal)' = 000c4e1h — um ciclo antes ainda e \
         cedo"
    );

    avanca(&mut bus, 1);
    assert_eq!(
        hintsts(&mut bus),
        3,
        "com o motor ligado, a 1a resposta continua chegando em 000c4e1h ciclos"
    );
}
