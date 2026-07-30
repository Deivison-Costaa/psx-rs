use psx_core::bus::{Bios, Bus, Ram};
use psx_core::cpu::Cpu;
use psx_core::irq::Irq;

fn bios_path() -> &'static str {
    if std::path::Path::new("bios/SCPH1001.BIN").exists() {
        return "bios/SCPH1001.BIN";
    }
    if std::path::Path::new("../../bios/SCPH1001.BIN").exists() {
        return "../../bios/SCPH1001.BIN";
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
fn bios_escreve_i_mask_durante_boot() {
    if !bios_existe() {
        eprintln!("BIOS nao encontrada — teste ignorado");
        return;
    }

    let bios = carregar_bios().expect("BIOS invalida");
    let ram = Ram::new();
    let mut bus = Bus::new(ram, bios);
    let mut cpu = Cpu::new();

    let max_steps = 30_000_000usize;
    let mut mask_nao_zero = false;
    for i in 0..max_steps {
        cpu.step(&mut bus);
        if bus.irq().read_mask() != 0 {
            mask_nao_zero = true;
            eprintln!(
                "I_MASK=0x{:04X} no passo {} (PC=0x{:08X}, writes={})",
                bus.irq().read_mask(),
                i + 1,
                cpu.pc,
                bus.irq().mask_write_count,
            );
            break;
        }
    }

    assert!(
        mask_nao_zero,
        "I_MASK deve deixar de ser 0x0000 durante o boot da BIOS. \
         Após {} passos: mask_write_count={}, I_MASK=0x{:04X}",
        max_steps,
        bus.irq().mask_write_count,
        bus.irq().read_mask(),
    );
}

#[test]
fn write_mask_incrementa_contador() {
    let mut irq = Irq::new();
    assert_eq!(irq.mask_write_count, 0);

    irq.write_mask(0x0001);
    assert_eq!(irq.mask_write_count, 1);
    assert_eq!(irq.read_mask(), 1);

    irq.write_mask_byte(0, 0x02);
    assert_eq!(irq.mask_write_count, 2);

    irq.write_mask_half(0, 0x0004);
    assert_eq!(irq.mask_write_count, 3);
}
