use crate::bus::{Bus, BusRead};
use crate::cpu::Cpu;

pub fn load_psexe(exe_data: &[u8], bus: &mut Bus, cpu: &mut Cpu) -> Result<(), String> {
    if exe_data.len() < 0x800 {
        return Err("PS-EXE file too short for header".to_string());
    }
    if &exe_data[0..8] != b"PS-X EXE" {
        return Err("invalid PS-EXE magic".to_string());
    }

    let read_u32 = |offset: usize| -> u32 {
        u32::from_le_bytes(
            exe_data[offset..offset + 4]
                .try_into()
                .expect("index válido no header do PS-EXE"),
        )
    };

    let pc_init = read_u32(0x10);
    let _initial_gp = read_u32(0x14);
    let dest_addr = read_u32(0x18);
    let file_size = read_u32(0x1C) as usize;
    let bss_addr = read_u32(0x28);
    let bss_size = read_u32(0x2C);
    let sp_fp_base = read_u32(0x30);
    let sp_fp_offset = read_u32(0x34);

    let header_size = 0x800;
    let body_available = exe_data.len().saturating_sub(header_size);
    let load_size = file_size.min(body_available);

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
    cpu.regs[28] = _initial_gp;
    if sp_fp_base != 0 {
        let sp_val = sp_fp_base.wrapping_add(sp_fp_offset);
        cpu.regs[29] = sp_val;
        cpu.regs[30] = sp_val;
    }
    cpu.regs[5] = 0;
    cpu.regs[4] = 1;

    Ok(())
}
