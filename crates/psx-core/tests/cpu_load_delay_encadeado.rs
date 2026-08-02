use psx_core::bus::BusRead;
use psx_core::cpu::Cpu;

mod support;
use support::asm::*;

const BASE: u32 = 0x1000;
const DEST: u32 = 10;
const OBSERVER: u32 = 11;
const INITIAL: u32 = 0xCAFE_BABE;

#[derive(Clone, Copy, Debug)]
enum LoadKind {
    Lb,
    Lbu,
    Lh,
    Lhu,
    Lw,
    Lwl,
    Lwr,
}

impl LoadKind {
    fn opcode(self) -> u32 {
        match self {
            Self::Lb => 0x20,
            Self::Lbu => 0x24,
            Self::Lh => 0x21,
            Self::Lhu => 0x25,
            Self::Lw => 0x23,
            Self::Lwl => 0x22,
            Self::Lwr => 0x26,
        }
    }

    fn offset(self, second: bool) -> u16 {
        match (self, second) {
            (Self::Lwl, false) => 0,
            (Self::Lwl, true) => 4,
            (Self::Lwr, false) => 3,
            (Self::Lwr, true) => 7,
            (_, false) => 0,
            (_, true) => 4,
        }
    }
}

const LOADS: [LoadKind; 7] = [
    LoadKind::Lb,
    LoadKind::Lbu,
    LoadKind::Lh,
    LoadKind::Lhu,
    LoadKind::Lw,
    LoadKind::Lwl,
    LoadKind::Lwr,
];

fn run_chain(first: LoadKind, second: LoadKind) -> u32 {
    let mut bus = bus_with_bios_empty();
    bus.write32::<BusRead>(BASE, 0x89AB_BA98);
    bus.write32::<BusRead>(BASE + 4, 0x7654_3210);

    let mut cpu = Cpu::new();
    cpu.pc = 0;
    cpu.regs[8] = BASE;
    cpu.regs[DEST as usize] = INITIAL;

    bus.write32::<BusRead>(0, nop());
    bus.write32::<BusRead>(4, encode_i_type(first.opcode(), DEST, 8, first.offset(false)));
    bus.write32::<BusRead>(8, encode_i_type(second.opcode(), DEST, 8, second.offset(true)));
    bus.write32::<BusRead>(12, encode_special(0x21, OBSERVER, DEST, 0));

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.regs[OBSERVER as usize]
}

#[test]
fn loads_encadeados_mantem_valor_antigo_no_delay_da_segunda_carga() {
    for first in LOADS {
        for second in LOADS {
            assert_eq!(
                run_chain(first, second),
                INITIAL,
                "{first:?} seguido de {second:?}: a instrucao seguinte le o valor antigo"
            );
        }
    }
}
