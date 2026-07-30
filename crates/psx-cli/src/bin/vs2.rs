use psx_core::bus::{Bios, Bus, BusRead, Ram};
use psx_core::cpu::Cpu;

const CABECA: u32 = 0x8005_9DC8;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let bios_path = args.get(1).expect("uso: vs2 <BIOS>");
    let data = std::fs::read(bios_path).expect("lendo BIOS");
    let bios = Bios::from_bytes(data).expect("BIOS valida");
    let ram = Ram::new();
    let mut bus = Bus::new(ram, bios);
    let mut cpu = Cpu::new();

    let mut esperas: Vec<(usize, usize, u32, u32)> = Vec::new();
    let mut iteracoes = 0usize;
    let mut inicio = 0usize;
    let mut orcamento_inicial = 0u32;
    let mut ultimo_passo_no_laco = 0usize;

    for passo in 0..30_000_000usize {
        let pc = cpu.pc;
        if pc == CABECA {
            if passo > ultimo_passo_no_laco + 100 {
                if iteracoes > 0 && esperas.len() < 8 {
                    let restante = bus.read32::<BusRead>(cpu.regs[29].wrapping_add(0x1C));
                    esperas.push((inicio, iteracoes, orcamento_inicial, restante));
                }
                inicio = passo;
                iteracoes = 0;
                orcamento_inicial = bus.read32::<BusRead>(cpu.regs[29].wrapping_add(0x1C));
            }
            iteracoes += 1;
            ultimo_passo_no_laco = passo;
        }
        cpu.step(&mut bus);
    }

    println!("periodo de vblank medido: 566188 passos");
    println!();
    println!("{:>12} {:>10} {:>12} {:>10}", "inicio", "iteracoes", "orcamento", "restante");
    for (inicio, it, orc, rest) in &esperas {
        println!("{inicio:>12} {it:>10} {orc:>12} {rest:>10}");
    }
}
