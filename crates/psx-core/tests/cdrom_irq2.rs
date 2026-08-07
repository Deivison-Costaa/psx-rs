mod support;

use psx_core::bus::{Bus, BusRead, BusWrite};
use psx_core::cpu::Cpu;
use support::asm;

const CD_BASE: u32 = 0x1F80_1800;
const ESPERA_PRIMEIRA_RESPOSTA: u32 = 0x1_4000;
const I_STAT: u32 = 0x1F80_1070;
const IRQ2: u32 = 1 << 2;

fn bus() -> Bus {
    asm::bus_with_bios_empty()
}

fn cd_write(bus: &mut Bus, offset: u32, val: u8) {
    bus.write8::<BusWrite>(CD_BASE + offset, val);
}

fn set_bank(bus: &mut Bus, b: u8) {
    cd_write(bus, 0, b);
}

fn i_stat(bus: &Bus) -> u32 {
    bus.read32::<BusRead>(I_STAT)
}

fn ack_i_stat_irq2(bus: &mut Bus) {
    bus.write32::<BusWrite>(I_STAT, !IRQ2);
}

fn habilita_hintmsk(bus: &mut Bus, val: u8) {
    set_bank(bus, 1);
    cd_write(bus, 2, val);
}

fn ack_hclrctl(bus: &mut Bus, val: u8) {
    set_bank(bus, 1);
    cd_write(bus, 3, val);
}

fn manda_getstat(bus: &mut Bus) {
    set_bank(bus, 0);
    cd_write(bus, 1, 0x01);
    bus.tick_timers(ESPERA_PRIMEIRA_RESPOSTA);
}

#[test]
fn hintmsk_e_hintsts_nao_zero_levanta_i_stat_bit2() {
    let mut bus = bus();
    habilita_hintmsk(&mut bus, 0x1F);

    assert_eq!(
        i_stat(&bus) & IRQ2,
        0,
        "antes de qualquer comando, I_STAT.2 deve estar limpo"
    );

    manda_getstat(&mut bus);

    assert_eq!(
        i_stat(&bus) & IRQ2,
        IRQ2,
        "GetStat responde INT3; com HINTMSK=1Fh o drive dispara IRQ e I_STAT.2 (IRQ2 CDROM) \
         tem de ficar setado"
    );
}

#[test]
fn hintmsk_zerado_nao_levanta_i_stat_bit2() {
    let mut bus = bus();
    habilita_hintmsk(&mut bus, 0x00);

    manda_getstat(&mut bus);

    assert_eq!(
        i_stat(&bus) & IRQ2,
        0,
        "o drive dispara IRQ so quando (HINTMSK & HINTSTS) != 0; com HINTMSK=0 nao ha IRQ2 \
         mesmo com HINTSTS=3"
    );
}

#[test]
fn i_stat_bit2_e_de_borda_nao_volta_sozinho_sem_hclrctl() {
    let mut bus = bus();
    habilita_hintmsk(&mut bus, 0x1F);
    manda_getstat(&mut bus);
    assert_eq!(i_stat(&bus) & IRQ2, IRQ2, "IRQ2 tem de subir no comando");

    ack_i_stat_irq2(&mut bus);
    assert_eq!(
        i_stat(&bus) & IRQ2,
        0,
        "escrever 0 no bit 2 de I_STAT limpa o pedido"
    );

    // HINTSTS continua 3 e HINTMSK continua 1Fh: a fonte segue "true", entao NAO existe nova
    // borda false->true. Qualquer coisa que reponha o bit aqui e nivel, nao borda.
    set_bank(&mut bus, 0);
    for _ in 0..8 {
        bus.tick_timers(1_000);
    }

    assert_eq!(
        i_stat(&bus) & IRQ2,
        0,
        "sem ack do HCLRCTL a fonte nunca voltou de false para true; I_STAT.2 e de borda e \
         tem de continuar limpo"
    );
}

