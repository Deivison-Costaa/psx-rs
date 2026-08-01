mod support;

use psx_core::bus::{Bus, BusRead, BusWrite};
use support::asm;

fn bus() -> Bus {
    asm::bus_with_bios_empty()
}

#[test]
fn escrita_no_post_nao_vaza_para_a_ram() {
    let mut bus = bus();
    bus.write32::<BusWrite>(0x0000_2040, 0xCAFE_BABE);
    bus.write8::<BusWrite>(0x1F80_2041, 0x0F);
    assert_eq!(
        bus.read32::<BusRead>(0x0000_2040),
        0xCAFE_BABE,
        "Expansion Region 2 e I/O de 8K (01-memory-map.md L32), nao RAM — escrever no \
         POST (1F802041h) nao pode corromper RAM[2041h] pelo fallback mascarado"
    );
}

#[test]
fn leitura_da_expansion_2_nao_vem_da_ram() {
    let mut bus = bus();
    bus.write32::<BusWrite>(0x0000_2040, 0x5A5A_5A5A);
    let v = bus.read8::<BusRead>(0x1F80_2041);
    assert_ne!(
        v, 0x5A,
        "ler 1F802041h nao pode devolver o byte da RAM fisica 2041h"
    );
}

#[test]
fn alias_kseg1_do_post_tambem_nao_vaza() {
    let mut bus = bus();
    bus.write32::<BusWrite>(0x0000_2040, 0x1122_3344);
    bus.write8::<BusWrite>(0xBF80_2041, 0x07);
    assert_eq!(
        bus.read32::<BusRead>(0x0000_2040),
        0x1122_3344,
        "espelho KSEG1 (BF802041h) da Expansion Region 2 idem (01-memory-map.md L32)"
    );
}

#[test]
fn escrita_de_16_e_32_bits_na_expansion_2_tambem_nao_vaza() {
    let mut bus = bus();
    bus.write32::<BusWrite>(0x0000_2040, 0x0BAD_F00D);
    bus.write16::<BusWrite>(0x1F80_2040, 0xDEAD);
    bus.write32::<BusWrite>(0x1F80_2040, 0xFEED_BEEF);
    assert_eq!(
        bus.read32::<BusRead>(0x0000_2040),
        0x0BAD_F00D,
        "todos os tamanhos de acesso a 1F8020xxh ficam fora da RAM — o padrao \
         porta-cai-no-sumidouro ja mordeu duas vezes (4.4j, 4.4m)"
    );
}
