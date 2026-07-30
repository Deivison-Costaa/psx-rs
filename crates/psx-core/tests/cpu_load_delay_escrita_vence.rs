use psx_core::bus::{Bios, Bus, BusRead, Ram};
use psx_core::cpu::Cpu;

fn bus_with_bios_empty() -> Bus {
    let ram = Ram::new();
    let bios_bytes = vec![0u8; 0x80000];
    let bios = Bios::from_bytes(bios_bytes).unwrap();
    Bus::new(ram, bios)
}

fn encode_special(secondary: u32, rd: u32, rt: u32, rs: u32) -> u32 {
    (rs << 21) | (rt << 16) | (rd << 11) | secondary
}

fn encode_load_store(primary: u32, rt: u32, rs: u32, imm: u16) -> u32 {
    (primary << 26) | (rs << 21) | (rt << 16) | (imm as u32)
}

fn encode_ori(rt: u32, rs: u32, imm: u16) -> u32 {
    (0x0D << 26) | (rs << 21) | (rt << 16) | (imm as u32)
}

fn encode_jal(target: u32) -> u32 {
    (0x03 << 26) | ((target >> 2) & 0x03FF_FFFF)
}

fn nop() -> u32 {
    encode_special(0x00, 0, 0, 0)
}

fn cpu_com_lw_em_zero(bus: &mut Bus, rt: u32) -> Cpu {
    bus.write32::<BusRead>(0x1000, 0xDEAD_BEEF);
    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = 0x1000;
    bus.write32::<BusRead>(0, encode_load_store(0x23, rt, 8, 0x0000));
    cpu
}

#[test]
fn escrita_alu_no_delay_slot_vence_o_load() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = cpu_com_lw_em_zero(&mut bus, 10);
    bus.write32::<BusRead>(4, encode_ori(10, 0, 0x0005));
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(
        cpu.regs[10], 0x0000_0005,
        "a escrita do delay slot vence: o valor do load e descartado \
         (padrao da BIOS SCPH1001 em 0x8004723C-40, medido na iter 0111)"
    );
}

#[test]
fn jal_no_delay_slot_de_lw_ra_mantem_o_link() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = cpu_com_lw_em_zero(&mut bus, 31);
    bus.write32::<BusRead>(4, encode_jal(0x100));
    bus.write32::<BusRead>(8, nop());
    bus.write32::<BusRead>(0x100, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(
        cpu.regs[31], 0x0000_000C,
        "jal em load delay slot de lw $ra: o link (PC+8) vence o load"
    );
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 0x100, "o salto do jal acontece normalmente");
}

#[test]
fn delay_slot_le_valor_velho_e_a_propria_escrita_vence() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = cpu_com_lw_em_zero(&mut bus, 10);
    cpu.regs[10] = 0x0000_0007;
    cpu.regs[11] = 0x0000_0001;
    bus.write32::<BusRead>(4, encode_special(0x21, 10, 11, 10));
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(
        cpu.regs[10], 0x0000_0008,
        "addu r10,r10,r11 no delay slot: le o r10 VELHO (7) e a soma (8) vence o load"
    );
}

#[test]
fn escrita_em_outro_registrador_nao_cancela_o_load() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = cpu_com_lw_em_zero(&mut bus, 10);
    bus.write32::<BusRead>(4, encode_ori(12, 0, 0x0005));
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.regs[10], 0xDEAD_BEEF, "load completa normalmente");
    assert_eq!(cpu.regs[12], 0x0000_0005, "escrita paralela intacta");
}

#[test]
fn lw_para_ra_sem_conflito_completa_normalmente() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = cpu_com_lw_em_zero(&mut bus, 31);
    bus.write32::<BusRead>(4, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(
        cpu.regs[31], 0xDEAD_BEEF,
        "lw $ra com nop no delay slot: o load completa (nada a cancelar)"
    );
}

#[test]
fn load_para_o_mesmo_registrador_no_delay_slot_nao_e_cancelado() {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(0x1004, 0x1234_5678);
    let mut cpu = cpu_com_lw_em_zero(&mut bus, 10);
    bus.write32::<BusRead>(4, encode_load_store(0x23, 10, 8, 0x0004));
    bus.write32::<BusRead>(8, nop());
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(
        cpu.regs[10], 0x1234_5678,
        "lw;lw no mesmo registrador: o segundo load prevalece no fim"
    );
}
