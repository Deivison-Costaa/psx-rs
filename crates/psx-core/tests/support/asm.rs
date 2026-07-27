use psx_core::bus::{Bios, Bus, Ram};

pub fn bus_with_bios_empty() -> Bus {
    let ram = Ram::new();
    let bios_bytes = vec![0u8; 0x80000];
    let bios = Bios::from_bytes(bios_bytes).expect("BIOS de teste");
    Bus::new(ram, bios)
}

pub fn encode_special(secondary: u32, rd: u32, rt: u32, rs: u32) -> u32 {
    (rs << 21) | (rt << 16) | (rd << 11) | secondary
}

pub fn encode_i_type(primary: u32, rt: u32, rs: u32, imm: u16) -> u32 {
    (primary << 26) | (rs << 21) | (rt << 16) | (imm as u32)
}

pub fn encode_j_type(opcode: u32, target: u32) -> u32 {
    (opcode << 26) | (target & 0x03FF_FFFF)
}

pub fn nop() -> u32 {
    encode_special(0x00, 0, 0, 0)
}

pub fn addiu(rt: u32, rs: u32, imm: u16) -> u32 {
    encode_i_type(0x09, rt, rs, imm)
}
