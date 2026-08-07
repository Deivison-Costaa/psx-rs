use psx_core::bus::BusRead;
use psx_core::cpu::Cpu;

mod support;
use support::asm::{addiu, bus_with_bios_empty, encode_i_type, nop};

const CODIGO: u32 = 0x0000_0100;
const RAM: u32 = 0x8000_2000;
const CU2: u32 = 1 << 30;

// § GTE Load Delay Slots (docs/reference/07-gte.md L101, regra em L112-114): "If an
// instruction that reads a GTE register or a GTE command is executed before the current
// GTE command is finished, the CPU will hold until the instruction has finished." So:
// MFC2/CFC2/SWC2 LEEM um registrador GTE -> esperam. MTC2/CTC2/LWC2 ESCREVEM -> a spec so
// fala de leitura, entao nao esperam (omissao declarada, nao medida).

fn gte_cmd(imm25: u32) -> u32 {
    (0x12 << 26) | (1 << 25) | (imm25 & 0x01FF_FFFF)
}
fn mfc2(rt: u32, rd: u32) -> u32 {
    (0x12 << 26) | (rt << 16) | (rd << 11)
}
fn cfc2(rt: u32, rd: u32) -> u32 {
    (0x12 << 26) | (0x02 << 21) | (rt << 16) | (rd << 11)
}
fn mtc2(rt: u32, rd: u32) -> u32 {
    (0x12 << 26) | (0x04 << 21) | (rt << 16) | (rd << 11)
}
fn lwc2(rt: u32, rs: u32, imm: u16) -> u32 {
    encode_i_type(0x32, rt, rs, imm)
}
fn swc2(rt: u32, rs: u32, imm: u16) -> u32 {
    encode_i_type(0x3A, rt, rs, imm)
}

const RTPS: u32 = 0x0180001; // 07-gte.md L481, 15 ciclos
const NCLIP: u32 = 0x1400006; // 07-gte.md L513, 8 ciclos
const NCDT: u32 = 0x0F80416; // 07-gte.md L597, 44 ciclos

fn total_de(corpo: &[u32]) -> u64 {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.pc = CODIGO;
    cpu.set_sr(CU2);
    cpu.regs[8] = RAM;
    for (i, instr) in corpo.iter().enumerate() {
        bus.write32::<BusRead>(CODIGO + (i as u32) * 4, *instr);
    }
    let antes = bus.total_cycles();
    for _ in 0..corpo.len() {
        cpu.step(&mut bus);
    }
    bus.total_cycles() - antes
}

#[test]
fn rtps_seguido_de_mfc2_custa_17() {
    assert_eq!(
        total_de(&[gte_cmd(RTPS), mfc2(1, 0)]),
        17,
        "RTPS (15) + mfc2 (2 = 1 emissao + 1 espera restante)? nao -- mfc2 espera os 15 \
         inteiros porque roda logo apos: 1 (rtps) + 1+15 (mfc2) = 17"
    );
}

#[test]
fn quinze_nops_cabem_de_graca_apos_rtps() {
    let mut corpo = vec![gte_cmd(RTPS)];
    corpo.extend(std::iter::repeat_n(nop(), 15));
    corpo.push(mfc2(1, 0));
    assert_eq!(
        total_de(&corpo),
        17,
        "15 nops cobrem os 15 ciclos de RTPS; mfc2 nao espera mais nada: 1+15+1=17"
    );
}

#[test]
fn o_decimo_sexto_nop_ja_custa() {
    let mut corpo = vec![gte_cmd(RTPS)];
    corpo.extend(std::iter::repeat_n(nop(), 16));
    corpo.push(mfc2(1, 0));
    assert_eq!(
        total_de(&corpo),
        18,
        "o 16o nop nao tem mais espera pra cobrir, so soma seu proprio ciclo: 1+16+1=18"
    );
}

