use crate::bus::Bus;
use crate::cpu::Cpu;

pub fn load_psexe(_exe_data: &[u8], _bus: &mut Bus, _cpu: &mut Cpu) -> Result<(), String> {
    Ok(())
}
