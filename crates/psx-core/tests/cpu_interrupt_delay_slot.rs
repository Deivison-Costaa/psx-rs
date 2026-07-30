use psx_core::bus::{Bus, BusRead};
use psx_core::cpu::Cpu;

mod support;
use support::asm::{addiu, bus_with_bios_empty, encode_j_type, nop};

const VETOR: u32 = 0x8000_0080;
const BRANCH: u32 = 0x0000_0100;
const DELAY: u32 = 0x0000_0104;
const ALVO: u32 = 0x0000_0200;

fn armar_irq(bus: &mut Bus, cpu: &mut Cpu) {
    bus.irq_mut().write_mask(0x0001);
    bus.irq_mut().raise(0);
    cpu.cop0[12] = 0x0000_0401;
}

fn cenario_salto_com_delay_slot() -> (Bus, Cpu) {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = BRANCH;
    bus.write32::<BusRead>(BRANCH, encode_j_type(0x02, ALVO >> 2));
    bus.write32::<BusRead>(DELAY, addiu(29, 29, 0x0060));
    bus.write32::<BusRead>(ALVO, nop());
    cpu.regs[29] = 0x801F_FB58;
    (bus, cpu)
}

#[test]
fn interrupcao_no_delay_slot_aponta_epc_para_o_branch() {
    let (mut bus, mut cpu) = cenario_salto_com_delay_slot();
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, DELAY, "o passo do branch deixa o PC no delay slot");

    armar_irq(&mut bus, &mut cpu);
    cpu.step(&mut bus);

    assert_eq!(cpu.pc, VETOR, "interrupcao vetora para 0x80000080");
    assert_eq!(
        cpu.cop0[14], BRANCH,
        "EPC tem de apontar para o BRANCH, nao para o delay slot: retornar ao delay slot \
         executaria a instrucao dele sem refazer o salto. docs/reference/02-cpu.md L683."
    );
    assert_ne!(
        cpu.cop0[13] & (1 << 31),
        0,
        "CAUSE.BD tem de estar setado quando EPC aponta para o branch (docs/reference/02-cpu.md L683)"
    );
}

#[test]
fn interrupcao_no_delay_slot_nao_executa_a_instrucao_do_slot() {
    let (mut bus, mut cpu) = cenario_salto_com_delay_slot();
    cpu.step(&mut bus);

    armar_irq(&mut bus, &mut cpu);
    let sp_antes = cpu.regs[29];
    cpu.step(&mut bus);

    assert_eq!(
        cpu.regs[29], sp_antes,
        "a instrucao do delay slot nao pode ter efeito quando a interrupcao a preempta: se ela \
         rodar aqui E de novo depois do retorno pelo EPC, o efeito e aplicado duas vezes; se \
         rodar so aqui e o EPC apontar para o branch, tudo bem — o que nao pode e sumir. Este \
         teste fixa a metade 'nao rodou agora'; o par dele e o teste do retorno."
    );
}

#[test]
fn handler_nao_e_sequestrado_pelo_salto_pendente() {
    let (mut bus, mut cpu) = cenario_salto_com_delay_slot();
    cpu.step(&mut bus);

    armar_irq(&mut bus, &mut cpu);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, VETOR);
    bus.irq_mut().write_mask(0x0000);

    bus.write32::<BusRead>(VETOR, nop());
    bus.write32::<BusRead>(VETOR + 4, nop());
    cpu.step(&mut bus);
    assert_eq!(
        cpu.pc,
        VETOR + 4,
        "depois de vetorar, o PC segue DENTRO do handler. O salto pendente do delay slot tem de \
         ser descartado: se sobreviver, ele sequestra o PC na primeira instrucao do handler e o \
         handler nunca roda — foi o defeito que matou o boot no passo 26 595 832, com o handler \
         durando exatamente uma instrucao."
    );

    cpu.step(&mut bus);
    assert_eq!(cpu.pc, VETOR + 8, "e continua dentro do handler no passo seguinte");
}

#[test]
fn retorno_pelo_epc_refaz_o_salto_e_o_delay_slot() {
    let (mut bus, mut cpu) = cenario_salto_com_delay_slot();
    cpu.step(&mut bus);

    armar_irq(&mut bus, &mut cpu);
    cpu.step(&mut bus);
    bus.irq_mut().write_mask(0x0000);

    let epc = cpu.cop0[14];
    cpu.pc = epc;
    let sp_antes = cpu.regs[29];
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        cpu.regs[29],
        sp_antes.wrapping_add(0x60),
        "voltando pelo EPC, o delay slot roda exatamente uma vez"
    );
    assert_eq!(cpu.pc, ALVO, "e o salto chega ao alvo");
}

#[test]
fn interrupcao_fora_de_delay_slot_nao_marca_bd() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = BRANCH;
    bus.write32::<BusRead>(BRANCH, nop());

    armar_irq(&mut bus, &mut cpu);
    cpu.step(&mut bus);

    assert_eq!(cpu.pc, VETOR);
    assert_eq!(cpu.cop0[14], BRANCH, "fora de delay slot, EPC e o proprio PC");
    assert_eq!(
        cpu.cop0[13] & (1 << 31),
        0,
        "CAUSE.BD so pode estar setado quando EPC foi recuado para o branch"
    );
}

#[test]
fn bt_indica_branch_tomado_no_delay_slot() {
    let (mut bus, mut cpu) = cenario_salto_com_delay_slot();
    cpu.step(&mut bus);

    armar_irq(&mut bus, &mut cpu);
    cpu.step(&mut bus);

    assert_ne!(
        cpu.cop0[13] & (1 << 30),
        0,
        "BT indica que o branch seria tomado; o salto incondicional sempre e \
         (docs/reference/02-cpu.md L682)"
    );
}
