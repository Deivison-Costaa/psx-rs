// Desmontador MIPS-I/R3000A minimo, so pra diagnostico manual via --disasm. Nao e usado
// pela execucao (crates/psx-core/src/cpu.rs tem seu proprio decode, independente deste).

const NOMES: [&str; 32] = [
    "zero", "at", "v0", "v1", "a0", "a1", "a2", "a3", "t0", "t1", "t2", "t3", "t4", "t5", "t6",
    "t7", "s0", "s1", "s2", "s3", "s4", "s5", "s6", "s7", "t8", "t9", "k0", "k1", "gp", "sp",
    "fp", "ra",
];

fn r(n: u32) -> &'static str {
    NOMES[(n & 0x1F) as usize]
}

fn imm16(instr: u32) -> i32 {
    (instr as i16) as i32
}

fn campos(instr: u32) -> (u32, u32, u32, u32, u32, u32, u32) {
    let opcode = instr >> 26;
    let rs = (instr >> 21) & 0x1F;
    let rt = (instr >> 16) & 0x1F;
    let rd = (instr >> 11) & 0x1F;
    let sa = (instr >> 6) & 0x1F;
    let funct = instr & 0x3F;
    let imm = instr & 0xFFFF;
    (opcode, rs, rt, rd, sa, funct, imm)
}

fn especial(instr: u32) -> String {
    let (_, rs, rt, rd, sa, funct, _) = campos(instr);
    match funct {
        0x00 => {
            if instr == 0 {
                "nop".to_string()
            } else {
                format!("sll   {}, {}, {}", r(rd), r(rt), sa)
            }
        }
        0x02 => format!("srl   {}, {}, {}", r(rd), r(rt), sa),
        0x03 => format!("sra   {}, {}, {}", r(rd), r(rt), sa),
        0x04 => format!("sllv  {}, {}, {}", r(rd), r(rt), r(rs)),
        0x06 => format!("srlv  {}, {}, {}", r(rd), r(rt), r(rs)),
        0x07 => format!("srav  {}, {}, {}", r(rd), r(rt), r(rs)),
        0x08 => format!("jr    {}", r(rs)),
        0x09 => format!("jalr  {}, {}", r(rd), r(rs)),
        0x0C => "syscall".to_string(),
        0x0D => "break".to_string(),
        0x10 => format!("mfhi  {}", r(rd)),
        0x11 => format!("mthi  {}", r(rs)),
        0x12 => format!("mflo  {}", r(rd)),
        0x13 => format!("mtlo  {}", r(rs)),
        0x18 => format!("mult  {}, {}", r(rs), r(rt)),
        0x19 => format!("multu {}, {}", r(rs), r(rt)),
        0x1A => format!("div   {}, {}", r(rs), r(rt)),
        0x1B => format!("divu  {}, {}", r(rs), r(rt)),
        0x20 => format!("add   {}, {}, {}", r(rd), r(rs), r(rt)),
        0x21 => format!("addu  {}, {}, {}", r(rd), r(rs), r(rt)),
        0x22 => format!("sub   {}, {}, {}", r(rd), r(rs), r(rt)),
        0x23 => format!("subu  {}, {}, {}", r(rd), r(rs), r(rt)),
        0x24 => format!("and   {}, {}, {}", r(rd), r(rs), r(rt)),
        0x25 => format!("or    {}, {}, {}", r(rd), r(rs), r(rt)),
        0x26 => format!("xor   {}, {}, {}", r(rd), r(rs), r(rt)),
        0x27 => format!("nor   {}, {}, {}", r(rd), r(rs), r(rt)),
        0x2A => format!("slt   {}, {}, {}", r(rd), r(rs), r(rt)),
        0x2B => format!("sltu  {}, {}, {}", r(rd), r(rs), r(rt)),
        _ => format!("special?0x{:02X}", funct),
    }
}

fn bcondz(instr: u32) -> String {
    let (_, rs, rt, _, _, _, _) = campos(instr);
    let off = imm16(instr) * 4;
    match rt {
        0x00 => format!("bltz  {}, {:+}", r(rs), off),
        0x01 => format!("bgez  {}, {:+}", r(rs), off),
        0x10 => format!("bltzal {}, {:+}", r(rs), off),
        0x11 => format!("bgezal {}, {:+}", r(rs), off),
        _ => "bcondz?".to_string(),
    }
}

