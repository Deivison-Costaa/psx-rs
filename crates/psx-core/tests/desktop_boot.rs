use psx_core::bus::{Bios, Bus, BusWrite, Ram};
use psx_core::cpu::Cpu;

#[test]
fn bios_vazia_mostra_display_ligado_padrao_gpu() {
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS vazia");
    let ram = Ram::new();
    let mut bus = Bus::new(ram, bios);
    let mut cpu = Cpu::new();

    assert_eq!(cpu.pc, 0xBFC0_0000, "PC inicial deve ser 0xBFC00000");

    for _ in 0..1_000_000 {
        cpu.step(&mut bus);
    }

    assert_eq!(
        cpu.pc,
        0xBFC0_0000 + 1_000_000u32.wrapping_mul(4),
        "CPU deve avancar PC apos 1M NOPs (BIOS vazia = so NOPs)"
    );

    assert!(
        bus.gpu().framebuffer_for_display().is_none(),
        "sem GP1(03h)=0 o display segue DESLIGADO apos reset (GPUSTAT.23=1=Disabled; \
         polaridade corrigida na iter 0112 — a versao anterior deste teste lia o bit invertido)"
    );

    bus.write32::<BusWrite>(0x1F80_1814, 0x0300_0000);
    let fb = bus
        .gpu()
        .framebuffer_for_display()
        .expect("GP1(03h)=0 liga o display: framebuffer deve existir");
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
