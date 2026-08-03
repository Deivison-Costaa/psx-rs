mod support;

use psx_core::bus::{Bus, BusRead};
use support::asm;

// § Linked List DMA (docs/reference/04-dma.md L198-216): "The transfer is stopped
// once an end marker is reached." A spec nao impoe teto de nos — quem percorre a
// cadeia so para no end-marker. § D#_MADR (L52-54): em SyncMode=2 o MADR guarda o
// end marker ao fim da transferencia.
//
// O unico limite real e a propria RAM: cada no comeca num endereco alinhado a
// palavra e o proximo endereco sai inteiro do header, entao uma cadeia com mais
// nos do que ha palavras na RAM repetiu algum endereco por principio da casa dos
// pombos — ou seja, tem ciclo, e ciclo nunca completa (dma_chain_looping.rs).

fn bus_com_dma() -> Bus {
    asm::bus_with_bios_empty()
}

const D2_MADR: u32 = 0x1F80_10A0;
const D2_BCR: u32 = 0x1F80_10A4;
const D2_CHCR: u32 = 0x1F80_10A8;
const DPCR: u32 = 0x1F80_10F0;
const GPUSTAT: u32 = 0x1F80_1814;

const BASE: u32 = 0x0001_0000;
const FIM: u32 = 0x00FF_FFFF;

fn habilitar_canal2(bus: &mut Bus) {
    let dpcr = bus.read32::<BusRead>(DPCR);
    bus.write32::<BusRead>(DPCR, dpcr | (1 << 11));
}

/// Monta uma cadeia de `nos` nos vazios (0 palavras extras) terminada em end-marker.
fn monta_cadeia_vazia(bus: &mut Bus, nos: u32) {
    for i in 0..nos {
        let addr = BASE + i * 4;
        let proximo = if i + 1 == nos { FIM } else { addr + 4 };
        bus.write32::<BusRead>(addr, proximo & 0x00FF_FFFF);
    }
}

fn dispara(bus: &mut Bus) {
    bus.write32::<BusRead>(D2_MADR, BASE);
    bus.write32::<BusRead>(D2_BCR, 0);
    bus.write32::<BusRead>(D2_CHCR, 0x0100_0401);
}

#[test]
fn cadeia_de_4097_nos_completa() {
    let mut bus = bus_com_dma();
    habilitar_canal2(&mut bus);
    monta_cadeia_vazia(&mut bus, 4097);
    dispara(&mut bus);

    assert_eq!(
        bus.read32::<BusRead>(D2_CHCR) & (1 << 24),
        0,
        "cadeia terminada em end-marker completa qualquer que seja o numero de nos; \
         teto artificial de nos deixa o canal ocupado para sempre (achado 0185.2)"
    );
    assert_eq!(
        bus.read32::<BusRead>(D2_MADR),
        FIM,
        "§ D#_MADR L52-54: em SyncMode=2 o MADR guarda o end marker ao fim"
    );
}

#[test]
fn cadeia_de_20000_nos_completa() {
    let mut bus = bus_com_dma();
    habilitar_canal2(&mut bus);
    monta_cadeia_vazia(&mut bus, 20000);
    dispara(&mut bus);

    assert_eq!(
        bus.read32::<BusRead>(D2_CHCR) & (1 << 24),
        0,
        "nem 20000 nos param a cadeia: a spec so conhece o end-marker"
    );
}

#[test]
fn cadeia_longa_entrega_todas_as_palavras_ao_gp0() {
    let mut bus = bus_com_dma();
    habilitar_canal2(&mut bus);

    // 5000 nos de 1 palavra extra cada: header, dado, header, dado...
    // Cada dado e um GP0(E1h), que aterrissa nos bits 0-10 do GPUSTAT. O ultimo no
    // leva um valor diferente dos outros, entao o GPUSTAT final so bate se a cadeia
    // foi ate o fim E cada palavra saiu do lugar certo.
    let nos = 5000u32;
    for i in 0..nos {
        let addr = BASE + i * 8;
        let ultimo = i + 1 == nos;
        let proximo = if ultimo { FIM } else { addr + 8 };
        let texpage = if ultimo { 0x1F } else { 0x0A };
        bus.write32::<BusRead>(addr, (1 << 24) | (proximo & 0x00FF_FFFF));
        bus.write32::<BusRead>(addr + 4, 0xE100_0000 | texpage);
    }
    dispara(&mut bus);

    assert_eq!(
        bus.read32::<BusRead>(D2_CHCR) & (1 << 24),
        0,
        "cadeia longa com dados tambem completa"
    );
    assert_eq!(
        bus.read32::<BusRead>(D2_MADR),
        FIM,
        "MADR final e o end marker mesmo com 5000 nos"
    );
    assert_eq!(
        bus.read32::<BusRead>(GPUSTAT) & 0x7FF,
        0x1F,
        "GPUSTAT tem o E1h do ULTIMO no: a cadeia inteira passou pelo GP0, cada \
         palavra lida do endereco certo"
    );
}
