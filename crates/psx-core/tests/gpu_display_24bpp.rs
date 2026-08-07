use psx_core::gpu::Gpu;

fn write_gp1(gpu: &mut Gpu, cmd: u8, param: u32) {
    gpu.write32(4, ((cmd as u32) << 24) | (param & 0x00FF_FFFF));
}

fn gpu_4x2(modo_gp1_08: u32) -> Gpu {
    let mut gpu = Gpu::new();
    write_gp1(&mut gpu, 0x03, 0);
    write_gp1(&mut gpu, 0x05, 0);
    write_gp1(&mut gpu, 0x06, 20 << 12);
    write_gp1(&mut gpu, 0x07, 2 << 10);
    write_gp1(&mut gpu, 0x08, modo_gp1_08);
    gpu
}

fn escreve_linha(gpu: &mut Gpu, y: usize, halfwords: &[u16]) {
    let vram = gpu.vram_raw_mut();
    for (i, hw) in halfwords.iter().enumerate() {
        vram[y * 1024 + i] = *hw;
    }
}

fn rgb(fb: &psx_core::gpu::Framebuffer, i: usize) -> [u8; 3] {
    [fb.data[i * 4], fb.data[i * 4 + 1], fb.data[i * 4 + 2]]
}

#[test]
fn modo_24bpp_le_tres_bytes_por_pixel() {
    let mut gpu = gpu_4x2(0x10);
    escreve_linha(
        &mut gpu,
        0,
        &[0x2211, 0x4433, 0x6655, 0x8877, 0xAA99, 0xCCBB],
    );
    gpu.enter_vblank();

    let fb = gpu.framebuffer_for_display().expect("display ligado");
    assert_eq!(fb.width, 4, "largura nao muda com a profundidade de cor");
    assert_eq!(
        rgb(&fb, 0),
        [0x11, 0x22, 0x33],
        "03-gpu.md L1279-1282: em 24bpp os bits 0-7 sao R, 8-15 G, 16-23 B; \
         o pixel 0 ocupa os bytes 0,1,2 da linha (halfwords 2211h,4433h)"
    );
    assert_eq!(
        rgb(&fb, 1),
        [0x44, 0x55, 0x66],
        "03-gpu.md L1284-1285: cada 6 bytes contem DOIS pixels de 24 bits, \
         entao o pixel 1 comeca no byte 3 e nao no halfword 1"
    );
    assert_eq!(rgb(&fb, 2), [0x77, 0x88, 0x99], "pixel 2 nos bytes 6,7,8");
    assert_eq!(rgb(&fb, 3), [0xAA, 0xBB, 0xCC], "pixel 3 nos bytes 9,10,11");
}

#[test]
fn modo_24bpp_avanca_uma_linha_de_vram_por_scanline() {
    let mut gpu = gpu_4x2(0x10);
    escreve_linha(&mut gpu, 1, &[0x0201, 0x0403, 0x0605]);
    gpu.enter_vblank();

    let fb = gpu.framebuffer_for_display().expect("display ligado");
    assert_eq!(fb.height, 2, "altura e Y2-Y1 = 2");
    assert_eq!(
        rgb(&fb, 4),
        [0x01, 0x02, 0x03],
        "a segunda scanline comeca no inicio da proxima linha de VRAM, \
         nao 3/2 halfwords adiante"
    );
}

#[test]
fn modo_15bpp_continua_expandindo_5_bits_por_componente() {
    let mut gpu = gpu_4x2(0x00);
    escreve_linha(&mut gpu, 0, &[0x7C1F]);
    gpu.enter_vblank();

    let fb = gpu.framebuffer_for_display().expect("display ligado");
    assert_eq!(
        rgb(&fb, 0),
        [0xF8, 0x00, 0xF8],
        "03-gpu.md L1275-1277: com GPUSTAT.21=0 o pixel continua sendo 5:5:5"
    );
}
