mod support;

use psx_core::bus::{Bus, BusRead, BusWrite};
use support::asm;

const DPCR: u32 = 0x1F80_10F0;
const DICR: u32 = 0x1F80_10F4;
const D6_MADR: u32 = 0x1F80_10E0;
const D6_BCR: u32 = 0x1F80_10E4;
const D6_CHCR: u32 = 0x1F80_10E8;
const I_STAT: u32 = 0x1F80_1070;
const IRQ3: u32 = 1 << 3;

const MASTER: u32 = 1 << 23;
const MASCARA_CH6: u32 = 1 << 22;
const FLAG_CH6: u32 = 1 << 30;
const BUS_ERROR: u32 = 1 << 15;
const MESTRE: u32 = 1 << 31;

fn bus() -> Bus {
    asm::bus_with_bios_empty()
}

fn dicr(bus: &Bus) -> u32 {
    bus.read32::<BusRead>(DICR)
}

fn escreve_dicr(bus: &mut Bus, val: u32) {
    bus.write32::<BusWrite>(DICR, val);
}

fn i_stat(bus: &Bus) -> u32 {
    bus.read32::<BusRead>(I_STAT)
}

fn ack_i_stat_irq3(bus: &mut Bus) {
    bus.write32::<BusWrite>(I_STAT, !IRQ3);
}

// OTC (canal 6) e o gatilho mais barato de uma conclusao de DMA: nao precisa de GPU nem de disco.
fn conclui_otc(bus: &mut Bus) {
    bus.write32::<BusWrite>(DPCR, 0x0765_4321 | (1 << 27));
    bus.write32::<BusWrite>(D6_MADR, 0x0000_0010);
    bus.write32::<BusWrite>(D6_BCR, 1);
    bus.write32::<BusWrite>(D6_CHCR, 0x1100_0002);
}

#[test]
fn flag_do_canal_sobe_com_mascara_e_master_ligados() {
    let mut bus = bus();
    escreve_dicr(&mut bus, MASTER | MASCARA_CH6);

    conclui_otc(&mut bus);

    assert_ne!(
        dicr(&bus) & FLAG_CH6,
        0,
        "concluida a transferencia do canal 6, o flag do bit 30 sobe quando o bit 22 (mascara \
         do canal) e o bit 23 (master enable) estao ligados"
    );
}

#[test]
fn flag_nao_sobe_sem_o_master_do_bit23() {
    let mut bus = bus();
    escreve_dicr(&mut bus, MASCARA_CH6);

    conclui_otc(&mut bus);

    assert_eq!(
        dicr(&bus) & FLAG_CH6,
        0,
        "os flags 24-30 sao setados SO se o bit (16+n) E o bit 23 estiverem ligados — ao \
         contrario do I_STAT, que marca independente de mascara"
    );
}

#[test]
fn flag_nao_sobe_sem_a_mascara_do_canal() {
    let mut bus = bus();
    escreve_dicr(&mut bus, MASTER);

    conclui_otc(&mut bus);

    assert_eq!(
        dicr(&bus) & FLAG_CH6,
        0,
        "sem o bit 22 nao ha flag do canal 6, mesmo com o master ligado"
    );
}

#[test]
fn bit15_de_bus_error_forca_o_bit31() {
    let mut bus = bus();

    escreve_dicr(&mut bus, BUS_ERROR);

    assert_ne!(
        dicr(&bus) & MESTRE,
        0,
        "b31 = b15 OR (b23 AND b24-30 != 0): o bus error sozinho ja forca o flag mestre"
    );
}

#[test]
fn bit31_nao_olha_as_mascaras_por_canal() {
    let mut bus = bus();
    escreve_dicr(&mut bus, MASTER | MASCARA_CH6);
    conclui_otc(&mut bus);
    assert_ne!(dicr(&bus) & MESTRE, 0, "pre-condicao: b31 ligado");

    // Desliga a mascara do canal SEM escrever 1 no flag (bits 24-30 zerados = sem ack).
    escreve_dicr(&mut bus, MASTER);

    assert_ne!(
        dicr(&bus) & FLAG_CH6,
        0,
        "escrever 0 num bit 24-30 nao limpa o flag: so a escrita de 1 reseta"
    );
    assert_ne!(
        dicr(&bus) & MESTRE,
        0,
        "as mascaras por canal (b16-22) nao entram no calculo do b31; uma vez setado, o flag \
         conta para o mestre mesmo com a mascara desligada"
    );
}

#[test]
fn borda_de_0_para_1_do_bit31_levanta_i_stat_bit3() {
    let mut bus = bus();
    escreve_dicr(&mut bus, MASTER | MASCARA_CH6);
    assert_eq!(i_stat(&bus) & IRQ3, 0, "pre-condicao: I_STAT.3 limpo");

    conclui_otc(&mut bus);

    assert_eq!(
        i_stat(&bus) & IRQ3,
        IRQ3,
        "na transicao 0->1 do bit 31 do DICR, o flag IRQ3 do I_STAT e setado"
    );
}

#[test]
fn i_stat_bit3_e_de_borda_e_nao_volta_sozinho() {
    let mut bus = bus();
    escreve_dicr(&mut bus, MASTER | MASCARA_CH6);
    conclui_otc(&mut bus);
    ack_i_stat_irq3(&mut bus);
    assert_eq!(i_stat(&bus) & IRQ3, 0, "pre-condicao: I_STAT.3 limpo no ack");

    // O bit 31 continua alto (o flag do canal nao foi reconhecido): nao ha nova borda 0->1.
    escreve_dicr(&mut bus, MASTER | MASCARA_CH6);
    for _ in 0..4 {
        bus.tick_timers(1_000);
    }

    assert_eq!(
        i_stat(&bus) & IRQ3,
        0,
        "sem o bit 31 descer e subir de novo nao existe borda, e os bits do I_STAT sao de borda"
    );
}

#[test]
fn escrever_1_no_flag_limpa_o_flag_e_derruba_o_bit31() {
    let mut bus = bus();
    escreve_dicr(&mut bus, MASTER | MASCARA_CH6);
    conclui_otc(&mut bus);
    assert_ne!(dicr(&bus) & FLAG_CH6, 0, "pre-condicao: flag do canal 6 alto");

    escreve_dicr(&mut bus, MASTER | MASCARA_CH6 | FLAG_CH6);

    assert_eq!(
        dicr(&bus) & FLAG_CH6,
        0,
        "os bits 24-30 sao reconhecidos escrevendo 1 neles"
    );
    assert_eq!(
        dicr(&bus) & MESTRE,
        0,
        "sem flag de canal e sem bus error, o b31 recalculado volta a zero"
    );
}

#[test]
fn nova_conclusao_depois_do_ack_levanta_irq3_de_novo() {
    let mut bus = bus();
    escreve_dicr(&mut bus, MASTER | MASCARA_CH6);
    conclui_otc(&mut bus);
    ack_i_stat_irq3(&mut bus);
    escreve_dicr(&mut bus, MASTER | MASCARA_CH6 | FLAG_CH6);
    assert_eq!(i_stat(&bus) & IRQ3, 0, "pre-condicao: tudo reconhecido");

    conclui_otc(&mut bus);

    assert_eq!(
        i_stat(&bus) & IRQ3,
        IRQ3,
        "com o b31 zerado pelo ack, a conclusao seguinte e uma nova borda 0->1 e o IRQ3 sobe \
         outra vez — sem isso o kernel espera para sempre pelo fim do DMA"
    );
}
