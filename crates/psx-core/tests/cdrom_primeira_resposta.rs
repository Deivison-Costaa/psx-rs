mod support;

use psx_core::bus::{Bus, BusRead, BusWrite};
use support::asm;

const CD_BASE: u32 = 0x1F80_1800;
const I_STAT: u32 = 0x1F80_1070;
const IRQ2: u32 = 1 << 2;

const PRIMEIRA_RESPOSTA: u64 = 0xC4E1;
const PRIMEIRA_RESPOSTA_INIT: u64 = 0x13CCE;

fn bus() -> Bus {
    asm::bus_with_bios_empty()
}

fn cd_read(bus: &Bus, offset: u32) -> u8 {
    bus.read8::<BusRead>(CD_BASE + offset)
}

fn cd_write(bus: &mut Bus, offset: u32, val: u8) {
    bus.write8::<BusWrite>(CD_BASE + offset, val);
}

fn set_bank(bus: &mut Bus, b: u8) {
    cd_write(bus, 0, b);
}

fn manda_comando(bus: &mut Bus, cmd: u8) {
    set_bank(bus, 0);
    cd_write(bus, 1, cmd);
}

fn empilha_param(bus: &mut Bus, val: u8) {
    set_bank(bus, 0);
    cd_write(bus, 2, val);
}

fn hintsts(bus: &mut Bus) -> u8 {
    set_bank(bus, 1);
    let val = cd_read(bus, 3) & 0x7;
    set_bank(bus, 0);
    val
}

fn intmsk_write(bus: &mut Bus, val: u8) {
    set_bank(bus, 1);
    cd_write(bus, 2, val);
    set_bank(bus, 0);
}

fn hsts(bus: &Bus) -> u8 {
    cd_read(bus, 0)
}

fn result_read(bus: &mut Bus) -> u8 {
    set_bank(bus, 0);
    cd_read(bus, 1)
}

fn i_stat_irq2(bus: &Bus) -> u32 {
    bus.read32::<BusRead>(I_STAT) & IRQ2
}

fn avanca(bus: &mut Bus, ciclos: u64) {
    bus.tick_timers(ciclos as u32);
}

#[test]
fn nenhuma_resposta_na_mesma_instrucao_da_escrita_do_comando() {
    let mut bus = bus();
    intmsk_write(&mut bus, 0x1F);

    manda_comando(&mut bus, 0x01);

    assert_eq!(
        hintsts(&mut bus),
        0,
        "spec § First Response: a primeira resposta leva dezenas de milhares de ciclos; \
         entregar HINTSTS dentro da escrita no porto pre-empta o driver que acabou de escrever"
    );
    assert_eq!(
        hsts(&bus) & (1 << 5),
        0,
        "RSLRRDY: a result FIFO nao pode ter resposta antes do prazo"
    );
    assert_eq!(
        i_stat_irq2(&bus),
        0,
        "IRQ2 nao pode subir na instrucao da escrita do comando"
    );
}

#[test]
fn nenhuma_resposta_um_ciclo_antes_do_prazo_da_spec() {
    let mut bus = bus();
    intmsk_write(&mut bus, 0x1F);

    manda_comando(&mut bus, 0x01);
    avanca(&mut bus, PRIMEIRA_RESPOSTA - 1);

    assert_eq!(
        hintsts(&mut bus),
        0,
        "spec § First Response: Nop (normal) tem media 000c4e1h; um ciclo antes ainda e cedo"
    );
    assert_eq!(
        i_stat_irq2(&bus),
        0,
        "sem resposta entregue nao ha borda: IRQ2 continua baixo"
    );
}

#[test]
fn resposta_chega_no_prazo_medio_da_spec() {
    let mut bus = bus();
    intmsk_write(&mut bus, 0x1F);

    manda_comando(&mut bus, 0x01);
    avanca(&mut bus, PRIMEIRA_RESPOSTA);

    assert_eq!(
        hintsts(&mut bus),
        3,
        "spec § First Response: em 000c4e1h ciclos o GetStat responde INT3"
    );
    assert_eq!(
        i_stat_irq2(&bus),
        IRQ2,
        "com HINTMSK=1Fh a entrega levanta IRQ2 pela borda"
    );
    assert_eq!(
        result_read(&mut bus),
        0x00,
        "sem disco o stat byte do GetStat e zero"
    );
}

