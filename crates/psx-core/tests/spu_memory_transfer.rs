mod support;

use psx_core::bus::{Bus, BusRead};
use support::asm;

// § SPU Control/Status/Memory Access (docs/reference/08-spu.md L659-773).
const SPU_TRANSFER_ADDR: u32 = 0x1F80_1DA6;
const SPU_FIFO: u32 = 0x1F80_1DA8;
const SPU_CNT: u32 = 0x1F80_1DAA;
const SPU_STAT: u32 = 0x1F80_1DAE;
const SPU_DTC: u32 = 0x1F80_1DAC;
const D4_MADR: u32 = 0x1F80_10C0;
const D4_BCR: u32 = 0x1F80_10C4;
const D4_CHCR: u32 = 0x1F80_10C8;
const DPCR: u32 = 0x1F80_10F0;

// § Commonly used DMA Control Register values (docs/reference/04-dma.md L159-168).
const CHCR_SPU_WRITE: u32 = 0x0100_0201;
const CHCR_SPU_READ: u32 = 0x0100_0200;

const TEST_STRING: &[u8] = b"Hello there   :)";

fn bus_com_spu() -> Bus {
    asm::bus_with_bios_empty()
}

fn set_transfer_mode(bus: &mut Bus, mode: u16) {
    let mut cnt = bus.read16::<BusRead>(SPU_CNT);
    cnt &= !(0b11 << 4);
    cnt |= mode << 4;
    bus.write16::<BusRead>(SPU_CNT, cnt);
}

fn set_start_address(bus: &mut Bus, addr: u32) {
    bus.write16::<BusRead>(SPU_TRANSFER_ADDR, (addr / 8) as u16);
}

// § SPU RAM Manual Write (docs/reference/08-spu.md L741-753).
fn manual_write(bus: &mut Bus, addr: u32, texto: &[u8]) {
    bus.write16::<BusRead>(SPU_DTC, 2 << 1);
    set_transfer_mode(bus, 0);
    set_start_address(bus, addr);
    for par in texto.chunks(2) {
        let lo = par[0] as u16;
        let hi = *par.get(1).unwrap_or(&0) as u16;
        bus.write16::<BusRead>(SPU_FIFO, lo | (hi << 8));
    }
    set_transfer_mode(bus, 1);
}

// § SPU RAM DMA-Write/-Read (docs/reference/08-spu.md L755-773).
fn dma_transfer(bus: &mut Bus, addr: u32, ram_addr: u32, tamanho_bytes: usize, escrita: bool) {
    bus.write16::<BusRead>(SPU_DTC, 2 << 1);
    set_transfer_mode(bus, 0);
    set_start_address(bus, addr);
    set_transfer_mode(bus, if escrita { 2 } else { 3 });

    let bs: u32 = 1;
    let ba = (tamanho_bytes as u32) / (4 * bs);
    bus.write32::<BusRead>(D4_MADR, ram_addr);
    bus.write32::<BusRead>(D4_BCR, bs | (ba << 16));
    bus.write32::<BusRead>(DPCR, 0x0765_4321 | (1 << 19));
    bus.write32::<BusRead>(
        D4_CHCR,
        if escrita {
            CHCR_SPU_WRITE
        } else {
            CHCR_SPU_READ
        },
    );
    espera_conclusao(bus, ba * bs);
}

// O canal 4 espera o SPU (33/8 clks/word contra 17/16 do lado da RAM), entao o bit24 so cai
// no evento de conclusao — a CPU real gasta esse tempo no laco que enquete o CHCR.
fn espera_conclusao(bus: &mut Bus, palavras: u32) {
    bus.tick_timers(psx_core::dma::Dma::transfer_cost(4, palavras) as u32 + 1);
}

fn write_ram_bytes(bus: &mut Bus, addr: u32, dados: &[u8]) {
    for (i, &b) in dados.iter().enumerate() {
        bus.write8::<BusRead>(addr + i as u32, b);
    }
}

fn read_ram_bytes(bus: &Bus, addr: u32, len: usize) -> Vec<u8> {
    (0..len)
        .map(|i| bus.read8::<BusRead>(addr + i as u32))
        .collect()
}

#[test]
fn spu_dtc_registrador_e_gravavel_e_legivel() {
    let mut bus = bus_com_spu();
    for &v in &[0x0000u16, 0xFFFF, 0x5555, 0xAAAA] {
        bus.write16::<BusRead>(SPU_DTC, v);
        assert_eq!(bus.read16::<BusRead>(SPU_DTC), v);
    }
}