fn cop(n: u32, instr: u32) -> String {
    let (_, rs, rt, rd, _, funct, _) = campos(instr);
    let unidade = match rs {
        0x00 => "mfc",
        0x02 => "cfc",
        0x04 => "mtc",
        0x06 => "ctc",
        0x08 => match rt {
            0x00 => return format!("bc{}f  {:+}", n, imm16(instr) * 4),
            0x01 => return format!("bc{}t  {:+}", n, imm16(instr) * 4),
            _ => "bc?",
        },
        0x10..=0x1F => {
            if n == 0 && funct == 0x10 {
                return "rfe".to_string();
            }
            return format!("cop{}  0x{:07X}", n, instr & 0x01FF_FFFF);
        }
        _ => "cop?",
    };
    format!("{}{}  {}, {}", unidade, n, r(rt), rd)
}

/// Desmonta uma instrucao MIPS-I crua. Cobre o subconjunto do R3000A que aparece em codigo
/// de jogo/BIOS comum; o que sobrar imprime o opcode/funct cru em vez de travar.
pub fn desmonta(instr: u32) -> String {
    if instr == 0 {
        return "nop".to_string();
    }
    let (opcode, rs, rt, _rd, _sa, _funct, imm) = campos(instr);
    let off = imm16(instr) * 4;
    match opcode {
        0x00 => especial(instr),
        0x01 => bcondz(instr),
        0x02 => format!("j     0x{:07X}", (instr & 0x03FF_FFFF) << 2),
        0x03 => format!("jal   0x{:07X}", (instr & 0x03FF_FFFF) << 2),
        0x04 => format!("beq   {}, {}, {:+}", r(rs), r(rt), off),
        0x05 => format!("bne   {}, {}, {:+}", r(rs), r(rt), off),
        0x06 => format!("blez  {}, {:+}", r(rs), off),
        0x07 => format!("bgtz  {}, {:+}", r(rs), off),
        0x08 => format!("addi  {}, {}, {}", r(rt), r(rs), imm16(instr)),
        0x09 => format!("addiu {}, {}, {}", r(rt), r(rs), imm16(instr)),
        0x0A => format!("slti  {}, {}, {}", r(rt), r(rs), imm16(instr)),
        0x0B => format!("sltiu {}, {}, {}", r(rt), r(rs), imm16(instr)),
        0x0C => format!("andi  {}, {}, 0x{:04X}", r(rt), r(rs), imm),
        0x0D => format!("ori   {}, {}, 0x{:04X}", r(rt), r(rs), imm),
        0x0E => format!("xori  {}, {}, 0x{:04X}", r(rt), r(rs), imm),
        0x0F => format!("lui   {}, 0x{:04X}", r(rt), imm),
        0x10 => cop(0, instr),
        0x12 => cop(2, instr),
        0x20 => format!("lb    {}, {}({})", r(rt), imm16(instr), r(rs)),
        0x21 => format!("lh    {}, {}({})", r(rt), imm16(instr), r(rs)),
        0x22 => format!("lwl   {}, {}({})", r(rt), imm16(instr), r(rs)),
        0x23 => format!("lw    {}, {}({})", r(rt), imm16(instr), r(rs)),
        0x24 => format!("lbu   {}, {}({})", r(rt), imm16(instr), r(rs)),
        0x25 => format!("lhu   {}, {}({})", r(rt), imm16(instr), r(rs)),
        0x26 => format!("lwr   {}, {}({})", r(rt), imm16(instr), r(rs)),
        0x28 => format!("sb    {}, {}({})", r(rt), imm16(instr), r(rs)),
        0x29 => format!("sh    {}, {}({})", r(rt), imm16(instr), r(rs)),
        0x2A => format!("swl   {}, {}({})", r(rt), imm16(instr), r(rs)),
        0x2B => format!("sw    {}, {}({})", r(rt), imm16(instr), r(rs)),
        0x2E => format!("swr   {}, {}({})", r(rt), imm16(instr), r(rs)),
        0x32 => format!("lwc2  {}, {}({})", rt, imm16(instr), r(rs)),
        0x3A => format!("swc2  {}, {}({})", rt, imm16(instr), r(rs)),
        _ => format!(".word 0x{:08X} (opcode?0x{:02X})", instr, opcode),
    }
}
