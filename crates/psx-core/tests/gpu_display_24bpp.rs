use psx_core::gpu::Gpu;

fn write_gp1(gpu: &mut Gpu, cmd: u8, param: u32) {
    gpu.write32(4, ((cmd as u32) << 24) | (param & 0x00FF_FFFF));
}

fn write_halfword(gpu: &mut Gpu, x: u16, y: u16, valor: u16) {
    let idx = (y as usize & 0x1FF) * 1024 + (x as usize & 0x3FF);
    gpu.vram_raw_mut()[idx] = valor;
}

fn px(fb: &psx_core::gpu::Framebuffer, x: usize, y: usize) -> (u8, u8, u8, u8) {
    let o = (y * fb.width as usize + x) * 4;
    (fb.data[o], fb.data[o + 1], fb.data[o + 2], fb.data[o + 3])
}

// GP1(08h) bit4 liga a profundidade de 24 bits da area de display (GPUSTAT.21).
fn liga_24bpp(gpu: &mut Gpu) {
    write_gp1(gpu, 0x08, 0x10);
}

#[test]
fn t1_gpustat21_marca_a_area_de_display_em_24_bits() {
    let mut gpu = Gpu::new();
    liga_24bpp(&mut gpu);
    assert_eq!(
        gpu.stat() & (1 << 21),
        1 << 21,
        "T1: GP1(08h) com bit4=1 deve setar GPUSTAT.21 (Display Area Color Depth, \
         docs/reference/03-gpu.md L1019); obtido GPUSTAT={:08X}",
        gpu.stat()
    );
}

#[test]
fn t2_pixels_de_24_bits_ocupam_tres_bytes_cada() {
    let mut gpu = Gpu::new();
    liga_24bpp(&mut gpu);

    // "The 24bit pixels occupy 3 bytes (not 4 bytes with unused MSBs), so each 6 bytes
    // contain two 24bit pixels" (docs/reference/03-gpu.md L1284-1285): a sequencia de bytes
    // 12 34 56 78 9A BC e o pixel0=(12,34,56) e o pixel1=(78,9A,BC).
    write_halfword(&mut gpu, 0, 0, 0x3412);
    write_halfword(&mut gpu, 1, 0, 0x7856);
    write_halfword(&mut gpu, 2, 0, 0xBC9A);
    gpu.enter_vblank();
    let fb = gpu.framebuffer();

    assert_eq!(
        px(&fb, 0, 0),
        (0x12, 0x34, 0x56, 255),
        "T2: em 24bpp o pixel 0 le os bytes 0..2 da linha (R=12,G=34,B=56); obtido {:?}. \
         Sem isso a FMV (unico uso real do modo 24bpp) sai como ruido.",
        px(&fb, 0, 0)
    );
    assert_eq!(
        px(&fb, 1, 0),
        (0x78, 0x9A, 0xBC, 255),
        "T2: o pixel 1 comeca no byte 3, ou seja no byte alto do halfword 1 — nao no \
         halfword 1 inteiro; obtido {:?}",
        px(&fb, 1, 0)
    );
}

#[test]
fn t3_24bpp_usa_os_oito_bits_de_cada_canal() {
    let mut gpu = Gpu::new();
    liga_24bpp(&mut gpu);

    // Vermelho puro 0xFF,0x00,0x00 seguido de verde puro 0x00,0xFF,0x00.
    write_halfword(&mut gpu, 0, 0, 0x00FF);
    write_halfword(&mut gpu, 1, 0, 0x0000);
    write_halfword(&mut gpu, 2, 0, 0x00FF);
    gpu.enter_vblank();
    let fb = gpu.framebuffer();

    assert_eq!(
        px(&fb, 0, 0),
        (0xFF, 0x00, 0x00, 255),
        "T3: 24bpp tem 8 bits por canal (0..255), nao 5 (docs/reference/03-gpu.md \
         L1280-1282); obtido {:?}",
        px(&fb, 0, 0)
    );
    assert_eq!(
        px(&fb, 1, 0),
        (0x00, 0xFF, 0x00, 255),
        "T3: pixel 1 deveria ser verde puro; obtido {:?}",
        px(&fb, 1, 0)
    );
}

#[test]
fn t4_display_start_x_continua_em_halfwords_no_modo_24bpp() {
    let mut gpu = Gpu::new();
    liga_24bpp(&mut gpu);

    write_halfword(&mut gpu, 10, 5, 0x3412);
    write_halfword(&mut gpu, 11, 5, 0x7856);
    write_gp1(&mut gpu, 0x05, 10 | (5 << 10));
    gpu.enter_vblank();
    let fb = gpu.framebuffer();

    assert_eq!(
        px(&fb, 0, 0),
        (0x12, 0x34, 0x56, 255),
        "T4: GP1(05h) da o inicio da area de display em halfwords da VRAM tambem em \
         24bpp; obtido {:?}",
        px(&fb, 0, 0)
    );
}

#[test]
fn t5_quinze_bits_nao_regride() {
    let mut gpu = Gpu::new();
    write_halfword(&mut gpu, 0, 0, 0x001F);
    gpu.enter_vblank();
    let fb = gpu.framebuffer();

    assert_eq!(
        px(&fb, 0, 0),
        (0xF8, 0x00, 0x00, 255),
        "T5: com GPUSTAT.21=0 o display continua 15bpp; obtido {:?}",
        px(&fb, 0, 0)
    );
}
