mod support;

use psx_core::bus::{Bus, BusRead};
use support::asm;

// dma/chopping (ps1-tests, tests/exes/ps1-tests/dma/chopping/psx.log): a suite
// cobre DMA2 em SyncMode=0 (Burst), inclusive "no chopping". Antes desta
// correcao, try_execute_dma2 nao tratava sync_mode=0 (so 1 e 2), entao o bit24
// (start/busy) nunca era limpo e a suite travava para sempre no primeiro caso
// SyncMode=0 — nao chegava a rodar as 131 linhas seguintes. O numero de "CPU
// cycles" continua divergindo (item novo no ROADMAP: nao ha custo de ciclo
// modelado para chopping); esta correcao so garante que a transferencia
// termina, como a spec exige (04-dma.md L84-119).

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
    bus.write32::<BusRead>(GP0, 0x0001_0001);
}

#[test]
fn dma2_burst_sync_mode0_completa_em_vez_de_travar() {
    let mut bus = bus_com_dma();
    habilitar_canal2(&mut bus);
    abrir_janela_cpu_para_vram(&mut bus);

    let data_addr: u32 = 0x0000_0100;
    write_ram32(&mut bus, data_addr, 0xABAB_CDCD);

    bus.write32::<BusRead>(D2_MADR, data_addr);
    bus.write32::<BusRead>(D2_BCR, 1);
    // SyncMode=0 (bits 9-10 = 00), direcao RAM->dispositivo (bit0), start
    // (bit24) e forcar sem DREQ (bit28), como manda a spec para SyncMode=0.
    bus.write32::<BusRead>(D2_CHCR, 0x1100_0001);

    let chcr = bus.read32::<BusRead>(D2_CHCR);
    assert_eq!(
        chcr & (1 << 24),
        0,
        "SyncMode=0 (Burst) deve completar e limpar bit24, nao travar para sempre"
    );

    let pixel = bus.gpu().vram_pixel(0, 0);
    assert_eq!(
        pixel, 0xCDCD,
        "VRAM (0,0) deve conter a metade inferior de ABABCDCD apos o burst"
    );

    let madr = bus.read32::<BusRead>(D2_MADR);
    assert_eq!(
        madr, data_addr,
        "sem chopping, SyncMode=0 nao atualiza MADR (spec citada no doc da iteracao)"
    );
}
