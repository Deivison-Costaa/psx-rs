use crate::bus::{Bus, BusRead};
use crate::cpu::Cpu;

pub fn load_psexe(exe_data: &[u8], bus: &mut Bus, cpu: &mut Cpu) -> Result<(), String> {
    if exe_data.len() < 0x800 {
        return Err("PS-EXE file too short for header".to_string());
    }
    if &exe_data[0..8] != b"PS-X EXE" {
        return Err("invalid PS-EXE magic".to_string());
    }

    let read_u32 = |offset: usize| -> Result<u32, String> {
        let slice = exe_data
            .get(offset..offset + 4)
            .ok_or_else(|| "PS-EXE header truncated".to_string())?;
        Ok(u32::from_le_bytes(
            slice
                .try_into()
                .map_err(|_| "PS-EXE header truncated".to_string())?,
        ))
    };

    let pc_init = read_u32(0x10)?;
    let gp_init = read_u32(0x14)?;
    let dest_addr = read_u32(0x18)?;
    let file_size = read_u32(0x1C)? as usize;
    let bss_addr = read_u32(0x28)?;
    let bss_size = read_u32(0x2C)?;
    let sp_fp_base = read_u32(0x30)?;
    let sp_fp_offset = read_u32(0x34)?;

    let header_size = 0x800;
    let body_available = exe_data.len().saturating_sub(header_size);
    let load_size = file_size.min(body_available);

    if load_size % 4 != 0 {
        return Err("PS-EXE body size not multiple of 4".to_string());
    }

    for i in (0..load_size).step_by(4) {
        let word = u32::from_le_bytes([
            exe_data[header_size + i],
            exe_data[header_size + i + 1],
            exe_data[header_size + i + 2],
            exe_data[header_size + i + 3],
        ]);
        bus.write32::<BusRead>(dest_addr.wrapping_add(i as u32), word);
    }

    if bss_size > 0 {
        for i in (0..bss_size).step_by(4) {
            bus.write32::<BusRead>(bss_addr.wrapping_add(i), 0);
        }
    }

    cpu.pc = pc_init;
    cpu.regs[28] = gp_init;
    if sp_fp_base != 0 {
        let sp_val = sp_fp_base.wrapping_add(sp_fp_offset);
        cpu.regs[29] = sp_val;
        cpu.regs[30] = sp_val;
    }
    cpu.regs[5] = 0;
    cpu.regs[4] = 1;
    cpu.set_sr(0x0000_1001);

    Ok(())
}

pub fn install_return_stubs(bus: &mut Bus) {
    let jr_ra: u32 = (31u32 << 21) | 0x08;
    let nop: u32 = 0x0000_0000;
    for &addr in &[0x0000_00A0u32, 0x0000_00B0u32, 0x0000_00C0u32] {
        bus.write32::<BusRead>(addr, jr_ra);
        bus.write32::<BusRead>(addr.wrapping_add(4), nop);
    }
    bus.write32::<BusRead>(0x8000_0080, 0x3C081F80);
    bus.write32::<BusRead>(0x8000_0084, 0x35091070);
    bus.write32::<BusRead>(0x8000_0088, 0x8D280000);
    bus.write32::<BusRead>(0x8000_008C, 0xAD280000);
    bus.write32::<BusRead>(0x8000_0090, 0x42000010);
    bus.write32::<BusRead>(0x8000_0094, 0x00000000);
}
