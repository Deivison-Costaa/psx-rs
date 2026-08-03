mod support;

use psx_core::bus::{Bus, BusRead};
use support::asm;

// § MDEC I/O Ports e DMA (docs/reference/09-mdec.md L61-124).
const MDEC_CMD: u32 = 0x1F80_1820;
const MDEC_STATUS: u32 = 0x1F80_1824;
const D0_MADR: u32 = 0x1F80_1080;
const D0_BCR: u32 = 0x1F80_1084;
const D0_CHCR: u32 = 0x1F80_1088;
const D1_MADR: u32 = 0x1F80_1090;
const D1_BCR: u32 = 0x1F80_1094;
const D1_CHCR: u32 = 0x1F80_1098;
const DPCR: u32 = 0x1F80_10F0;

// § Commonly used DMA Control Register values (docs/reference/04-dma.md L159-168).
const CHCR_MDEC_IN: u32 = 0x0100_0201;
const CHCR_MDEC_OUT: u32 = 0x0100_0200;

fn bus_com_mdec() -> Bus {
    asm::bus_with_bios_empty()
}

fn write_ram32(bus: &mut Bus, addr: u32, val: u32) {
    bus.write32::<BusRead>(addr, val);
}

// § MDEC(2) Set Quant Table (L141-149): comando (2<<29)|color, so luminancia.
fn cmd_quant_table_luminancia() -> u32 {
    2 << 29
}

// § MDEC(3) Set Scale Table (L151-158).
fn cmd_scale_table() -> u32 {
    3 << 29
}

// § MDEC(1) Decode Macroblock (L129-139): depth=0 (4bit), 32 palavras (0x80 bytes).
fn cmd_decode_4bit_32_palavras() -> u32 {
    (1 << 29) | 32
}

#[test]
fn mdec_status_inicial_fifo_de_saida_vazia_e_nao_ocupado() {
    let bus = bus_com_mdec();
    let status = bus.read32::<BusRead>(MDEC_STATUS);
    assert_eq!(
        status & (1 << 31),
        1 << 31,
        "bit31 Data-Out Fifo Empty=1 ao ligar"
    );
    assert_eq!(status & (1 << 29), 0, "bit29 Command Busy=0 ao ligar");
}

#[test]
fn mdec_reset_limpa_estado_ocupado_de_um_comando_em_andamento() {
    let mut bus = bus_com_mdec();
    bus.write32::<BusRead>(MDEC_CMD, cmd_quant_table_luminancia());
    assert_eq!(
        bus.read32::<BusRead>(MDEC_STATUS) & (1 << 29),
        1 << 29,
        "bit29 deve subir apos o comando (16 palavras de parametro ainda faltam)"
    );

    // § MDEC1 Control/Reset bit31 (L103): "Abort any command".
    bus.write32::<BusRead>(MDEC_STATUS, 1 << 31);
    assert_eq!(
        bus.read32::<BusRead>(MDEC_STATUS) & (1 << 29),
        0,
        "reset deve abortar o comando pendente"
    );
}

#[test]
fn mdec_quant_table_luminancia_completa_apos_16_palavras() {
    let mut bus = bus_com_mdec();
    bus.write32::<BusRead>(MDEC_CMD, cmd_quant_table_luminancia());
    for _ in 0..15 {
        bus.write32::<BusRead>(MDEC_CMD, 0);
        assert_eq!(
            bus.read32::<BusRead>(MDEC_STATUS) & (1 << 29),
            1 << 29,
            "ainda ocupado antes da 16a palavra"
        );
    }
    bus.write32::<BusRead>(MDEC_CMD, 0);
    assert_eq!(
        bus.read32::<BusRead>(MDEC_STATUS) & (1 << 29),
        0,
        "16 palavras (64 bytes) fecham o comando MDEC(2) sem cor"
    );
}

#[test]
fn mdec_scale_table_completa_apos_32_palavras() {
    let mut bus = bus_com_mdec();
    bus.write32::<BusRead>(MDEC_CMD, cmd_scale_table());
    for _ in 0..31 {
        bus.write32::<BusRead>(MDEC_CMD, 0);
    }
    assert_eq!(
        bus.read32::<BusRead>(MDEC_STATUS) & (1 << 29),
        1 << 29,
        "ainda ocupado antes da 32a palavra (64 halfwords)"
    );
    bus.write32::<BusRead>(MDEC_CMD, 0);
    assert_eq!(
        bus.read32::<BusRead>(MDEC_STATUS) & (1 << 29),
        0,
        "32 palavras fecham o comando MDEC(3)"
    );
}

#[test]
fn mdec_dma0_decode_dma_entrega_todas_as_palavras_e_destrava_o_comando() {
    let mut bus = bus_com_mdec();
    bus.write32::<BusRead>(MDEC_CMD, cmd_decode_4bit_32_palavras());
    assert_eq!(bus.read32::<BusRead>(MDEC_STATUS) & (1 << 29), 1 << 29);

    let data_addr: u32 = 0x0000_1000;
    for i in 0..32u32 {
        write_ram32(&mut bus, data_addr + i * 4, 0xFE00_FE00);
    }
    bus.write32::<BusRead>(D0_MADR, data_addr);
    bus.write32::<BusRead>(D0_BCR, 0x0001_0020); // BS=0x20 blocos, BA=1
    bus.write32::<BusRead>(DPCR, 0x0765_4321 | (1 << 3));
    bus.write32::<BusRead>(D0_CHCR, CHCR_MDEC_IN);

    assert_eq!(
        bus.read32::<BusRead>(D0_CHCR) & (1 << 24),
        0,
        "DMA0 entregou o bloco inteiro (BS*BA==32==numero de parametros)"
    );
    assert_eq!(
        bus.read32::<BusRead>(MDEC_STATUS) & (1 << 29),
        0,
        "comando MDEC(1) fechou: 32 palavras de FE00 (padding) so tem fim de bloco"
    );
}

#[test]
fn mdec_dma1_pede_mais_que_o_decodificado_mantem_canal_em_andamento() {
    let mut bus = bus_com_mdec();
    // Um bloco mono minimo (DC=0 com q_scale=0, sem AC) decodifica para 8
    // palavras (32 bytes) em profundidade 4-bit — ver mdec_macroblock_decode.rs.
    bus.write32::<BusRead>(MDEC_CMD, (1 << 29) | 1);
    bus.write32::<BusRead>(MDEC_CMD, 0x0000_FE00);
    assert_eq!(bus.mdec().output_len(), 8);

    let dst_addr: u32 = 0x0000_2000;
    bus.write32::<BusRead>(D1_MADR, dst_addr);
    // § docs/reference/09-mdec.md L122-124: DMA1 usa blocksize 0x20 — igual ao
    // main.cpp do ps1-tests mdec/4bit/8bit, que pede mais do que o bloco unico
    // decodificado tem (defeito do binario de teste, nao do nosso MDEC).
    bus.write32::<BusRead>(D1_BCR, 0x0001_0020);
    bus.write32::<BusRead>(DPCR, 0x0765_4321 | (1 << 7));
    bus.write32::<BusRead>(D1_CHCR, CHCR_MDEC_OUT);

    assert_eq!(
        bus.read32::<BusRead>(D1_CHCR) & (1 << 24),
        1 << 24,
        "canal continua 'em andamento': so 8 das 32 palavras pedidas existem"
    );
    assert_eq!(
        bus.mdec().output_len(),
        0,
        "as 8 palavras disponiveis foram consumidas"
    );
}