#[test]
fn nova_borda_depois_do_ack_do_hclrctl_levanta_de_novo() {
    let mut bus = bus();
    habilita_hintmsk(&mut bus, 0x1F);
    manda_getstat(&mut bus);
    ack_i_stat_irq2(&mut bus);

    ack_hclrctl(&mut bus, 0x07);
    assert_eq!(
        i_stat(&bus) & IRQ2,
        0,
        "limpar HINTSTS baixa a fonte; ainda sem pedido novo"
    );

    manda_getstat(&mut bus);

    assert_eq!(
        i_stat(&bus) & IRQ2,
        IRQ2,
        "com HINTSTS de volta a 3 ha nova borda false->true: I_STAT.2 sobe outra vez"
    );
}

#[test]
fn ack_do_hclrctl_sem_ack_do_i_stat_nao_gera_pedido_novo() {
    let mut bus = bus();
    habilita_hintmsk(&mut bus, 0x1F);
    manda_getstat(&mut bus);
    ack_hclrctl(&mut bus, 0x07);

    assert_eq!(
        i_stat(&bus) & IRQ2,
        IRQ2,
        "baixar a fonte nao limpa I_STAT: o bit so cai por escrita em I_STAT"
    );

    ack_i_stat_irq2(&mut bus);
    assert_eq!(
        i_stat(&bus) & IRQ2,
        0,
        "depois do ack em I_STAT com a fonte ja baixa, o bit fica limpo"
    );
}

#[test]
fn ack_parcial_do_hclrctl_deixa_a_fonte_alta_e_nao_gera_borda_nova() {
    let mut bus = bus();
    habilita_hintmsk(&mut bus, 0x1F);
    manda_getstat(&mut bus);
    ack_i_stat_irq2(&mut bus);

    // HCLRCTL=01h em HINTSTS=3 (INT3) deixa 2 (INT2): a fonte continua nao-zero.
    ack_hclrctl(&mut bus, 0x01);

    assert_eq!(
        i_stat(&bus) & IRQ2,
        0,
        "ack parcial nao zera (HINTMSK & HINTSTS); a fonte nunca desceu, entao nao ha borda \
         false->true e I_STAT.2 tem de continuar limpo"
    );

    set_bank(&mut bus, 0);
    assert_eq!(
        i_stat(&bus) & IRQ2,
        0,
        "nenhuma escrita posterior nos portos pode inventar uma borda com a fonte ja alta"
    );
}

#[test]
fn segunda_resposta_do_init_tambem_levanta_irq2() {
    let mut bus = bus();
    habilita_hintmsk(&mut bus, 0x1F);

    // Init (0Ah) --> INT3(stat) --> ack --> atraso fisico --> INT2(stat).
    set_bank(&mut bus, 0);
    cd_write(&mut bus, 1, 0x0A);
    bus.tick_timers(ESPERA_PRIMEIRA_RESPOSTA);
    assert_eq!(i_stat(&bus) & IRQ2, IRQ2, "INT3 do Init levanta IRQ2");

    ack_i_stat_irq2(&mut bus);
    ack_hclrctl(&mut bus, 0x07);
    // § L2077-2078: o Init tambem responde depois de uma busca, nao de um tempo fixo.
    let ciclos = bus.cdrom().second_response_cycles() as u32;
    bus.tick_timers(ciclos);

    assert_eq!(
        i_stat(&bus) & IRQ2,
        IRQ2,
        "a segunda resposta (INT2) do Init chega apos o atraso contado do ack e e uma nova \
         borda que levanta IRQ2 de novo — sem isso a BIOS espera para sempre pelo fim do comando"
    );
}

#[test]
fn cpu_vetora_para_0x80000080_quando_o_cdrom_pede_irq() {
    let mut bus = bus();
    bus.write32::<BusWrite>(0x1F80_1074, IRQ2);
    habilita_hintmsk(&mut bus, 0x1F);

    let mut cpu = Cpu::new();
    cpu.cop0[12] = 1 | (1 << 10);
    let pc_antes = cpu.pc;

    manda_getstat(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        cpu.pc, 0x8000_0080,
        "com I_MASK.2 ligado e IRQ2 pedido, o proximo passo tem de vetorar para o handler \
         (partiu de 0x{pc_antes:08X})"
    );
}