#[test]
fn init_espera_o_atraso_maior_da_spec() {
    let mut bus = bus();
    intmsk_write(&mut bus, 0x1F);

    manda_comando(&mut bus, 0x0A);
    avanca(&mut bus, PRIMEIRA_RESPOSTA);

    assert_eq!(
        hintsts(&mut bus),
        0,
        "spec § First Response: o Init faz inicializacao antes da 1a resposta e leva 0013cceh, \
         nao os 000c4e1h dos demais comandos"
    );

    avanca(&mut bus, PRIMEIRA_RESPOSTA_INIT - PRIMEIRA_RESPOSTA);

    assert_eq!(
        hintsts(&mut bus),
        3,
        "spec § First Response: em 0013cceh ciclos o Init responde INT3"
    );
}

#[test]
fn busysts_nao_fica_preso_alto_durante_a_janela() {
    let mut bus = bus();

    manda_comando(&mut bus, 0x01);
    avanca(&mut bus, PRIMEIRA_RESPOSTA / 2);

    assert_eq!(
        hsts(&bus) & (1 << 7),
        0,
        "spec § SUB-CPU Mainloop: o busy flag cai no passo 4 e a IRQ so vem no passo 5, \
         'around 1000-6000 cycles later'; segurar BUSYSTS pela janela inteira estoura os \
         lacos de espera da BIOS, que tem orcamento de 0x8000 giros"
    );
}

#[test]
fn a_entrega_e_evento_do_scheduler_e_nao_efeito_da_escrita() {
    let mut bus = bus();
    let antes = bus.scheduler_pending_count();

    manda_comando(&mut bus, 0x01);

    assert_eq!(
        bus.scheduler_pending_count(),
        antes + 1,
        "R2: a resposta do dispositivo e um evento agendado, nao efeito colateral da escrita \
         no porto"
    );

    avanca(&mut bus, PRIMEIRA_RESPOSTA);

    assert_eq!(
        bus.scheduler_pending_count(),
        antes,
        "o evento sai da fila quando vence"
    );
}

#[test]
fn parametros_empilhados_sobrevivem_a_janela_de_espera() {
    let mut bus = bus();

    empilha_param(&mut bus, 0x20);
    manda_comando(&mut bus, 0x19);

    assert_eq!(
        hsts(&bus) & (1 << 5),
        0,
        "a result FIFO segue vazia enquanto a resposta nao vence"
    );

    avanca(&mut bus, PRIMEIRA_RESPOSTA);

    assert_eq!(hintsts(&mut bus), 3, "Test 20h responde INT3 no prazo");
    assert_eq!(result_read(&mut bus), 0x97, "byte 0 da data de fabricacao");
    assert_eq!(result_read(&mut bus), 0x01, "byte 1 da data de fabricacao");
    assert_eq!(result_read(&mut bus), 0x10, "byte 2 da data de fabricacao");
    assert_eq!(result_read(&mut bus), 0xC2, "versao do controlador");
}

#[test]
fn comando_novo_na_janela_nao_faz_o_evento_velho_entregar_cedo() {
    let mut bus = bus();
    let atraso_do_segundo: u64 = 1_000;

    manda_comando(&mut bus, 0x01);
    avanca(&mut bus, atraso_do_segundo);
    manda_comando(&mut bus, 0x01);

    avanca(&mut bus, PRIMEIRA_RESPOSTA - atraso_do_segundo);

    assert_eq!(
        hintsts(&mut bus),
        0,
        "o scheduler nao cancela eventos: o evento do primeiro comando vence antes do prazo \
         do segundo e tem de ser no-op"
    );

    avanca(&mut bus, atraso_do_segundo);

    assert_eq!(
        hintsts(&mut bus),
        3,
        "o segundo comando responde no proprio prazo, contado da sua escrita"
    );
}
