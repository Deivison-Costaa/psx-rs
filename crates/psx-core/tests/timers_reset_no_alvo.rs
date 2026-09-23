mod support;

use psx_core::bus::{Bus, BusRead, BusWrite};
use support::asm;

const T2_CNT: u32 = 0x1F80_1120;
const T2_MODE: u32 = 0x1F80_1124;
const T2_TARGET: u32 = 0x1F80_1128;

fn histograma(bus: &mut Bus, ciclos: usize) -> [u32; 12] {
    let mut vistos = [0u32; 12];
    for _ in 0..ciclos {
        bus.timers_mut().tick(T2_CNT, 1, false, false);
        let valor = bus.read32::<BusRead>(T2_CNT) as usize & 0xFFFF;
        vistos[valor.min(11)] += 1;
    }
    vistos
}

#[test]
fn alvo_de_10_da_periodo_de_12_ciclos_com_zero_duas_vezes() {
    let mut bus = asm::bus_with_bios_empty();
    bus.write32::<BusWrite>(T2_TARGET, 10);
    bus.write32::<BusWrite>(T2_MODE, 0x0008);
    histograma(&mut bus, 1);

    let vistos = histograma(&mut bus, 12 * 100);

    assert_eq!(
        vistos[0], 200,
        "timers.exe em hardware: o 0 aparece o dobro (1667 de 10000)"
    );
    for (valor, vezes) in vistos.iter().enumerate().take(11).skip(1) {
        assert_eq!(*vezes, 100, "valor {valor} aparece 1 vez por periodo");
    }
    assert_eq!(vistos[11], 0, "alvo+1 nunca e alcancado");
}

#[test]
fn contador_mostra_o_alvo_antes_de_voltar_a_zero() {
    let mut bus = asm::bus_with_bios_empty();
    bus.write32::<BusWrite>(T2_TARGET, 1);
    bus.write32::<BusWrite>(T2_MODE, 0x0008);
    let mut seq = Vec::new();
    for _ in 0..6 {
        seq.push(bus.read32::<BusRead>(T2_CNT) & 0xFFFF);
        bus.timers_mut().tick(T2_CNT, 1, false, false);
    }
    assert_eq!(
        seq,
        vec![0, 0, 1, 0, 0, 1],
        "exemplo de 05-timers.md (Reset and Wrap) com alvo 1 e bit3 ligado"
    );
}

#[test]
fn clock_do_sistema_dividido_por_8_mantem_periodo_de_alvo_ticks() {
    let mut bus = asm::bus_with_bios_empty();
    bus.write32::<BusWrite>(T2_TARGET, 4);
    bus.write32::<BusWrite>(T2_MODE, 0x0208);
    bus.timers_mut().tick(T2_CNT, 8 * 4, false, false);
    assert_eq!(bus.read32::<BusRead>(T2_CNT) & 0xFFFF, 4);
    bus.timers_mut().tick(T2_CNT, 8 * 4, false, false);
    assert_eq!(
        bus.read32::<BusRead>(T2_CNT) & 0xFFFF,
        4,
        "com fonte lenta o reset nao engole tick: periodo = alvo"
    );
}
