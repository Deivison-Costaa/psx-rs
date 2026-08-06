use psx_core::gpu::Gpu;

fn write_gp1(gpu: &mut Gpu, cmd: u8, param: u32) {
    gpu.write32(4, ((cmd as u32) << 24) | (param & 0x00FF_FFFF));
}

fn write_pixel(gpu: &mut Gpu, x: u16, y: u16, color: u16) {
    let idx = (y as usize & 0x1FF) * 1024 + (x as usize & 0x3FF);
    gpu.vram_raw_mut()[idx] = color;
}

#[rustfmt::skip]
#[test]
fn t1_framebuffer_dimensoes_padrao() {
    let gpu = Gpu::new();
    let fb = gpu.framebuffer();

    assert_eq!(
        fb.width, 256,
        "T1: largura padrao deve ser 256 (HR1=0), obtido {}",
        fb.width,
    );
    assert_eq!(
        fb.height, 240,
        "T1: altura padrao deve ser 240, obtido {}",
        fb.height,
    );
    assert_eq!(
        fb.data.len(), (256 * 240 * 4) as usize,
        "T1: framebuffer deve ter {} bytes, obtido {}",
        256 * 240 * 4, fb.data.len(),
    );
}

#[rustfmt::skip]
#[test]
fn t2_framebuffer_muda_dimensoes_com_display_range_e_modo() {
    let mut gpu = Gpu::new();

    write_gp1(&mut gpu, 0x08, 0x01);
    write_gp1(&mut gpu, 0x06, 0x200 | (0xE00 << 12));
    write_gp1(&mut gpu, 0x07, 0x10 | (0xF0 << 10));

    let fb = gpu.framebuffer();

    let expected_width = ((0xE00u16 - 0x200u16) / 8 + 2) & !3;
    let expected_height = 0xF0u16 - 0x10u16;

    assert_eq!(
        fb.width, expected_width,
        "T2: largura deve ser {}, obtido {}",
        expected_width, fb.width,
    );
    assert_eq!(
        fb.height, expected_height,
        "T2: altura deve ser {}, obtido {}",
        expected_height, fb.height,
    );
}

#[rustfmt::skip]
#[test]
fn t3_framebuffer_converte_15bpp_para_rgba8() {
    let mut gpu = Gpu::new();

    write_pixel(&mut gpu, 0, 0, 0x7FFF);
    write_pixel(&mut gpu, 1, 0, 0x001F);
    write_pixel(&mut gpu, 2, 0, 0x03E0);
    write_pixel(&mut gpu, 3, 0, 0x7C00);
    write_pixel(&mut gpu, 4, 0, 0x0000);

    gpu.enter_vblank();
    let fb = gpu.framebuffer();

    let px = |index: usize| {
        let offset = index * 4;
        (fb.data[offset], fb.data[offset + 1], fb.data[offset + 2], fb.data[offset + 3])
    };

    assert_eq!(
        px(0),
        (0xF8, 0xF8, 0xF8, 255),
        "T3: pixel 0 (0x7FFF = branco maximo) deve ser (0xF8, 0xF8, 0xF8, 255), obtido {:?}",
        px(0),
    );

    assert_eq!(
        px(1),
        (0xF8, 0x00, 0x00, 255),
        "T3: pixel 1 (0x001F = vermelho puro) deve ser (0xF8, 0x00, 0x00, 255), obtido {:?}",
        px(1),
    );

    assert_eq!(
        px(2),
        (0x00, 0xF8, 0x00, 255),
        "T3: pixel 2 (0x03E0 = verde puro) deve ser (0x00, 0xF8, 0x00, 255), obtido {:?}",
        px(2),
    );

    assert_eq!(
        px(3),
        (0x00, 0x00, 0xF8, 255),
        "T3: pixel 3 (0x7C00 = azul puro) deve ser (0x00, 0x00, 0xF8, 255), obtido {:?}",
        px(3),
    );

    let preto = px(4);
    assert!(
        preto.0 == 0 && preto.1 == 0 && preto.2 == 0,
        "T3: pixel 4 (0x0000 = preto) deve ser (0, 0, 0, 255), obtido {:?}",
        preto,
    );
}

#[rustfmt::skip]
#[test]
fn t4_framebuffer_respeita_display_start_offset() {
    let mut gpu = Gpu::new();

    write_pixel(&mut gpu, 10, 5, 0x001F);

    write_gp1(&mut gpu, 0x05, 10 | (5 << 10));

    gpu.enter_vblank();
    let fb = gpu.framebuffer();

    let offset = 0;
    let px = (fb.data[offset], fb.data[offset + 1], fb.data[offset + 2], fb.data[offset + 3]);

    assert_eq!(
        px,
        (0xF8, 0x00, 0x00, 255),
        "T4: pixel (0,0) do framebuffer deve ler VRAM (display_start_x, display_start_y), obtido {:?}",
        px,
    );
}

#[rustfmt::skip]
#[test]
fn t5_framebuffer_faz_wrap_de_coordenadas_da_vram() {
    let mut gpu = Gpu::new();

    write_pixel(&mut gpu, 0x3FF, 0x1FF, 0x001F);

    write_gp1(&mut gpu, 0x05, (0x3FF) | (0x1FF << 10));

    gpu.enter_vblank();
    let fb = gpu.framebuffer();

    let offset = 0;
    let px = (fb.data[offset], fb.data[offset + 1], fb.data[offset + 2], fb.data[offset + 3]);

    assert_eq!(
        px,
        (0xF8, 0x00, 0x00, 255),
        "T5: pixel no canto maximo da VRAM deve ser lido corretamente, obtido {:?}",
        px,
    );
}

#[rustfmt::skip]
#[test]
fn t6_framebuffer_le_multiplas_linhas_da_vram() {
    let mut gpu = Gpu::new();

    write_pixel(&mut gpu, 100, 10, 0x001F);
    write_pixel(&mut gpu, 101, 10, 0x03E0);
    write_pixel(&mut gpu, 100, 11, 0x7C00);
    write_pixel(&mut gpu, 101, 11, 0x7FFF);

    write_gp1(&mut gpu, 0x05, 100 | (10 << 10));

    gpu.enter_vblank();
    let fb = gpu.framebuffer();

    let px = |x: usize, y: usize| {
        let offset = (y * fb.width as usize + x) * 4;
        (
            fb.data[offset],
            fb.data[offset + 1],
            fb.data[offset + 2],
            fb.data[offset + 3],
        )
    };

    assert_eq!(
        px(0, 0),
        (0xF8, 0x00, 0x00, 255),
        "T6: pixel (0, 0) deve ser vermelho, obtido {:?}",
        px(0, 0),
    );
    assert_eq!(
        px(1, 0),
        (0x00, 0xF8, 0x00, 255),
        "T6: pixel (1, 0) deve ser verde, obtido {:?}",
        px(1, 0),
    );
    assert_eq!(
        px(0, 1),
        (0x00, 0x00, 0xF8, 255),
        "T6: pixel (0, 1) deve ser azul, obtido {:?}",
        px(0, 1),
    );
    assert_eq!(
        px(1, 1),
        (0xF8, 0xF8, 0xF8, 255),
        "T6: pixel (1, 1) deve ser branco, obtido {:?}",
        px(1, 1),
    );
}
