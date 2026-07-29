use psx_core::gpu::Gpu;

fn gp0_e6h(gpu: &mut Gpu, param: u32) {
    gpu.write32(0, (0xE6u32 << 24) | (param & 0x3));
}

fn espera_idle(gpu: &mut Gpu) {
    while gpu.read32(4) & (1 << 26) == 0 {}
}

fn preenche_vram(gpu: &mut Gpu, x: u16, y: u16, w: u16, h: u16, pixel: u16) {
    for row in y..y.wrapping_add(h) {
        for col in x..x.wrapping_add(w) {
            let idx = (row as usize & 0x1FF) * 1024 + (col as usize & 0x3FF);
            gpu.vram_raw_mut()[idx] = pixel;
        }
    }
}

#[rustfmt::skip]
#[test]
fn t1_gp0_e6h_bit0_liga_bit15_do_pixel_escrito() {
    let mut gpu = Gpu::new();
    gp0_e6h(&mut gpu, 1);

    let cor: u32 = 0x00000707;
    let cmd: u32 = (0x20u32 << 24) | (cor & 0x00FF_FFFF);
    gpu.write32(0, cmd);
    gpu.write32(0, 0u32);
    gpu.write32(0, 4u32);
    gpu.write32(0, ((3u32) << 16) | 2u32);
    espera_idle(&mut gpu);

    assert_eq!(
        gpu.vram_pixel(2, 1), 0x8000,
        "T1: GP0(E6h).0=1 -> bit15=1, R=7>>3=0, G=7>>3=0, esperado 0x8000, obtido 0x{:04X}",
        gpu.vram_pixel(2, 1),
    );
}

#[rustfmt::skip]
#[test]
fn t2_gp0_e6h_bit0_zero_bit15_e_zero() {
    let mut gpu = Gpu::new();
    gp0_e6h(&mut gpu, 0);

    let cor: u32 = 0x00000707;
    let cmd: u32 = (0x20u32 << 24) | (cor & 0x00FF_FFFF);
    gpu.write32(0, cmd);
    gpu.write32(0, 0u32);
    gpu.write32(0, 4u32);
    gpu.write32(0, ((3u32) << 16) | 2u32);
    espera_idle(&mut gpu);

    assert_eq!(
        gpu.vram_pixel(2, 1), 0x0000,
        "T2: GP0(E6h).0=0 -> bit15=0, R=7>>3=0, esperado 0x0000, obtido 0x{:04X}",
        gpu.vram_pixel(2, 1),
    );
}

#[rustfmt::skip]
#[test]
fn t3_gp0_e6h_bit1_protege_pixel_com_bit15_1() {
    let mut gpu = Gpu::new();

    preenche_vram(&mut gpu, 2, 1, 1, 1, 0x8001);

    gp0_e6h(&mut gpu, 2);

    let cor: u32 = 0x00F80000;
    let cmd: u32 = (0x20u32 << 24) | (cor & 0x00FF_FFFF);
    gpu.write32(0, cmd);
    gpu.write32(0, 0u32);
    gpu.write32(0, 4u32);
    gpu.write32(0, ((3u32) << 16) | 2u32);
    espera_idle(&mut gpu);

    assert_eq!(
        gpu.vram_pixel(2, 1), 0x8001,
        "T3: pixel(2,1) com bit15=1 nao foi sobrescrito (write-protect), esperado 0x8001, obtido 0x{:04X}",
        gpu.vram_pixel(2, 1),
    );
}

#[rustfmt::skip]
#[test]
fn t4_gp0_e6h_bit1_zero_permite_sobrescrever_pixel_com_bit15_1() {
    let mut gpu = Gpu::new();

    preenche_vram(&mut gpu, 2, 1, 1, 1, 0x8001);

    gp0_e6h(&mut gpu, 0);

    let cor: u32 = 0x00F80000;
    let cmd: u32 = (0x20u32 << 24) | (cor & 0x00FF_FFFF);
    gpu.write32(0, cmd);
    gpu.write32(0, 0u32);
    gpu.write32(0, 4u32);
    gpu.write32(0, ((3u32) << 16) | 2u32);
    espera_idle(&mut gpu);

    assert_eq!(
        gpu.vram_pixel(2, 1), 0x7C00,
        "T4: sem write-protect, pixel(2,1) foi sobrescrito, esperado 0x7C00 (B=31), obtido 0x{:04X}",
        gpu.vram_pixel(2, 1),
    );
}

#[rustfmt::skip]
#[test]
fn t5_mask_bit_aplica_a_linhas() {
    let mut gpu = Gpu::new();

    preenche_vram(&mut gpu, 2, 0, 1, 1, 0x8000);

    gp0_e6h(&mut gpu, 2);

    let cor: u32 = 0x00F80000;
    let cmd: u32 = (0x40u32 << 24) | (cor & 0x00FF_FFFF);
    gpu.write32(0, cmd);
    gpu.write32(0, 0u32);
    gpu.write32(0, 4u32);
    espera_idle(&mut gpu);

    assert_eq!(
        gpu.vram_pixel(2, 0), 0x8000,
        "T5: linha respeita write-protect, pixel(2,0) nao sobrescrito, obtido 0x{:04X}",
        gpu.vram_pixel(2, 0),
    );
}

#[rustfmt::skip]
#[test]
fn t6_mask_bit_aplica_a_retangulos() {
    let mut gpu = Gpu::new();

    preenche_vram(&mut gpu, 0, 0, 1, 1, 0x8000);

    gp0_e6h(&mut gpu, 2);

    let cor: u32 = 0x00F80000;
    let cmd: u32 = (0x60u32 << 24) | (cor & 0x00FF_FFFF);
    gpu.write32(0, cmd);
    gpu.write32(0, 0u32);
    gpu.write32(0, ((1u32) << 16) | 1u32);
    espera_idle(&mut gpu);

    assert_eq!(
        gpu.vram_pixel(0, 0), 0x8000,
        "T6: retangulo respeita write-protect, pixel(0,0) nao sobrescrito, obtido 0x{:04X}",
        gpu.vram_pixel(0, 0),
    );
}

#[rustfmt::skip]
#[test]
fn t7_fill_nao_respeita_mask_bit() {
    let mut gpu = Gpu::new();

    preenche_vram(&mut gpu, 0, 0, 1, 1, 0x8000);

    gp0_e6h(&mut gpu, 3);

    gpu.write32(0, (0x02u32 << 24) | 0x00F81820);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0001_0010);

    assert_eq!(
        gpu.vram_pixel(0, 0), 0x7C64,
        "T7: fill ignora mask bit, pixel(0,0) foi sobrescrito, obtido 0x{:04X}",
        gpu.vram_pixel(0, 0),
    );
}

#[rustfmt::skip]
#[test]
fn t8_cpu_para_vram_nao_respeita_mask_bit() {
    let mut gpu = Gpu::new();

    preenche_vram(&mut gpu, 0, 0, 1, 1, 0x8000);

    gp0_e6h(&mut gpu, 3);

    let cmd_top3_5: u32 = (0xA0u32) << 24;
    gpu.write32(0, cmd_top3_5);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0001_0001);
    gpu.write32(0, 0x0000_BEEF);

    assert_eq!(
        gpu.vram_pixel(0, 0), 0xBEEF,
        "T8: CPU->VRAM ignora mask bit, pixel(0,0) foi sobrescrito, obtido 0x{:04X}",
        gpu.vram_pixel(0, 0),
    );
}
