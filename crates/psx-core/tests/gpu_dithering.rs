use psx_core::gpu::Gpu;

fn gp0_e1h(gpu: &mut Gpu, param: u32) {
    gpu.write32(0, (0xE1u32 << 24) | (param & 0x3FFF));
}

fn espera_idle(gpu: &mut Gpu) {
    while gpu.read32(4) & (1 << 26) == 0 {}
}

fn desenha_gouraud_triangulo(
    gpu: &mut Gpu,
    v0: (i16, i16),
    v1: (i16, i16),
    v2: (i16, i16),
    c0: u32,
    c1: u32,
    c2: u32,
) {
    let cmd: u32 = (0x30u32 << 24) | (c0 & 0x00FF_FFFF);
    gpu.write32(0, cmd);
    gpu.write32(0, ((v0.1 as u16 as u32) << 16) | (v0.0 as u16 as u32));
    gpu.write32(0, c1 & 0x00FF_FFFF);
    gpu.write32(0, ((v1.1 as u16 as u32) << 16) | (v1.0 as u16 as u32));
    gpu.write32(0, c2 & 0x00FF_FFFF);
    gpu.write32(0, ((v2.1 as u16 as u32) << 16) | (v2.0 as u16 as u32));
}

#[rustfmt::skip]
#[test]
fn t1_gouraud_triangulo_com_dither_matriz_4x4() {
    let mut gpu = Gpu::new();
    gp0_e1h(&mut gpu, 1 << 9);

    let cor: u32 = 0x00000707;
    desenha_gouraud_triangulo(
        &mut gpu,
        (0, 0), (4, 0), (2, 3),
        cor, cor, cor,
    );
    espera_idle(&mut gpu);

    assert_eq!(
        gpu.vram_pixel(3, 0), 0x0021,
        "T1(3,0): offset +1, R=7+1=8>>3=1, G=7+1=8>>3=1, esperado 0x0021",
    );
    assert_eq!(
        gpu.vram_pixel(0, 0), 0x0000,
        "T1(0,0): offset -4, R=7-4=3>>3=0, G=7-4=3>>3=0, esperado 0x0000",
    );
    assert_eq!(
        gpu.vram_pixel(1, 0), 0x0000,
        "T1(1,0): offset 0, R=7>>3=0, G=7>>3=0, esperado 0x0000",
    );
    assert_eq!(
        gpu.vram_pixel(0, 1), 0x0021,
        "T1(0,1): offset +2, R=7+2=9>>3=1, G=7+2=9>>3=1, esperado 0x0021",
    );
    assert_eq!(
        gpu.vram_pixel(1, 1), 0x0000,
        "T1(1,1): offset -2, R=7-2=5>>3=0, G=7-2=5>>3=0, esperado 0x0000",
    );
    assert_eq!(
        gpu.vram_pixel(2, 1), 0x0021,
        "T1(2,1): offset +3, R=7+3=10>>3=1, G=7+3=10>>3=1, esperado 0x0021",
    );
    assert_eq!(
        gpu.vram_pixel(1, 2), 0x0021,
        "T1(1,2): offset +1, R=7+1=8>>3=1, G=7+1=8>>3=1, esperado 0x0021",
    );
}

#[rustfmt::skip]
#[test]
fn t2_gouraud_triangulo_sem_dither() {
    let mut gpu = Gpu::new();

    let cor: u32 = 0x00000707;
    desenha_gouraud_triangulo(
        &mut gpu,
        (0, 0), (4, 0), (2, 3),
        cor, cor, cor,
    );
    espera_idle(&mut gpu);

    assert_eq!(
        gpu.vram_pixel(3, 0), 0x0000,
        "T2(3,0): sem dither, R=7>>3=0, esperado 0x0000",
    );
    assert_eq!(
        gpu.vram_pixel(0, 1), 0x0000,
        "T2(0,1): sem dither, R=7>>3=0, G=7>>3=0, esperado 0x0000",
    );
}

