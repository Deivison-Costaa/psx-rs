use psx_core::bus::{Bus, BusRead};
use psx_core::cpu::Cpu;

mod support;
use support::asm::{bus_with_bios_empty, nop};

const JR_RA: u32 = (31 << 21) | 0x08;
const DISPATCHER: u32 = 0x3C08_0000;
const FMT: u32 = 0x2000;

fn jal(addr: u32) -> u32 {
    (0x03 << 26) | ((addr >> 2) & 0x03FF_FFFF)
}

fn escreve_str(bus: &mut Bus, addr: u32, s: &str) {
    for (i, b) in s.bytes().enumerate() {
        bus.write8::<BusRead>(addr + i as u32, b);
    }
    bus.write8::<BusRead>(addr + s.len() as u32, 0);
}

// `vetor` e o que mora em 0x000000A0: `jr $ra` significa ambiente stubado (sem kernel),
// qualquer outra coisa significa que a rotina real da BIOS vai rodar e emitir os bytes.
fn chama_printf(vetor: u32) -> Vec<u8> {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();

    bus.write32::<BusRead>(0xA0, vetor);
    bus.write32::<BusRead>(0xA4, nop());
    escreve_str(&mut bus, FMT, "ola\n");

    cpu.pc = 0;
    cpu.regs[4] = FMT;
    bus.write32::<BusRead>(0, jal(0xA0));
    bus.write32::<BusRead>(4, 0x3401_003F | (9 << 16));
    for _ in 0..3 {
        cpu.step(&mut bus);
    }
    bus.take_tty()
}

#[test]
fn sem_kernel_o_hook_de_printf_emite_a_string() {
    assert_eq!(
        chama_printf(JR_RA),
        b"ola\n",
        "com A0h stubado com `jr $ra` ninguem mais emite: o hook e a unica fonte"
    );
}

#[test]
fn com_kernel_real_o_hook_de_printf_nao_emite_nada() {
    assert!(
        chama_printf(DISPATCHER).is_empty(),
        "com o dispatcher real em A0h quem emite e a BIOS, byte a byte por putchar; \
         emitir aqui tambem duplica cada linha do TTY"
    );
}

#[test]
fn putchar_emite_sempre_porque_e_a_saida_do_dispositivo() {
    for vetor in [JR_RA, DISPATCHER] {
        let mut bus = bus_with_bios_empty();
        let mut cpu = Cpu::new();
        bus.write32::<BusRead>(0xA0, vetor);
        bus.write32::<BusRead>(0xA4, nop());
        cpu.pc = 0;
        cpu.regs[4] = b'X' as u32;
        bus.write32::<BusRead>(0, jal(0xA0));
        bus.write32::<BusRead>(4, 0x3401_003C | (9 << 16));
        for _ in 0..3 {
            cpu.step(&mut bus);
        }
        assert_eq!(
            bus.take_tty(),
            b"X",
            "putchar (A0h/3Ch) e a saida do dispositivo e vale nos dois ambientes"
        );
    }
}
