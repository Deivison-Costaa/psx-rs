mod support;

use psx_core::bus::{Bus, BusRead, BusWrite};
use support::asm;

// Faixas FISICAS que o mapa de memoria (docs/reference/01-memory-map.md L28-L34) NAO lista, e
// que por isso nao podem ter respaldo em RAM. O offset ao lado e' o endereco de RAM que o
// fallback mascarado por 1FFFFFh atingiria.
const NAO_MAPEADOS: [(u32, u32, &str); 9] = [
    (0x1F00_0000, 0x00_0000, "Expansion Region 1 (L29)"),
    (0x1F00_0080, 0x00_0080, "EXP1 aliasando o vetor de excecao"),
    (
        0x1F80_0400,
        0x00_0400,
        "vao acima do scratchpad de 1K (L30)",
    ),
    (
        0x1F80_4000,
        0x00_4000,
        "vao entre I/O e Expansion 3 (L31/L33)",
    ),
    (0x1FA0_0000, 0x00_0000, "Expansion Region 3 (L33)"),
    (0x1FC0_0000, 0x00_0000, "BIOS ROM e somente leitura (L34)"),
    (
        0x0080_0000,
        0x00_0000,
        "acima do espelho de 8MB da RAM (L146)",
    ),
    (
        0x2000_0000,
        0x00_0000,
        "acima dos 512MB de KUSEG (L143-L145)",
    ),
    (0x6400_1234, 0x00_1234, "ponteiro lixo tipico fora de KUSEG"),
];

// Enderecos que TEM de continuar chegando na RAM: os 2MB fisicos e os tres espelhos que os
// completam ate 8MB ("2MB RAM can be mirrored to the first 8MB (strangely, enabled by
// default)", 01-memory-map.md L146), mais o alias de KSEG0 que o Tomb Raider usa (80200080h).
const ESPELHOS_LEGITIMOS: [(u32, u32, &str); 6] = [
    (0x0000_0080, 0x00_0080, "RAM fisica direta"),
    (0x0020_0080, 0x00_0080, "espelho 2 dentro dos 8MB"),
    (0x0040_0080, 0x00_0080, "espelho 3 dentro dos 8MB"),
    (0x0060_0080, 0x00_0080, "espelho 4 dentro dos 8MB"),
    (0x8020_0080, 0x00_0080, "espelho de KSEG0 (fisico 200080h)"),
    (0xA060_0080, 0x00_0080, "espelho de KSEG1 (fisico 600080h)"),
];

const SENTINELA: u32 = 0xCAFE_BABE;

fn bus() -> Bus {
    asm::bus_with_bios_empty()
}

/// Le a RAM por um endereco que e' inequivocamente RAM (KSEG1 abaixo de 2MB), sem passar pelo
/// endereco sob teste.
fn ram32(bus: &Bus, offset: u32) -> u32 {
    bus.read32::<BusRead>(0xA000_0000 | offset)
}

fn semeia(bus: &mut Bus, offset: u32) {
    bus.write32::<BusWrite>(0xA000_0000 | offset, SENTINELA);
}

#[test]
fn write32_em_regiao_nao_mapeada_nao_altera_a_ram() {
    for (addr, offset, nome) in NAO_MAPEADOS {
        let mut bus = bus();
        semeia(&mut bus, offset);

        bus.write32::<BusWrite>(addr, 0x1234_5678);

        assert_eq!(
            ram32(&bus, offset),
            SENTINELA,
            "sw em {addr:08X} ({nome}) nao pode cair na RAM {offset:06X}"
        );
    }
}

#[test]
fn write16_em_regiao_nao_mapeada_nao_altera_a_ram() {
    for (addr, offset, nome) in NAO_MAPEADOS {
        let mut bus = bus();
        semeia(&mut bus, offset);

        bus.write16::<BusWrite>(addr, 0x1234);

        assert_eq!(
            ram32(&bus, offset),
            SENTINELA,
            "sh em {addr:08X} ({nome}) nao pode cair na RAM {offset:06X}"
        );
    }
}

