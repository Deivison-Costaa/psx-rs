mod support;

use psx_core::bus::{Bus, BusRead, BusWrite};
use support::asm;

const D2_MADR: u32 = 0x1F80_10A0;
const D2_BCR: u32 = 0x1F80_10A4;
const D2_CHCR: u32 = 0x1F80_10A8;
const DPCR: u32 = 0x1F80_10F0;
const GP0: u32 = 0x1F80_1810;
const GPUSTAT: u32 = 0x1F80_1814;

// CHCR do StoreImage do kernel, medido no boot real: sync=1 (slice) e bit 0 = 0 (device -> RAM).
const CHCR_PARA_RAM: u32 = 0x0100_0200;
const CHCR_DA_RAM: u32 = 0x0100_0201;

const X: u32 = 16;
const Y: u32 = 8;
const W: u32 = 4;
const H: u32 = 2;
const PIXELS: [u32; 4] = [0x2222_1111, 0x4444_3333, 0x6666_5555, 0x8888_7777];
const DESTINO: u32 = 0x0000_2000;

fn bus() -> Bus {
    let mut bus = asm::bus_with_bios_empty();
    bus.write32::<BusWrite>(DPCR, 0x0765_4321 | (1 << 11));
    bus
}

fn gp0(bus: &mut Bus, val: u32) {
    bus.write32::<BusWrite>(GP0, val);
}

// Semeia a janela de VRAM por A0h (CPU->VRAM), o caminho que ja existia antes desta iteracao.
fn semeia_vram(bus: &mut Bus) {
    gp0(bus, 0xA000_0000);
    gp0(bus, (Y << 16) | X);
    gp0(bus, (H << 16) | W);
    for p in PIXELS {
        gp0(bus, p);
    }
}

fn pede_c0h(bus: &mut Bus) {
    gp0(bus, 0xC000_0000);
    gp0(bus, (Y << 16) | X);
    gp0(bus, (H << 16) | W);
}

fn dispara_dma(bus: &mut Bus, chcr: u32, palavras: u32) {
    bus.write32::<BusWrite>(D2_MADR, DESTINO);
    bus.write32::<BusWrite>(D2_BCR, (1 << 16) | palavras);
    bus.write32::<BusWrite>(D2_CHCR, chcr);
}

fn ram32(bus: &Bus, addr: u32) -> u32 {
    bus.read32::<BusRead>(addr)
}

fn gpustat(bus: &Bus) -> u32 {
    bus.read32::<BusRead>(GPUSTAT)
}

#[test]
fn dma2_para_ram_drena_a_janela_pedida_pelo_c0h() {
    let mut bus = bus();
    semeia_vram(&mut bus);
    pede_c0h(&mut bus);

    dispara_dma(&mut bus, CHCR_PARA_RAM, 4);

    for (i, esperado) in PIXELS.iter().enumerate() {
        assert_eq!(
            ram32(&bus, DESTINO + (i as u32) * 4),
            *esperado,
            "com o bit 0 do CHCR em 0 a transferencia e device -> RAM: a palavra {i} da janela \
             lida por C0h tem de aparecer na RAM apontada pelo MADR"
        );
    }
}

#[test]
fn dma2_para_ram_devolve_o_gpustat_26_ao_terminar() {
    let mut bus = bus();
    semeia_vram(&mut bus);
    pede_c0h(&mut bus);
    assert_eq!(
        gpustat(&bus) & (1 << 26),
        0,
        "pre-condicao: o C0h pendente abaixa o bit 26 (Ready to receive Cmd Word)"
    );

    dispara_dma(&mut bus, CHCR_PARA_RAM, 4);

    assert_ne!(
        gpustat(&bus) & (1 << 26),
        0,
        "drenada a janela inteira, a GPU volta a Idle e o bit 26 sobe — sem isso o driver de \
         GPU do kernel espera para sempre e imprime `GPU timeout`"
    );
    assert_eq!(
        gpustat(&bus) & (1 << 27),
        0,
        "o bit 27 (pronto para enviar VRAM->CPU) cai junto, porque nao ha mais o que enviar"
    );
}

#[test]
fn dma2_para_ram_avanca_o_madr_ate_o_fim_da_janela() {
    let mut bus = bus();
    semeia_vram(&mut bus);
    pede_c0h(&mut bus);

    dispara_dma(&mut bus, CHCR_PARA_RAM, 4);

    assert_eq!(
        bus.read32::<BusRead>(D2_MADR) & 0x00FF_FFFF,
        DESTINO + 16,
        "o MADR anda 4 bytes por palavra transferida, como no sentido oposto"
    );
}

#[test]
fn dma2_para_ram_limpa_o_bit24_do_chcr() {
    let mut bus = bus();
    semeia_vram(&mut bus);
    pede_c0h(&mut bus);

    dispara_dma(&mut bus, CHCR_PARA_RAM, 4);

    assert_eq!(
        bus.read32::<BusRead>(D2_CHCR) & (1 << 24),
        0,
        "terminada a transferencia, o bit 24 (start/busy) cai — e como o kernel sabe que acabou"
    );
}

#[test]
fn dma2_para_ram_transfere_so_o_tamanho_pedido_no_bcr() {
    let mut bus = bus();
    semeia_vram(&mut bus);
    pede_c0h(&mut bus);
    bus.write32::<BusWrite>(DESTINO + 8, 0xDEAD_BEEF);

    dispara_dma(&mut bus, CHCR_PARA_RAM, 2);

    assert_eq!(
        ram32(&bus, DESTINO + 8),
        0xDEAD_BEEF,
        "BCR pediu 2 palavras: a terceira posicao da RAM nao pode ser tocada"
    );
    assert_eq!(
        gpustat(&bus) & (1 << 26),
        0,
        "com a janela drenada pela metade a GPU continua no C0h e o bit 26 segue baixo"
    );
}

#[test]
fn dma2_para_ram_nao_roda_com_o_canal_desabilitado_no_dpcr() {
    let mut bus = asm::bus_with_bios_empty();
    bus.write32::<BusWrite>(DPCR, 0x0765_4321 & !(1 << 11));
    semeia_vram(&mut bus);
    pede_c0h(&mut bus);

    dispara_dma(&mut bus, CHCR_PARA_RAM, 4);

    assert_eq!(
        ram32(&bus, DESTINO),
        0,
        "o gate do DPCR vale nos dois sentidos"
    );
}

#[test]
fn dma2_da_ram_continua_empurrando_para_o_gp0() {
    let mut bus = bus();
    // E5h (drawing offset) e um comando de 1 palavra que muda GPUSTAT? nao: usamos E1h, que
    // grava os bits 0-10 do GPUSTAT, para provar que a palavra chegou ao GP0.
    bus.write32::<BusWrite>(0x0000_3000, 0xE100_0007);
    bus.write32::<BusWrite>(D2_MADR, 0x0000_3000);
    bus.write32::<BusWrite>(D2_BCR, (1 << 16) | 1);
    bus.write32::<BusWrite>(D2_CHCR, CHCR_DA_RAM);

    assert_eq!(
        gpustat(&bus) & 0x7,
        0x7,
        "regressao: com o bit 0 do CHCR em 1 o sentido continua RAM -> device, e o E1h escrito \
         pelo DMA aparece nos bits baixos do GPUSTAT"
    );
}
