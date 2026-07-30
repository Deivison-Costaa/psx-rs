use psx_core::bus::{Bios, Bus, Ram};
use psx_core::cpu::Cpu;

fn bios_path() -> &'static str {
    for path in &["bios/SCPH1001.BIN", "../../bios/SCPH1001.BIN"] {
        if std::path::Path::new(path).exists() {
            return path;
        }
    }
    "bios/SCPH1001.BIN"
}

fn bios_existe() -> bool {
    std::path::Path::new(bios_path()).exists()
}

fn carregar_bios() -> Option<Bios> {
    let data = std::fs::read(bios_path()).ok()?;
    Bios::from_bytes(data).ok()
}

#[test]
fn boot_da_bios_nao_imprime_vsync_timeout() {
    if !bios_existe() {
        eprintln!("BIOS nao encontrada — teste ignorado");
        return;
    }

    let bios = carregar_bios().expect("BIOS invalida");
    let ram = Ram::new();
    let mut bus = Bus::new(ram, bios);
    let mut cpu = Cpu::new();

    let max_steps = 50_000_000usize;
    for _ in 0..max_steps {
        cpu.step(&mut bus);
    }

    let tty = bus.take_tty();
    let tty_str = String::from_utf8_lossy(&tty);

    assert!(
        !tty_str.contains("VSync: timeout"),
        "BIOS nao deve imprimir 'VSync: timeout' apos {} passos.\n\
         TTY ({:.100}...):\n{}",
        max_steps,
        tty_str,
        tty_str
    );
}
