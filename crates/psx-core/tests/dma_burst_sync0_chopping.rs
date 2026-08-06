mod support;

use psx_core::bus::{Bus, BusRead};
use support::asm;

// Achado legado 10.109 (04-dma.md L48-50): "In SyncMode=0, the hardware doesn't
// update the MADR registers ... unless Chopping is enabled, in that case it
// does update MADR" e (L80-81) "SyncMode=0 with chopping enabled decrements BC
// to zero". execute_burst usa uma variavel local pra endereco corrente e nunca
// escreve de volta em madr[2]/bcr[2], entao hoje MADR/BC ficam congelados
// mesmo com o bit de chopping (CHCR.8) ligado.

fn bus_com_dma() -> Bus {
    asm::bus_with_bios_empty()
}

const D2_MADR: u32 = 0x1F80_10A0;
const D2_BCR: u32 = 0x1F80_10A4;
const D2_CHCR: u32 = 0x1F80_10A8;
const DPCR: u32 = 0x1F80_10F0;
const GP0: u32 = 0x1F80_1810;

fn write_ram32(bus: &mut Bus, addr: u32, val: u32) {
    bus.write32::<BusRead>(addr, val);
}

fn habilitar_canal2(bus: &mut Bus) {
    let dpcr = bus.read32::<BusRead>(DPCR);
    bus.write32::<BusRead>(DPCR, dpcr | (1 << 11));
}

fn abrir_janela_cpu_para_vram(bus: &mut Bus) {
    bus.write32::<BusRead>(GP0, 0xA000_0000);
    bus.write32::<BusRead>(GP0, 0x0000_0000);
    bus.write32::<BusRead>(GP0, 0x0002_0001);
}

#[test]
fn dma2_burst_sync_mode0_com_chopping_atualiza_madr_e_zera_bc() {
    let mut bus = bus_com_dma();
    habilitar_canal2(&mut bus);
    abrir_janela_cpu_para_vram(&mut bus);

    let data_addr: u32 = 0x0000_0100;
    write_ram32(&mut bus, data_addr, 0xABAB_CDCD);
    write_ram32(&mut bus, data_addr + 4, 0xEFEF_1212);

    bus.write32::<BusRead>(D2_MADR, data_addr);
    bus.write32::<BusRead>(D2_BCR, 2);
    // SyncMode=0 (bits 9-10=00), direcao RAM->dispositivo (bit0), chopping
    // ligado (bit8), start (bit24) e forcar sem DREQ (bit28).
    bus.write32::<BusRead>(D2_CHCR, 0x1100_0101);

    let chcr = bus.read32::<BusRead>(D2_CHCR);
    assert_eq!(
        chcr & (1 << 24),
        0,
        "transferencia com chopping tambem tem que completar e limpar bit24"
    );

    let madr = bus.read32::<BusRead>(D2_MADR);
    assert_eq!(
        madr,
        data_addr + 8,
        "spec § D#_MADR (04-dma.md L48-50): com chopping ligado, MADR tem que \
         avancar ate o fim da transferencia (2 palavras, +8), nao ficar \
         congelado no endereco inicial"
    );

    let bcr = bus.read32::<BusRead>(D2_BCR);
    assert_eq!(
        bcr & 0xFFFF,
        0,
        "spec § D#_BCR (04-dma.md L80-81): 'SyncMode=0 with chopping enabled \
         decrements BC to zero' — o campo BC (bits 0-15) tem que terminar \
         zerado, nao nos 2 originais"
    );
}
