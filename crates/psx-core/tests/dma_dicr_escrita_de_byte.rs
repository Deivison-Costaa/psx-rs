mod support;

use psx_core::bus::{Bus, BusRead, BusWrite};
use support::asm;

const DPCR: u32 = 0x1F80_10F0;
const DICR: u32 = 0x1F80_10F4;
const DICR_BYTE2: u32 = 0x1F80_10F6;
const D6_MADR: u32 = 0x1F80_10E0;
const D6_BCR: u32 = 0x1F80_10E4;
const D6_CHCR: u32 = 0x1F80_10E8;

const MASTER: u32 = 1 << 23;
const MASCARA_CH3: u32 = 1 << 19;
const MASCARA_CH6: u32 = 1 << 22;
const FLAG_CH6: u32 = 1 << 30;

fn bus() -> Bus {
    asm::bus_with_bios_empty()
}

fn dicr(bus: &Bus) -> u32 {
    bus.read32::<BusRead>(DICR)
}

fn conclui_otc(bus: &mut Bus) {
    bus.write32::<BusWrite>(DPCR, 0x0765_4321 | (1 << 27));
    bus.write32::<BusWrite>(D6_MADR, 0x0000_0010);
    bus.write32::<BusWrite>(D6_BCR, 1);
    bus.write32::<BusWrite>(D6_CHCR, 0x1100_0002);
}

#[test]
fn lbu_no_terceiro_byte_do_dicr_devolve_mascara_e_master() {
    let mut bus = bus();
    bus.write32::<BusWrite>(DICR, MASTER | MASCARA_CH3);

    assert_eq!(
        bus.read8::<BusRead>(DICR_BYTE2),
        0x88,
        "ler o byte 2 do DICR tem de devolver os bits 16-23 do registrador; o driver de \
         streaming faz read-modify-write e um zero fixo aqui apaga o master enable"
    );
}

#[test]
fn lhu_na_metade_alta_do_dicr_devolve_mascara_e_master() {
    let mut bus = bus();
    bus.write32::<BusWrite>(DICR, MASTER | MASCARA_CH3);

    assert_eq!(
        bus.read16::<BusRead>(DICR + 2),
        0x0088,
        "a meia-palavra alta do DICR sao os bits 16-31; sem flag de conclusao e sem bus \
         error o b31 esta baixo e sobra 0088h"
    );
}

#[test]
fn sb_no_terceiro_byte_do_dicr_liga_a_mascara_do_canal() {
    let mut bus = bus();
    bus.write32::<BusWrite>(DICR, MASTER);

    // `sb rt,2(1F8010F4h)` com rt = 88h: master (bit 23) + mascara do canal 3 (bit 19).
    bus.write8_gpr_completo::<BusWrite>(DICR_BYTE2, 0x0000_0088);

    assert_eq!(
        dicr(&bus) & (MASTER | MASCARA_CH3),
        MASTER | MASCARA_CH3,
        "o byte 2 do DICR sao os bits 16-23; uma escrita de byte nesse endereco tem de \
         chegar na mascara por canal e no master enable"
    );
}

#[test]
fn sb_no_terceiro_byte_do_dicr_desliga_a_mascara_do_canal() {
    let mut bus = bus();
    bus.write32::<BusWrite>(DICR, MASTER | MASCARA_CH6);

    // Mesmo `sb`, agora com rt = 80h: so o master; todas as mascaras por canal desligadas.
    bus.write8_gpr_completo::<BusWrite>(DICR_BYTE2, 0x0000_0080);

    assert_eq!(
        dicr(&bus) & MASCARA_CH6,
        0,
        "escrever 80h no byte 2 desliga as mascaras dos canais 0-6 e mantem so o bit 23"
    );
    assert_eq!(
        dicr(&bus) & MASTER,
        MASTER,
        "o bit 23 esta dentro do mesmo byte e continua ligado"
    );
}

#[test]
fn mascara_desligada_por_sb_impede_o_flag_de_conclusao() {
    let mut bus = bus();
    bus.write32::<BusWrite>(DICR, MASTER | MASCARA_CH6);

    bus.write8_gpr_completo::<BusWrite>(DICR_BYTE2, 0x0000_0080);
    conclui_otc(&mut bus);

    assert_eq!(
        dicr(&bus) & FLAG_CH6,
        0,
        "sem a mascara do canal 6 nao ha flag de conclusao — e a mascara foi desligada por \
         uma escrita de byte, que e como o driver de streaming da Sony liga e desliga a \
         interrupcao de fim de DMA entre um setor e outro"
    );
}

#[test]
fn sb_no_terceiro_byte_do_dicr_nao_reconhece_os_flags_de_conclusao() {
    let mut bus = bus();
    bus.write32::<BusWrite>(DICR, MASTER | MASCARA_CH6);
    conclui_otc(&mut bus);
    assert_ne!(
        dicr(&bus) & FLAG_CH6,
        0,
        "pre-condicao: flag do canal 6 alto"
    );

    bus.write8_gpr_completo::<BusWrite>(DICR_BYTE2, 0x0000_0080);

    assert_ne!(
        dicr(&bus) & FLAG_CH6,
        0,
        "os bits 24-30 moram no byte 3, fora do alcance dessa escrita: o ack por escrita \
         de 1 nao pode acontecer de tabela"
    );
}