#[rustfmt::skip]
#[test]
fn t3_poligono_flat_nao_aplica_dither() {
    let mut gpu = Gpu::new();
    gp0_e1h(&mut gpu, 1 << 9);

    let cor: u32 = 0x00000707;
    let cmd: u32 = (0x20u32 << 24) | (cor & 0x00FF_FFFF);
    gpu.write32(0, cmd);
    gpu.write32(0, ((0u32) << 16) | 0u32);
    gpu.write32(0, ((0u32) << 16) | 4u32);
    gpu.write32(0, ((3u32) << 16) | 2u32);
    espera_idle(&mut gpu);

    let pixel = gpu.vram_pixel(3, 0);
    assert_eq!(
        pixel, 0x0000,
        "T3: poligono flat nao aplica dither, R=7>>3=0, obtido 0x{:04X}",
        pixel,
    );
}

#[rustfmt::skip]
#[test]
fn t4_linha_aplica_dither() {
    let mut gpu = Gpu::new();
    gp0_e1h(&mut gpu, 1 << 9);

    let cor: u32 = 0x00000707;
    let cmd: u32 = (0x40u32 << 24) | (cor & 0x00FF_FFFF);
    gpu.write32(0, cmd);
    gpu.write32(0, ((0u32) << 16) | 0u32);
    gpu.write32(0, ((0u32) << 16) | 4u32);
    espera_idle(&mut gpu);

    assert_eq!(
        gpu.vram_pixel(0, 0), 0x0000,
        "T4(0,0): dither offset -4, R=7-4=3>>3=0, esperado 0x0000",
    );
    assert_eq!(
        gpu.vram_pixel(3, 0), 0x0021,
        "T4(3,0): dither offset +1, R=7+1=8>>3=1, G=7+1=8>>3=1, esperado 0x0021",
    );
}

#[rustfmt::skip]
#[test]
fn t5_retangulo_nao_aplica_dither() {
    let mut gpu = Gpu::new();
    gp0_e1h(&mut gpu, 1 << 9);

    let cor: u32 = 0x00000707;
    let cmd: u32 = (0x60u32 << 24) | (cor & 0x00FF_FFFF);
    gpu.write32(0, cmd);
    gpu.write32(0, ((0u32) << 16) | 3u32);
    gpu.write32(0, ((2u32) << 16) | 1u32);
    espera_idle(&mut gpu);

    let pixel = gpu.vram_pixel(3, 0);
    assert_eq!(
        pixel, 0x0000,
        "T5: retangulo nao aplica dither, R=7>>3=0, obtido 0x{:04X}",
        pixel,
    );
}

#[rustfmt::skip]
#[test]
fn t6_saturacao_dither_nao_ultrapassa_255() {
    let mut gpu = Gpu::new();
    gp0_e1h(&mut gpu, 1 << 9);

    let cor: u32 = 0x00FCFCFC;
    let cmd: u32 = (0x30u32 << 24) | (cor & 0x00FF_FFFF);
    gpu.write32(0, cmd);
    gpu.write32(0, ((0u32) << 16) | 0u32);
    gpu.write32(0, cor & 0x00FF_FFFF);
    gpu.write32(0, ((0u32) << 16) | 4u32);
    gpu.write32(0, cor & 0x00FF_FFFF);
    gpu.write32(0, ((3u32) << 16) | 2u32);
    espera_idle(&mut gpu);

    assert_eq!(
        gpu.vram_pixel(1, 1), 0x7FFF,
        "T6: offset -2, R=252-2=250>>3=31, G=31, B=31, esperado 0x7FFF (31,31,31)",
    );
}

#[rustfmt::skip]
#[test]
fn t7_saturacao_dither_nao_fica_negativa() {
    let mut gpu = Gpu::new();
    gp0_e1h(&mut gpu, 1 << 9);

    let cor: u32 = 0x00000101;
    let cmd: u32 = (0x30u32 << 24) | (cor & 0x00FF_FFFF);
    gpu.write32(0, cmd);
    gpu.write32(0, ((0u32) << 16) | 0u32);
    gpu.write32(0, cor & 0x00FF_FFFF);
    gpu.write32(0, ((0u32) << 16) | 4u32);
    gpu.write32(0, cor & 0x00FF_FFFF);
    gpu.write32(0, ((3u32) << 16) | 2u32);
    espera_idle(&mut gpu);

    assert_eq!(
        gpu.vram_pixel(0, 0), 0x0000,
        "T7: offset -4, R=1-4=-3 clamp 0>>3=0, G=1-4=-3 clamp 0>>3=0, esperado 0x0000",
    );
}
