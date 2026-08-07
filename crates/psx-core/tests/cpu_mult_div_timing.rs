use psx_core::bus::BusRead;
use psx_core::cpu::Cpu;

mod support;
use support::asm::{addiu, bus_with_bios_empty, encode_special, nop};

const CODIGO: u32 = 0x0000_0100;

fn multu(rs: u32, rt: u32) -> u32 {
    encode_special(0x19, 0, rt, rs)
}
fn mult(rs: u32, rt: u32) -> u32 {
    encode_special(0x18, 0, rt, rs)
}
fn divu(rs: u32, rt: u32) -> u32 {
    encode_special(0x1B, 0, rt, rs)
}
fn div(rs: u32, rt: u32) -> u32 {
    encode_special(0x1A, 0, rt, rs)
}
fn mflo(rd: u32) -> u32 {
    encode_special(0x12, rd, 0, 0)
}
fn mfhi(rd: u32) -> u32 {
    encode_special(0x10, rd, 0, 0)
}
fn mthi(rs: u32) -> u32 {
    encode_special(0x11, 0, 0, rs)
}

// Executa `corpo` inteiro a partir de CODIGO, com $8=rs_val e $9=1 (operando nao-zero, so
// pra nao cair no caso especial de divisao por zero -- o CUSTO de div/divu e fixo
// independente dos operandos). Devolve o total de ciclos da sequencia inteira.
fn total_de(rs_val: u32, corpo: &[u32]) -> u64 {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = CODIGO;
    cpu.regs[8] = rs_val;
    cpu.regs[9] = 1;
    for (i, instr) in corpo.iter().enumerate() {
        bus.write32::<BusRead>(CODIGO + (i as u32) * 4, *instr);
    }
    let antes = bus.total_cycles();
    for _ in 0..corpo.len() {
        cpu.step(&mut bus);
    }
    bus.total_cycles() - antes
}

// § MULT/DIV timing (docs/reference/02-cpu.md L420-436): iniciar custa 1 ciclo (ja cobrado
// como qualquer instrucao); ler HI/LO antes do fim do calculo trava a CPU pelo resto do
// custo. multu/mult tem 3 faixas por valor de rs (6/9/13 ciclos); div/divu sao fixos em 36.

#[test]
fn multu_rapido_seguido_de_mflo_custa_8() {
    assert_eq!(
        total_de(0x0000_0100, &[multu(8, 9), mflo(1)]),
        8,
        "rs=100h esta na faixa Fast (6 ciclos): 1 (multu) + 1+6 (mflo espera o resto) = 8"
    );
}

#[test]
fn seis_nops_cabem_de_graca_entre_multu_e_mflo() {
    assert_eq!(
        total_de(
            0x0000_0100,
            &[
                multu(8, 9),
                nop(),
                nop(),
                nop(),
                nop(),
                nop(),
                nop(),
                mflo(1)
            ]
        ),
        8,
        "L437-440: 'one can insert up to six (cached) ALU opcodes... between multu and mflo \
         without additional slowdown' -- 6 nops cobrem os 6 ciclos Fast, mflo nao espera mais"
    );
}

#[test]
fn o_setimo_nop_ja_custa() {
    assert_eq!(
        total_de(
            0x0000_0100,
            &[
                multu(8, 9),
                nop(),
                nop(),
                nop(),
                nop(),
                nop(),
                nop(),
                nop(),
                mflo(1)
            ]
        ),
        9,
        "7 nops: o 6o ja tinha esgotado a espera, o 7o so soma seu proprio ciclo normal"
    );
}

#[test]
fn multu_medio_custa_9() {
    assert_eq!(
        total_de(0x0000_0800, &[multu(8, 9), mflo(1)]),
        11,
        "rs=800h esta na faixa Med (9 ciclos): 1 + 1+9 = 11"
    );
}

#[test]
fn multu_com_valor_grande_e_sem_sinal_custa_13() {
    // rs=FFFFFF00h interpretado SEM sinal (multu) e um numero enorme -- faixa Slow (13).
    // Com sinal (mult) o mesmo bit pattern seria -256, faixa Fast (6) -- a diferenca entre
    // as duas tabelas so aparece com um valor assim, negativo em complemento de dois.
    assert_eq!(
        total_de(0xFFFF_FF00, &[multu(8, 9), mflo(1)]),
        15,
        "rs=FFFFFF00h sem sinal esta na faixa Slow (13 ciclos): 1 + 1+13 = 15"
    );
}

#[test]
fn multu_lento_custa_13() {
    assert_eq!(
        total_de(0x0010_0000, &[multu(8, 9), mflo(1)]),
        15,
        "rs=100000h esta na faixa Slow (13 ciclos): 1 + 1+13 = 15"
    );
}

#[test]
fn mult_negativo_rapido() {
    assert_eq!(
        total_de(0xFFFF_FF00, &[mult(8, 9), mflo(1)]),
        8,
        "mult com sinal: rs=FFFFFF00h esta na faixa Fast negativa (FFFFF800h..FFFFFFFFh)"
    );
}

#[test]
fn mult_negativo_medio() {
    assert_eq!(
        total_de(0xFFF0_0000, &[mult(8, 9), mflo(1)]),
        11,
        "mult com sinal: rs=FFF00000h esta na faixa Med negativa (FFF00000h..FFFFF7FFh -- a \
         spec imprime o teto como FFFFF801h, mas isso sobrepoe 2 valores com a faixa Fast \
         que comeca em FFFFF800h; resolvido a favor da faixa Fast, mais apertada)"
    );
}

#[test]
fn mult_lento() {
    assert_eq!(
        total_de(0x0010_0000, &[mult(8, 9), mflo(1)]),
        15,
        "mult com sinal: rs=100000h (positivo) esta na faixa Slow (100000h..7FFFFFFFh)"
    );
}

#[test]
fn divu_custa_36_fixo() {
    assert_eq!(
        total_de(0x1234_5678, &[divu(8, 9), mfhi(1)]),
        38,
        "L432: divu e fixo em 36 ciclos, sem faixa por operando: 1 + 1+36 = 38"
    );
}

#[test]
fn div_custa_36_fixo_com_operandos_diferentes() {
    assert_eq!(
        total_de(0xFFFF_0001, &[div(8, 9), mfhi(1)]),
        38,
        "mesmo fixo em 36 com operandos completamente diferentes (com sinal, negativo)"
    );
}

#[test]
fn mfhi_sem_mult_pendente_custa_1() {
    assert_eq!(
        total_de(0, &[mfhi(1)]),
        1,
        "controle: sem mult/div pendente, mfhi custa so o ciclo normal de emissao"
    );
}

#[test]
fn instrucao_que_nao_le_hi_lo_nao_espera() {
    assert_eq!(
        total_de(0x0010_0000, &[multu(8, 9), addiu(2, 2, 1)]),
        2,
        "controle: uma instrucao que nao toca HI/LO nao e afetada pelo multu pendente, \
         mesmo na faixa Slow (13 ciclos) -- 1 (multu) + 1 (addiu) = 2"
    );
}

#[test]
fn mthi_nao_espera() {
    assert_eq!(
        total_de(0x0010_0000, &[multu(8, 9), mthi(9)]),
        2,
        "controle: a spec (L420-436) so fala de LER hi/lo travar a CPU; escrever (mthi/mtlo) \
         nao esta documentado como espera, entao nao espera"
    );
}