#[test]
fn spu_cnt_bits_0_a_5_sao_copiados_para_stat() {
    let mut bus = bus_com_spu();
    for &cnt in &[0x3fu16, 0x00, 0x15, 0x2a] {
        bus.write16::<BusRead>(SPU_CNT, cnt);
        assert_eq!(bus.read16::<BusRead>(SPU_STAT) & 0x3f, cnt & 0x3f);
    }
}

#[test]
fn spu_escrita_manual_fica_visivel_na_ram_via_dma_read() {
    let mut bus = bus_com_spu();
    manual_write(&mut bus, 0x1000, TEST_STRING);

    let buf_addr: u32 = 0x0000_2000;
    write_ram_bytes(&mut bus, buf_addr, &[0xCC; 32]);
    dma_transfer(&mut bus, 0x1000, buf_addr, TEST_STRING.len(), false);

    assert_eq!(
        bus.read32::<BusRead>(D4_CHCR) & (1 << 24),
        0,
        "DMA de leitura deve completar (16 bytes cabem em 4 blocos de 1 palavra)"
    );
    let lido = read_ram_bytes(&bus, buf_addr, TEST_STRING.len());
    assert_eq!(lido, TEST_STRING);
}

#[test]
fn spu_dma_write_depois_dma_read_faz_round_trip() {
    let mut bus = bus_com_spu();

    let src_addr: u32 = 0x0000_3000;
    write_ram_bytes(&mut bus, src_addr, TEST_STRING);
    dma_transfer(&mut bus, 0x2000, src_addr, TEST_STRING.len(), true);
    assert_eq!(bus.read32::<BusRead>(D4_CHCR) & (1 << 24), 0);

    let dst_addr: u32 = 0x0000_4000;
    write_ram_bytes(&mut bus, dst_addr, &[0xCC; 32]);
    dma_transfer(&mut bus, 0x2000, dst_addr, TEST_STRING.len(), false);
    assert_eq!(bus.read32::<BusRead>(D4_CHCR) & (1 << 24), 0);

    let lido = read_ram_bytes(&bus, dst_addr, TEST_STRING.len());
    assert_eq!(lido, TEST_STRING);
}

// § D#_BCR SyncMode=0 (docs/reference/04-dma.md L37-40): "0-15 BC Number of
// words" — um unico campo, sem BS*BA. testDMAWriteToRamSyncMode0 do ps1-tests
// usa BCR::mode0(4) (bits16-31 = 0), que uma leitura do BCR como par (bs,ba)
// interpretaria como bs=4, ba=0 -> 0x10000 (spec L76), pedindo 65536x mais
// palavras do que as 4 (16 bytes) reais e estourando a RAM do SPU.
#[test]
fn spu_dma_sync_mode0_transfere_so_o_numero_de_palavras_do_campo_unico() {
    let mut bus = bus_com_spu();
    let sentinela = b"SENTINELA-INTACT";
    manual_write(&mut bus, 0x0100, sentinela);

    let src_addr: u32 = 0x0000_5000;
    write_ram_bytes(&mut bus, src_addr, TEST_STRING);

    bus.write16::<BusRead>(SPU_DTC, 2 << 1);
    set_transfer_mode(&mut bus, 0);
    set_start_address(&mut bus, 0x2200);
    set_transfer_mode(&mut bus, 2); // DMAWrite

    bus.write32::<BusRead>(D4_MADR, src_addr);
    bus.write32::<BusRead>(D4_BCR, 4); // SyncMode0: BC=4 palavras, bits16-31=0
    bus.write32::<BusRead>(DPCR, 0x0765_4321 | (1 << 19));
    // CHCR::SPUwrite(startImmediately): dir=fromRam(bit0), enabled(bit24), syncMode=0.
    bus.write32::<BusRead>(D4_CHCR, 0x0100_0001);
    espera_conclusao(&mut bus, 4);

    assert_eq!(
        bus.read32::<BusRead>(D4_CHCR) & (1 << 24),
        0,
        "4 palavras (16 bytes) devem completar dentro do prazo da taxa do canal"
    );

    let buf_addr: u32 = 0x0000_6000;
    write_ram_bytes(&mut bus, buf_addr, &[0xCC; 32]);
    dma_transfer(&mut bus, 0x0100, buf_addr, sentinela.len(), false);
    assert_eq!(
        read_ram_bytes(&bus, buf_addr, sentinela.len()),
        sentinela,
        "0x100 nao pode ser tocado por uma transferencia de so 4 palavras em 0x2200"
    );
}