#[test]
fn nclip_seguido_de_mfc2_custa_10() {
    assert_eq!(
        total_de(&[gte_cmd(NCLIP), mfc2(1, 0)]),
        10,
        "NCLIP=8: 1+(1+8)=10"
    );
}

#[test]
fn ncdt_seguido_de_mfc2_custa_46() {
    assert_eq!(
        total_de(&[gte_cmd(NCDT), mfc2(1, 0)]),
        46,
        "NCDT=44, o comando mais caro da tabela: 1+(1+44)=46"
    );
}

#[test]
fn comando_gte_espera_o_comando_anterior() {
    // A propria spec diz que emitir um NOVO comando GTE antes do anterior terminar tambem
    // espera -- nao so leitura de registrador.
    assert_eq!(
        total_de(&[gte_cmd(RTPS), gte_cmd(NCLIP)]),
        17,
        "NCLIP emitido logo apos RTPS espera os 15 ciclos de RTPS antes de sequer comecar: \
         1 (rtps) + 1+15 (nclip espera, so entao emite) = 17"
    );
}

#[test]
fn prazo_do_segundo_comando_conta_do_instante_em_que_ele_realmente_comecou() {
    // RTPS (15) trava NCLIP (segundo comando) por 15 -- NCLIP so comeca de verdade
    // depois dessa espera, entao seu proprio prazo tem que contar 1+15(espera)+8(custo)
    // a partir do total_cycles ANTES da espera, nao so 1+8. Um terceiro comando/leitura
    // prova a diferenca: se o prazo do NCLIP tivesse esquecido a espera ja empilhada,
    // ele teria terminado cedo demais e o mfc2 nao esperaria nada.
    assert_eq!(
        total_de(&[gte_cmd(RTPS), gte_cmd(NCLIP), mfc2(1, 0)]),
        26,
        "rtps custa 1 (total=1), nclip espera 15 do rtps (extra_cycles=15, custa 16, \
         total=17): seu prazo real e' total-antes-da-espera(1) + 1 + 15 + custo(8) = 25, \
         nao 1+1+8=10. mfc2 entao espera 25-17=8: 17+1+8=26"
    );
}

#[test]
fn swc2_espera_o_comando() {
    assert_eq!(
        total_de(&[gte_cmd(RTPS), swc2(1, 8, 0)]),
        17,
        "SWC2 le um registrador de dados do GTE -- espera como mfc2/cfc2"
    );
}

#[test]
fn cfc2_espera_o_comando() {
    assert_eq!(
        total_de(&[gte_cmd(RTPS), cfc2(1, 0)]),
        17,
        "CFC2 tambem le, tambem espera"
    );
}

#[test]
fn mfc2_sem_comando_pendente_custa_1() {
    assert_eq!(
        total_de(&[mfc2(1, 0)]),
        1,
        "controle: sem comando GTE pendente, mfc2 custa so o ciclo normal de emissao"
    );
}

#[test]
fn instrucao_alheia_nao_espera() {
    assert_eq!(
        total_de(&[gte_cmd(RTPS), addiu(2, 2, 1)]),
        2,
        "controle: addiu nao le nem escreve GTE, RTPS pendente nao afeta: 1+1=2"
    );
}

#[test]
fn mtc2_nao_espera() {
    assert_eq!(
        total_de(&[gte_cmd(RTPS), mtc2(9, 0)]),
        2,
        "controle: MTC2 escreve, a spec so fala de leitura travando a CPU -- nao espera"
    );
}

#[test]
fn lwc2_nao_espera_mas_ainda_paga_a_regiao_do_load() {
    assert_eq!(
        total_de(&[gte_cmd(RTPS), lwc2(1, 8, 0)]),
        8,
        "controle: LWC2 escreve um registrador GTE (nao espera o RTPS pendente), mas \
         continua pagando o custo de regiao do load (Degrau 2, RAM=7): 1+7=8"
    );
}
