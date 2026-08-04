use psx_core::gpu::Gpu;

fn escreve_vram_halfword(gpu: &mut Gpu, x: u16, y: u16, val: u16) {
    gpu.write32(0, 0xA0u32 << 24);
    gpu.write32(0, (y as u32) << 16 | (x as u32));
    gpu.write32(0, 0x0001_0001);
    gpu.write32(0, val as u32);
}

fn aplica_e1h(gpu: &mut Gpu, param: u32) {
    gpu.write32(0, (0xE1 << 24) | (param & 0x3FFF));
}

fn rect_texturizado(gpu: &mut Gpu, cmd: u32, x: i16, y: i16, u: u8, v: u8, clut_attr: u16) {
    gpu.write32(0, cmd);
    gpu.write32(0, ((y as u16 as u32) << 16) | (x as u16 as u32));
    gpu.write32(0, ((clut_attr as u32) << 16) | ((v as u32) << 8) | (u as u32));
}

const E1_15BPP: u32 = 2 << 7;
const RECT_8X8_RAW: u32 = 0x7500_0000;
const RECT_1X1_RAW: u32 = 0x6D00_0000;
const RECT_VAR_RAW: u32 = 0x6500_0000;
const RECT_8X8_RAW_SEMI: u32 = 0x7700_0000;

#[test]
fn rect_15bpp_raw_copia_texels_da_pagina() {
    let mut gpu = Gpu::new();
    aplica_e1h(&mut gpu, E1_15BPP);
    escreve_vram_halfword(&mut gpu, 0, 0, 0x7C1F);
    escreve_vram_halfword(&mut gpu, 7, 0, 0x03E0);
    escreve_vram_halfword(&mut gpu, 7, 7, 0x1234);

    rect_texturizado(&mut gpu, RECT_8X8_RAW, 100, 100, 0, 0, 0);

    assert_eq!(gpu.vram_pixel(100, 100), 0x7C1F, "texel (0,0) no canto");
    assert_eq!(gpu.vram_pixel(107, 100), 0x03E0, "texel (7,0) na borda direita");
    assert_eq!(gpu.vram_pixel(107, 107), 0x1234, "texel (7,7) no canto oposto");
}

#[test]
fn texel_zero_e_transparente_e_preserva_o_fundo() {
    let mut gpu = Gpu::new();
    aplica_e1h(&mut gpu, E1_15BPP);
    escreve_vram_halfword(&mut gpu, 0, 0, 0x7FFF);
    escreve_vram_halfword(&mut gpu, 101, 100, 0x5A5A);

    rect_texturizado(&mut gpu, RECT_8X8_RAW, 100, 100, 0, 0, 0);

    assert_eq!(gpu.vram_pixel(100, 100), 0x7FFF, "texel nao-zero desenha");
    assert_eq!(
        gpu.vram_pixel(101, 100),
        0x5A5A,
        "texel 0x0000 e totalmente transparente: o fundo fica"
    );
}

#[test]
fn rect_1x1_desenha_um_unico_pixel() {
    let mut gpu = Gpu::new();
    aplica_e1h(&mut gpu, E1_15BPP);
    escreve_vram_halfword(&mut gpu, 3, 2, 0x2222);
    escreve_vram_halfword(&mut gpu, 201, 200, 0x1111);

    rect_texturizado(&mut gpu, RECT_1X1_RAW, 200, 200, 3, 2, 0);

    assert_eq!(gpu.vram_pixel(200, 200), 0x2222, "UV (3,2) amostrado");
    assert_eq!(gpu.vram_pixel(201, 200), 0x1111, "vizinho intocado");
}

