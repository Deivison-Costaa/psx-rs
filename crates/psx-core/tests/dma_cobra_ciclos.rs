mod support;

use psx_core::bus::{Bus, BusRead};
use support::asm;

// Degrau 9 da escada de timing de CPU/barramento (achado 0193.4): o custo de
// Dma::transfer_cost (Degrau 8) precisa ser cobrado de verdade em Bus::tick_timers, nao so
// calculado. O acumulador (`dma_extra_cycles`, espelhando `Cpu::extra_cycles`) fica pendente
// entre a execucao sincrona do DMA (dentro do write32 que dispara o canal) e o proximo
// tick_timers -- por isso os testes disparam o DMA e so entao chamam `tick_timers(1)` a
// mao, sem precisar de uma CPU inteira.

const D4_MADR: u32 = 0x1F80_10C0;
const D4_BCR: u32 = 0x1F80_10C4;
const D4_CHCR: u32 = 0x1F80_10C8;
const D6_MADR: u32 = 0x1F80_10E0;
const D6_BCR: u32 = 0x1F80_10E4;
const D6_CHCR: u32 = 0x1F80_10E8;
const DPCR: u32 = 0x1F80_10F0;

fn bus_com_dma() -> Bus {
    asm::bus_with_bios_empty()
}

#[test]
fn otc_de_16_palavras_cobra_17_ciclos_extras() {
    let mut bus = bus_com_dma();
    bus.write32::<BusRead>(D6_MADR, 0x0000_1000);
    bus.write32::<BusRead>(D6_BCR, 16);
    bus.write32::<BusRead>(DPCR, 0x0765_4321 | (1 << 27));
    let antes = bus.total_cycles();
    bus.write32::<BusRead>(D6_CHCR, 0x1100_0002);
    bus.tick_timers(1);
    assert_eq!(
        bus.total_cycles() - antes,
        1 + 17,
        "1 ciclo nominal do tick + 17 do OTC (16 palavras a 17/16, Degrau 8)"
    );
}

#[test]
fn spu_de_8_palavras_cobra_33_ciclos_extras() {
    let mut bus = bus_com_dma();
    bus.write32::<BusRead>(D4_MADR, 0x0000_2000);
    bus.write32::<BusRead>(D4_BCR, 8);
    bus.write32::<BusRead>(DPCR, 0x0765_4321 | (1 << 19));
    let antes = bus.total_cycles();
    bus.write32::<BusRead>(D4_CHCR, 0x0100_0001);
    bus.tick_timers(1);
    // § DRAM Hyper Page mode (04-dma.md L238-243) x § DMA Transfer Rates (L217-226): os 33
    // ciclos das 8 palavras do SPU sao a vazao do DISPOSITIVO e definem QUANDO o canal
    // conclui; o que trava a CPU e so o lado da RAM (17 clks por 16 palavras).
    assert_eq!(
        bus.total_cycles() - antes,
        1 + 8,
        "1 ciclo nominal do tick + 8 do lado da RAM (8 palavras a 17/16)"
    );
    assert_ne!(
        bus.read32::<BusRead>(D4_CHCR) & (1 << 24),
        0,
        "aos 9 ciclos o canal ainda nao chegou aos 33 da taxa do SPU"
    );
    bus.tick_timers(33);
    assert_eq!(
        bus.read32::<BusRead>(D4_CHCR) & (1 << 24),
        0,
        "passados os 33 ciclos das 8 palavras do SPU o canal concluiu"
    );
}

#[test]
fn canal_desabilitado_no_dpcr_nao_cobra_nada() {
    let mut bus = bus_com_dma();
    bus.write32::<BusRead>(D6_MADR, 0x0000_1000);
    bus.write32::<BusRead>(D6_BCR, 16);
    // DPCR no valor de reset (0x0765_4321) nao habilita o canal 6.
    let antes = bus.total_cycles();
    bus.write32::<BusRead>(D6_CHCR, 0x1100_0002);
    bus.tick_timers(1);
    assert_eq!(
        bus.total_cycles() - antes,
        1,
        "sem habilitar no DPCR, o OTC nao executa e nao cobra nada"
    );
}

#[test]
fn custo_do_dma_e_drenado_no_tick_e_nao_se_repete_no_seguinte() {
    let mut bus = bus_com_dma();
    bus.write32::<BusRead>(D6_MADR, 0x0000_1000);
    bus.write32::<BusRead>(D6_BCR, 16);
    bus.write32::<BusRead>(DPCR, 0x0765_4321 | (1 << 27));
    let antes = bus.total_cycles();
    bus.write32::<BusRead>(D6_CHCR, 0x1100_0002);
    bus.tick_timers(1); // drena os 17 do OTC
    bus.tick_timers(1); // so o ciclo nominal, sem cobrar de novo
    assert_eq!(
        bus.total_cycles() - antes,
        1 + 17 + 1,
        "o segundo tick nao deveria repetir o custo do primeiro DMA"
    );
}

#[test]
fn dois_dmas_disparados_antes_do_tick_acumulam_o_custo_de_cada_um() {
    let mut bus = bus_com_dma();
    bus.write32::<BusRead>(D6_MADR, 0x0000_1000);
    bus.write32::<BusRead>(D6_BCR, 16);
    bus.write32::<BusRead>(D4_MADR, 0x0000_2000);
    bus.write32::<BusRead>(D4_BCR, 8);
    bus.write32::<BusRead>(DPCR, 0x0765_4321 | (1 << 27) | (1 << 19));
    let antes = bus.total_cycles();
    bus.write32::<BusRead>(D6_CHCR, 0x1100_0002); // +17 (16 palavras)
    bus.write32::<BusRead>(D4_CHCR, 0x0100_0001); // +8  (8 palavras)
    bus.tick_timers(1);
    assert_eq!(
        bus.total_cycles() - antes,
        1 + 17 + 8,
        "os dois custos se acumulam ate o proximo tick_timers, nao se sobrescrevem"
    );
}