#[test]
fn write8_em_regiao_nao_mapeada_nao_altera_a_ram() {
    for (addr, offset, nome) in NAO_MAPEADOS {
        let mut bus = bus();
        semeia(&mut bus, offset);

        bus.write8::<BusWrite>(addr, 0x5A);

        assert_eq!(
            ram32(&bus, offset),
            SENTINELA,
            "sb em {addr:08X} ({nome}) nao pode cair na RAM {offset:06X}"
        );
    }
}

// A leitura sofre do mesmo fallback: sem guarda, ler um endereco nao mapeado devolve o
// conteudo da RAM mascarada. O BIOS fica de fora da lista porque 1FC00000h e' ROM de verdade
// (L34) e ja tem guarda propria na leitura.
#[test]
fn leitura_de_regiao_nao_mapeada_nao_vem_da_ram() {
    for (addr, offset, nome) in NAO_MAPEADOS {
        if addr == 0x1FC0_0000 {
            continue;
        }
        let mut bus = bus();
        semeia(&mut bus, offset);

        assert_ne!(
            bus.read32::<BusRead>(addr),
            SENTINELA,
            "lw em {addr:08X} ({nome}) nao pode devolver a RAM {offset:06X}"
        );
        assert_ne!(
            bus.read16::<BusRead>(addr),
            (SENTINELA & 0xFFFF) as u16,
            "lh em {addr:08X} ({nome}) nao pode devolver a RAM {offset:06X}"
        );
        assert_ne!(
            bus.read8::<BusRead>(addr),
            (SENTINELA & 0xFF) as u8,
            "lb em {addr:08X} ({nome}) nao pode devolver a RAM {offset:06X}"
        );
    }
}

#[test]
fn controle_espelho_de_ram_continua_funcionando_na_escrita() {
    for (addr, offset, nome) in ESPELHOS_LEGITIMOS {
        let mut bus = bus();

        bus.write32::<BusWrite>(addr, SENTINELA);

        assert_eq!(
            ram32(&bus, offset),
            SENTINELA,
            "sw em {addr:08X} ({nome}) tem de chegar na RAM {offset:06X}"
        );
    }
}

#[test]
fn controle_espelho_de_ram_continua_funcionando_na_leitura() {
    for (addr, offset, nome) in ESPELHOS_LEGITIMOS {
        let mut bus = bus();
        semeia(&mut bus, offset);

        assert_eq!(
            bus.read32::<BusRead>(addr),
            SENTINELA,
            "lw em {addr:08X} ({nome}) tem de ler a RAM {offset:06X}"
        );
        assert_eq!(
            bus.read16::<BusRead>(addr),
            (SENTINELA & 0xFFFF) as u16,
            "lh em {addr:08X} ({nome}) tem de ler a RAM {offset:06X}"
        );
        assert_eq!(
            bus.read8::<BusRead>(addr),
            (SENTINELA & 0xFF) as u8,
            "lb em {addr:08X} ({nome}) tem de ler a RAM {offset:06X}"
        );
    }
}

// Controle de borda: o ultimo byte dentro do espelho de 8MB ainda e' RAM, o primeiro byte
// acima dele ja nao e'. Fixa onde a janela termina.
#[test]
fn controle_borda_dos_8mb() {
    let mut dentro = bus();
    dentro.write8::<BusWrite>(0x007F_FFFF, 0x77);
    assert_eq!(
        dentro.read8::<BusRead>(0x001F_FFFF),
        0x77,
        "7FFFFFh ainda esta dentro do espelho de 8MB (L146)"
    );

    let mut acima = bus();
    semeia(&mut acima, 0x1F_FFFC);
    acima.write32::<BusWrite>(0x009F_FFFC, 0x1111_1111);
    assert_eq!(
        ram32(&acima, 0x1F_FFFC),
        SENTINELA,
        "9FFFFCh ja esta acima do espelho de 8MB e nao pode escrever na RAM"
    );
}