#[test]
fn rect_variavel_respeita_largura_e_altura() {
    let mut gpu = Gpu::new();
    aplica_e1h(&mut gpu, E1_15BPP);
    for u in 0..3u16 {
        for v in 0..2u16 {
            escreve_vram_halfword(&mut gpu, u, v, 0x4000 | (v << 4) | u);
        }
    }
    escreve_vram_halfword(&mut gpu, 303, 300, 0x0FED);
    escreve_vram_halfword(&mut gpu, 300, 302, 0x0ABC);

    gpu.write32(0, RECT_VAR_RAW);
    gpu.write32(0, (300u32 << 16) | 300);
    gpu.write32(0, 0);
    gpu.write32(0, (2u32 << 16) | 3);

    assert_eq!(gpu.vram_pixel(300, 300), 0x4000, "texel (0,0)");
    assert_eq!(gpu.vram_pixel(302, 301), 0x4012, "texel (2,1)");
    assert_eq!(gpu.vram_pixel(303, 300), 0x0FED, "fora da largura 3: intocado");
    assert_eq!(gpu.vram_pixel(300, 302), 0x0ABC, "fora da altura 2: intocado");
}

#[test]
fn clut_4bpp_vem_do_uv_word_do_rect() {
    let mut gpu = Gpu::new();
    aplica_e1h(&mut gpu, 0);
    escreve_vram_halfword(&mut gpu, 0, 0, 0x0053);
    let clut_attr: u16 = 4 | (200 << 6);
    escreve_vram_halfword(&mut gpu, 64 + 3, 200, 0x7001);
    escreve_vram_halfword(&mut gpu, 64 + 5, 200, 0x7002);

    rect_texturizado(&mut gpu, RECT_8X8_RAW, 400, 100, 0, 0, clut_attr);

    assert_eq!(gpu.vram_pixel(400, 100), 0x7001, "nibble 3 -> CLUT[3]");
    assert_eq!(gpu.vram_pixel(401, 100), 0x7002, "nibble 5 -> CLUT[5]");
}

#[test]
fn semi_transparencia_obedece_o_bit_stp_do_texel() {
    let mut gpu = Gpu::new();
    aplica_e1h(&mut gpu, E1_15BPP);
    escreve_vram_halfword(&mut gpu, 0, 0, 0x8000 | (16 << 10));
    escreve_vram_halfword(&mut gpu, 1, 0, 16 << 10);
    escreve_vram_halfword(&mut gpu, 500, 200, 16);
    escreve_vram_halfword(&mut gpu, 501, 200, 16);

    rect_texturizado(&mut gpu, RECT_8X8_RAW_SEMI, 500, 200, 0, 0, 0);

    assert_eq!(
        gpu.vram_pixel(500, 200),
        0x8000 | (8 << 10) | 8,
        "STP=1 blenda B/2+F/2 com o fundo"
    );
    assert_eq!(
        gpu.vram_pixel(501, 200),
        16 << 10,
        "STP=0 substitui mesmo com o bit 25 do comando ligado"
    );
}

#[test]
fn uv_acima_de_255_da_volta_na_janela_de_256() {
    let mut gpu = Gpu::new();
    aplica_e1h(&mut gpu, E1_15BPP);
    escreve_vram_halfword(&mut gpu, 254, 0, 0x0111);
    escreve_vram_halfword(&mut gpu, 255, 0, 0x0222);
    escreve_vram_halfword(&mut gpu, 0, 0, 0x0333);
    escreve_vram_halfword(&mut gpu, 1, 0, 0x0444);

    gpu.write32(0, RECT_VAR_RAW);
    gpu.write32(0, (400u32 << 16) | 600);
    gpu.write32(0, 254);
    gpu.write32(0, (1u32 << 16) | 4);

    assert_eq!(gpu.vram_pixel(600, 400), 0x0111, "u=254");
    assert_eq!(gpu.vram_pixel(601, 400), 0x0222, "u=255");
    assert_eq!(gpu.vram_pixel(602, 400), 0x0333, "u=256 vira 0");
    assert_eq!(gpu.vram_pixel(603, 400), 0x0444, "u=257 vira 1");
}
