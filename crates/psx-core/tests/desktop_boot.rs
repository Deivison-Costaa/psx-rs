use psx_core::bus::{Bios, Bus, Ram};
use psx_core::cpu::Cpu;

// A BIOS real e gitignored e nunca entra no repositorio, entao teste que a le nao pode rodar na
// CI. Tornar o teste condicional ao arquivo tambem esta fora: `ci_workflow.rs` reprova condicional
// no job `check` porque "um passo pulado nao mede nada". O criterio com a BIOS real e medido pelo
// orquestrador e registrado no doc da iteracao.

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
