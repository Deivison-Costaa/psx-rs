mod support;

use psx_core::bus::{Bus, BusRead, BusWrite};
use psx_core::cpu::Cpu;
use support::asm;

const VETOR_GERAL: u32 = 0x8000_0080;
const EXCODE_ADEL: u32 = 0x04;
const BASE: u32 = 0x0000_1000;

fn jr(rs: u32) -> u32 {
    asm::encode_special(0x08, 0, 0, rs)
}

fn jalr(rd: u32, rs: u32) -> u32 {
    asm::encode_special(0x09, rd, 0, rs)
}

fn lui(rt: u32, imm: u16) -> u32 {
    asm::encode_i_type(0x0F, rt, 0, imm)
}

fn ori(rt: u32, rs: u32, imm: u16) -> u32 {
    asm::encode_i_type(0x0D, rt, rs, imm)
}

fn excode(cpu: &Cpu) -> u32 {
    (cpu.cop0[13] >> 2) & 0x1F
}

/// Monta `lui/ori` do alvo em $t0 e um `jr`/`jalr` para ele, e roda ate vetorizar.
fn salta_para(destino: u32, usa_jalr: bool) -> Cpu {
    let mut bus = asm::bus_with_bios_empty();
    let mut cpu = Cpu::new();

    let salto = if usa_jalr { jalr(31, 8) } else { jr(8) };
    let programa = [
        lui(8, (destino >> 16) as u16),
        ori(8, 8, (destino & 0xFFFF) as u16),
        salto,
        asm::nop(),
    ];
    for (i, palavra) in programa.iter().enumerate() {
        bus.write32::<BusWrite>(BASE + (i as u32) * 4, *palavra);
    }

    cpu.pc = BASE;
    for _ in 0..8 {
        cpu.step(&mut bus);
        if cpu.pc == VETOR_GERAL {
            break;
        }
    }
    cpu
}

#[test]
fn jr_para_endereco_desalinhado_levanta_adel() {
    let cpu = salta_para(0x0000_2001, false);

    assert_eq!(cpu.pc, VETOR_GERAL, "a excecao tem de vetorizar");
    assert_eq!(
        excode(&cpu),
        EXCODE_ADEL,
        "buscar opcode em endereco desalinhado e AdEL (04h), nao AdES (05h)"
    );
}

#[test]
fn jalr_para_endereco_desalinhado_levanta_adel() {
    let cpu = salta_para(0x0000_2002, true);

    assert_eq!(cpu.pc, VETOR_GERAL);
    assert_eq!(excode(&cpu), EXCODE_ADEL);
}

#[test]
fn o_endereco_do_fetch_vai_para_badvaddr_e_epc() {
    let destino = 0x0000_2003;
    let cpu = salta_para(destino, false);

    assert_eq!(
        cpu.cop0[8], destino,
        "BadVaddr recebe o endereco que falhou, e so erro de endereco o atualiza"
    );
    assert_eq!(
        cpu.cop0[14], destino,
        "EPC aponta para o fetch que falhou, nao para o salto"
    );
}

#[test]
fn o_fetch_que_falha_nao_esta_em_delay_slot() {
    let cpu = salta_para(0x0000_2001, false);

    assert_eq!(cpu.pc, VETOR_GERAL, "pre-condicao: a excecao aconteceu");
    assert_eq!(
        cpu.cop0[13] & (1 << 31),
        0,
        "o delay slot do salto ja executou; o fetch do alvo nao e delay slot"
    );
}

#[test]
fn jalr_grava_o_retorno_antes_de_falhar() {
    let cpu = salta_para(0x0000_2002, true);

    assert_eq!(cpu.pc, VETOR_GERAL, "pre-condicao: a excecao aconteceu");
    assert_eq!(
        cpu.regs[31],
        BASE + 16,
        "o link do jalr acontece na execucao do salto, antes do fetch que falha"
    );
}

#[test]
fn salto_para_endereco_alinhado_nao_levanta_excecao() {
    let mut bus = asm::bus_with_bios_empty();
    let mut cpu = Cpu::new();
    let destino = 0x0000_2000u32;
    bus.write32::<BusWrite>(destino, asm::nop());

    let programa = [
        lui(8, (destino >> 16) as u16),
        ori(8, 8, (destino & 0xFFFF) as u16),
        jr(8),
        asm::nop(),
    ];
    for (i, palavra) in programa.iter().enumerate() {
        bus.write32::<BusWrite>(BASE + (i as u32) * 4, *palavra);
    }

    cpu.pc = BASE;
    for _ in 0..5 {
        cpu.step(&mut bus);
    }

    assert_ne!(cpu.pc, VETOR_GERAL, "alvo alinhado nao vetoriza");
    assert_eq!(bus.read32::<BusRead>(destino), asm::nop());
}
