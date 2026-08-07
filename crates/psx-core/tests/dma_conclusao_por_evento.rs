mod support;

use psx_core::bus::{Bus, BusRead};
use psx_core::dma::Dma;
use support::asm;

// § "Bit 24 is automatically cleared upon COMPLETION of the transfer" (04-dma.md L115-116)
// + § CPU Operation during DMA (04-dma.md L245-252). Detalhe e citacao em
// docs/iterations/.

const D4_MADR: u32 = 0x1F80_10C0;
const D4_BCR: u32 = 0x1F80_10C4;
const D4_CHCR: u32 = 0x1F80_10C8;
const D6_MADR: u32 = 0x1F80_10E0;
const D6_BCR: u32 = 0x1F80_10E4;
const D6_CHCR: u32 = 0x1F80_10E8;
const DPCR: u32 = 0x1F80_10F0;
const DICR: u32 = 0x1F80_10F4;

const BIT24: u32 = 1 << 24;

fn bus_com_spu_armado(palavras: u32) -> Bus {
    let mut bus = asm::bus_with_bios_empty();
    bus.write32::<BusRead>(D4_MADR, 0x0000_2000);
    bus.write32::<BusRead>(D4_BCR, palavras);
    bus.write32::<BusRead>(DPCR, 0x0765_4321 | (1 << 19));
    bus
}

#[test]
fn canal_do_spu_continua_ocupado_logo_apos_a_escrita_do_chcr() {
    let mut bus = bus_com_spu_armado(256);
    bus.write32::<BusRead>(D4_CHCR, 0x0100_0001);
    assert_ne!(
        bus.read32::<BusRead>(D4_CHCR) & BIT24,
        0,
        "logo apos o start o canal ainda esta transferindo: bit24 so cai na CONCLUSAO"
    );
}

#[test]
fn bit24_do_spu_cai_quando_o_prazo_da_taxa_do_canal_vence() {
    let mut bus = bus_com_spu_armado(256);
    bus.write32::<BusRead>(D4_CHCR, 0x0100_0001);
    let prazo = Dma::transfer_cost(4, 256);
    let mut andados = 0u64;
    while andados < prazo {
        bus.tick_timers(1);
        andados += 1;
    }
    bus.tick_timers(1);
    assert_eq!(
        bus.read32::<BusRead>(D4_CHCR) & BIT24,
        0,
        "passado o custo da taxa do canal (04-dma.md L217-226) o bit24 tem de estar limpo"
    );
}

#[test]
fn a_cpu_avanca_ciclos_antes_do_canal_concluir() {
    let mut bus = bus_com_spu_armado(256);
    let antes = bus.total_cycles();
    bus.write32::<BusRead>(D4_CHCR, 0x0100_0001);
    bus.tick_timers(1);
    let gastos = bus.total_cycles() - antes;
    assert!(
        gastos < Dma::transfer_cost(4, 256),
        "o stall da CPU e o do DRAM Hyper Page (04-dma.md L238-243), menor que a taxa do \
         dispositivo: gastou {gastos} de {}",
        Dma::transfer_cost(4, 256)
    );
    assert_ne!(
        bus.read32::<BusRead>(D4_CHCR) & BIT24,
        0,
        "com a CPU tendo rodado menos que o prazo, o canal segue ocupado"
    );
}

#[test]
fn flag_de_conclusao_no_dicr_so_sobe_junto_com_a_queda_do_bit24() {
    let mut bus = bus_com_spu_armado(256);
    bus.write32::<BusRead>(DICR, (1 << 23) | (1 << (16 + 4)));
    bus.write32::<BusRead>(D4_CHCR, 0x0100_0001);
    assert_eq!(
        bus.read32::<BusRead>(DICR) & (1 << (24 + 4)),
        0,
        "canal ainda ocupado nao pode ter marcado o flag de fim de transferencia"
    );
    let prazo = Dma::transfer_cost(4, 256);
    for _ in 0..=prazo {
        bus.tick_timers(1);
    }
    assert_ne!(
        bus.read32::<BusRead>(DICR) & (1 << (24 + 4)),
        0,
        "concluida a transferencia, o flag do canal 4 sobe"
    );
}

#[test]
fn otc_conclui_dentro_da_escrita_porque_a_cpu_fica_parada_a_transferencia_inteira() {
    let mut bus = asm::bus_with_bios_empty();
    bus.write32::<BusRead>(D6_MADR, 0x0000_1000);
    bus.write32::<BusRead>(D6_BCR, 4096);
    bus.write32::<BusRead>(DPCR, 0x0765_4321 | (1 << 27));
    bus.write32::<BusRead>(D6_CHCR, 0x1100_0002);
    assert_eq!(
        Dma::transfer_cost(6, 4096),
        Dma::stall_cost(4096),
        "OTC nao tem espera de dispositivo: a taxa do canal e a propria taxa da RAM"
    );
    assert_eq!(
        bus.read32::<BusRead>(D6_CHCR) & BIT24,
        0,
        "sem espera de dispositivo a CPU fica travada a transferencia toda (04-dma.md L245-252): \
         nao existe instante em que ela leia o canal ocupado"
    );
}

#[test]
fn segundo_gatilho_no_dpcr_nao_reexecuta_canal_ainda_ocupado() {
    let mut bus = bus_com_spu_armado(256);
    bus.write32::<BusRead>(D4_CHCR, 0x0100_0001);
    let madr_apos_start = bus.read32::<BusRead>(D4_MADR);
    bus.write32::<BusRead>(DPCR, 0x0765_4321 | (1 << 19));
    assert_eq!(
        bus.read32::<BusRead>(D4_MADR),
        madr_apos_start,
        "canal ocupado nao pode ser disparado de novo pela reescrita do DPCR"
    );
}
