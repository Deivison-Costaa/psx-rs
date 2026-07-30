use psx_core::bus::{Bios, Bus, Ram};
use psx_core::cpu::Cpu;

fn bios_path() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../bios/SCPH1001.BIN")
}

fn boot_bios() -> (Cpu, Bus) {
    let bios_data = std::fs::read(bios_path()).expect("BIOS nao encontrada");
    let bios = Bios::from_bytes(bios_data).expect("BIOS invalida");
    let ram = Ram::new();
    let bus = Bus::new(ram, bios);
    let cpu = Cpu::new();
    (cpu, bus)
}

#[test]
fn psx_desktop_com_bios_mostra_display_ligado() {
    let (mut cpu, mut bus) = boot_bios();
    let max_steps = 5_000_000;

    for _ in 0..max_steps {
        cpu.step(&mut bus);
    }

    let fb = bus
        .gpu()
        .framebuffer_for_display()
        .expect("Display deve estar ligado — janela nao mostra 'Display desligado'");

    assert!(
        fb.width > 0,
        "Framebuffer width deve ser > 0, obtido {}",
        fb.width
    );
    assert!(
        fb.height > 0,
        "Framebuffer height deve ser > 0, obtido {}",
        fb.height
    );
    assert_eq!(
        fb.data.len(),
        fb.width as usize * fb.height as usize * 4,
        "Framebuffer data deve ter width*height*4 bytes"
    );
}

#[test]
fn bios_vazia_mostra_display_ligado_padrao_gpu() {
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS vazia");
    let ram = Ram::new();
    let mut bus = Bus::new(ram, bios);
    let mut cpu = Cpu::new();

    for _ in 0..1_000_000 {
        cpu.step(&mut bus);
    }

    let fb = bus
        .gpu()
        .framebuffer_for_display()
        .expect("GPU padrao tem display ligado (bit 23 set)");

    assert!(
        fb.width > 0,
        "Framebuffer width deve ser > 0, obtido {}",
        fb.width
    );
    assert!(
        fb.height > 0,
        "Framebuffer height deve ser > 0, obtido {}",
        fb.height
    );
    assert_eq!(
        fb.data.len(),
        fb.width as usize * fb.height as usize * 4,
        "Framebuffer data deve ter width*height*4 bytes"
    );
}
