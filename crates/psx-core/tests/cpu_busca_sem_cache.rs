use psx_core::bus::{Bus, BusWrite};
use psx_core::cpu::Cpu;

mod support;
use support::asm::{bus_with_bios_empty, encode_i_type, nop};

fn com_atrasos_da_bios(bus: &mut Bus) {
    bus.write32::<BusWrite>(0x1F80_1010, 0x0013_243F);
    bus.write32::<BusWrite>(0x1F80_1020, 0x0003_1125);
}

fn ciclos_de_nops(bus: &mut Bus, pc: u32, n: u32) -> u64 {
    let mut cpu = Cpu::new();
    cpu.pc = pc;
    let antes = bus.total_cycles();
    for _ in 0..n {
        cpu.step(bus);
    }
    bus.total_cycles() - antes
}

#[test]
fn busca_na_rom_da_bios_sem_cache_paga_o_acesso_de_32_bits() {
    let mut bus = bus_with_bios_empty();
    com_atrasos_da_bios(&mut bus);
    assert_eq!(
        ciclos_de_nops(&mut bus, 0xBFC0_0000, 10),
        250,
        "KSEG1 nao tem i-cache: cada opcode le 4 bytes da ROM de 8 bits (access-time BIOS 32 bits: 24.94)"
    );
}

#[test]
fn busca_na_rom_segue_o_registrador_de_atraso() {
    let mut bus = bus_with_bios_empty();
    com_atrasos_da_bios(&mut bus);
    bus.write32::<BusWrite>(0x1F80_1010, 0x0013_247F);
    assert_eq!(
        ciclos_de_nops(&mut bus, 0xBFC0_0000, 1),
        11 + 3 * 10,
        "atraso 7: primeiro acesso 11, cada byte seguinte 7+2+COM2"
    );
}

#[test]
fn busca_em_kseg1_na_ram_paga_o_acesso_a_ram() {
    let mut bus = bus_with_bios_empty();
    for i in 0..8 {
        bus.write32::<BusWrite>(0x0000_0100 + 4 * i, nop());
    }
    assert_eq!(ciclos_de_nops(&mut bus, 0xA000_0100, 8), 8 * 5);
}

#[test]
fn busca_em_regiao_com_cache_custa_um_ciclo() {
    let mut bus = bus_with_bios_empty();
    com_atrasos_da_bios(&mut bus);
    assert_eq!(ciclos_de_nops(&mut bus, 0x9FC0_0000, 10), 10);
    assert_eq!(ciclos_de_nops(&mut bus, 0x8000_0100, 10), 10);
    assert_eq!(ciclos_de_nops(&mut bus, 0x0000_0100, 10), 10);
}

#[test]
fn load_na_rom_soma_a_busca_e_o_acesso() {
    let mut bus = bus_with_bios_empty();
    com_atrasos_da_bios(&mut bus);
    bus.write32::<BusWrite>(0x0000_0100, encode_i_type(0x24, 9, 8, 0));
    let mut cpu = Cpu::new();
    cpu.pc = 0xA000_0100;
    cpu.regs[8] = 0xBFC0_0000;
    let antes = bus.total_cycles();
    cpu.step(&mut bus);
    assert_eq!(
        bus.total_cycles() - antes,
        5 + 7 - 1,
        "busca em RAM sem cache (5) + lbu na ROM (7), sobrepondo o ciclo de emissao"
    );
}
