use psx_core::gpu::Gpu;

fn write_gp1(gpu: &mut Gpu, cmd: u8, param: u32) {
    gpu.write32(4, ((cmd as u32) << 24) | (param & 0x00FF_FFFF));
}

fn gpu_ligado(mode: u32, x1: u32, x2: u32, y1: u32, y2: u32) -> Gpu {
    let mut gpu = Gpu::new();
    write_gp1(&mut gpu, 0x03, 0);
    write_gp1(&mut gpu, 0x05, 0);
    write_gp1(&mut gpu, 0x08, mode);
    write_gp1(&mut gpu, 0x06, x1 | (x2 << 12));
    write_gp1(&mut gpu, 0x07, y1 | (y2 << 10));
    gpu
}

fn pixel(fb: &psx_core::gpu::Framebuffer, x: usize, y: usize) -> (u8, u8, u8) {
    let o = (y * fb.width as usize + x) * 4;
    (fb.data[o], fb.data[o + 1], fb.data[o + 2])
}

#[test]
fn ntsc_240_linhas_mostra_so_as_224_visiveis() {
    let gpu = gpu_ligado(0x01, 0x260, 0xC60, 0x10, 0x100);
    let fb = gpu.framebuffer_for_display().expect("display ligado");
    assert_eq!(
        (fb.width, fb.height),
        (320, 224),
        "NTSC: so as linhas 24..248 (88h +/- 112) aparecem na TV"
    );
    let raw = gpu.framebuffer();
    assert_eq!(
        raw.height, 240,
        "framebuffer() continua com a faixa Y2-Y1 inteira"
    );
}

#[test]
fn recorte_vertical_pula_as_linhas_de_overscan_do_topo() {
    let mut gpu = gpu_ligado(0x01, 0x260, 0xC60, 0x10, 0x100);
    {
        let vram = gpu.vram_raw_mut();
        vram[7 * 1024] = 0x001F;
        vram[8 * 1024] = 0x03E0;
        vram[231 * 1024] = 0x7C00;
    }
    gpu.enter_vblank();
    let fb = gpu.framebuffer_for_display().expect("display ligado");
    assert_eq!(
        pixel(&fb, 0, 0),
        (0, 0xF8, 0),
        "Y1=16 e a janela comeca em 24: a linha 0 exibida e a linha 8 da VRAM"
    );
    assert_eq!(
        pixel(&fb, 0, 223),
        (0, 0, 0xF8),
        "a ultima linha exibida e a linha 231 da VRAM"
    );
}

#[test]
fn bios_480i_fica_com_448_linhas_e_pula_16_da_vram() {
    let mut gpu = gpu_ligado(0x27, 0x260, 0xC60, 0x10, 0xFF);
    {
        let vram = gpu.vram_raw_mut();
        vram[16 * 1024] = 0x03E0;
        vram[15 * 1024] = 0x001F;
    }
    gpu.enter_vblank();
    let fb = gpu.framebuffer_for_display().expect("display ligado");
    assert_eq!((fb.width, fb.height), (640, 448), "BIOS: 640x448 visivel");
    assert_eq!(
        pixel(&fb, 0, 0),
        (0, 0xF8, 0),
        "480i: 8 linhas de overscan viram 16 linhas de VRAM"
    );
}

#[test]
fn largura_e_a_faixa_horizontal_dividida_pelo_dotclock() {
    let casos = [
        (
            0x41,
            0x260,
            0xC60,
            365,
            "368 px com 2560 ciclos: 2560/7 = 365",
        ),
        (
            0x03,
            592,
            3152,
            640,
            "640 px deslocado para a esquerda: sem corte",
        ),
        (0x02, 0x260, 0xC60, 512, "512 px: 2560/5"),
        (0x00, 512, 3072, 256, "256 px com X1=200h: sem corte"),
        (0x41, 588, 3052, 352, "368 px com faixa curta: 2464/7 = 352"),
    ];
    for (mode, x1, x2, w, msg) in casos {
        let gpu = gpu_ligado(mode, x1, x2, 0x10, 0x100);
        let fb = gpu.framebuffer_for_display().expect("display ligado");
        assert_eq!(fb.width, w, "{msg}");
    }
}

#[test]
fn faixa_inteira_fora_da_janela_vertical_da_imagem_vazia() {
    let gpu = gpu_ligado(0x01, 0x260, 0xC60, 0x100, 0x110);
    let fb = gpu.framebuffer_for_display().expect("display ligado");
    assert_eq!(fb.height, 0, "linhas 256..272 ficam abaixo da area visivel");
    assert!(fb.data.is_empty(), "sem linhas, sem pixels");
}

#[test]
fn fmv_24bpp_recortada_le_bytes_a_partir_da_linha_certa() {
    let mut gpu = gpu_ligado(0x11, 0x260, 0xC60, 0x10, 0x100);
    {
        let vram = gpu.vram_raw_mut();
        vram[8 * 1024] = 0x2211;
        vram[8 * 1024 + 1] = 0x4433;
        vram[8 * 1024 + 2] = 0x6655;
    }
    gpu.enter_vblank();
    let fb = gpu.framebuffer_for_display().expect("display ligado");
    assert_eq!((fb.width, fb.height), (320, 224), "FMV 24bpp NTSC: 320x224");
    assert_eq!(pixel(&fb, 0, 0), (0x11, 0x22, 0x33), "pixel 0: bytes 0..2");
    assert_eq!(pixel(&fb, 1, 0), (0x44, 0x55, 0x66), "pixel 1: bytes 3..5");
}

#[test]
fn pal_mostra_as_288_linhas_centradas_em_a3h() {
    let gpu = gpu_ligado(0x09, 0x260, 0xC60, 0x10, 0x140);
    let fb = gpu.framebuffer_for_display().expect("display ligado");
    assert_eq!(fb.height, 288, "PAL: linhas 19..307 (A3h +/- 144)");
}
