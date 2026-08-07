use psx_core::gpu::Gpu;

fn escreve_vram_halfword(gpu: &mut Gpu, x: u16, y: u16, val: u16) {
    gpu.write32(0, 0xA0u32 << 24);
    gpu.write32(0, (y as u32) << 16 | (x as u32));
    gpu.write32(0, 0x0001_0001);
    gpu.write32(0, val as u32);
}

fn draw_mode(gpu: &mut Gpu, param: u32) {
    gpu.write32(0, (0xE1 << 24) | (param & 0x3FFF));
}

fn retangulo_texturizado(gpu: &mut Gpu, x: i16, y: i16, u: u8, v: u8, w: u16, h: u16) {
    gpu.write32(0, 0x6500_0000);
    gpu.write32(0, ((y as u16 as u32) << 16) | (x as u16 as u32));
    gpu.write32(0, ((v as u32) << 8) | (u as u32));
    gpu.write32(0, ((h as u32) << 16) | (w as u32));
}

fn prepara_linha_de_texels(gpu: &mut Gpu) {
    for i in 0..6u16 {
        escreve_vram_halfword(gpu, i, 0, 0x1000 + i);
    }
}

fn prepara_coluna_de_texels(gpu: &mut Gpu) {
    for i in 0..6u16 {
        escreve_vram_halfword(gpu, 0, i, 0x2000 + i);
    }
}

const TEXPAGE_15BPP: u32 = 2 << 7;
const X_FLIP: u32 = 1 << 12;
const Y_FLIP: u32 = 1 << 13;

#[rustfmt::skip]
#[test]
fn a1_sem_flip_o_u_e_incrementado_da_esquerda_para_a_direita() {
    let mut gpu = Gpu::new();
    prepara_linha_de_texels(&mut gpu);
    draw_mode(&mut gpu, TEXPAGE_15BPP);

    retangulo_texturizado(&mut gpu, 10, 10, 0, 0, 4, 1);

    for i in 0..4u16 {
        assert_eq!(
            gpu.vram_pixel(10 + i, 10), 0x1000 + i,
            "A1 sem flip: pixel(x={}) tem de amostrar texel u={} (0x{:04X}), obtido 0x{:04X}",
            10 + i, i, 0x1000 + i, gpu.vram_pixel(10 + i, 10)
        );
    }
}

#[rustfmt::skip]
#[test]
fn a2_x_flip_do_e1h_bit12_percorre_u_para_tras_comecando_em_u_mais_um() {
    let mut gpu = Gpu::new();
    prepara_linha_de_texels(&mut gpu);
    draw_mode(&mut gpu, TEXPAGE_15BPP | X_FLIP);

    retangulo_texturizado(&mut gpu, 10, 10, 3, 0, 4, 1);

    for i in 0..4u16 {
        let esperado = 0x1000 + (4 - i);
        assert_eq!(
            gpu.vram_pixel(10 + i, 10), esperado,
            "A2 X-Flip (GP0(E1h).12): 03-gpu.md L421-423 diz que o bit faz a coordenada de \
             textura ser DECREMENTADA em vez de incrementada, e o gabarito de hardware \
             tests/exes/ps1-tests/gpu/texture-flip/vram.png mede a sequencia comecando em \
             u+1 (a coluna inteira do quadrante espelhado fica deslocada de um texel sem \
             ele). Com u base=3 e largura 4, o pixel(x={}) tem de amostrar texel u={} \
             (0x{:04X}); obtido 0x{:04X}",
            10 + i, 4 - i, esperado, gpu.vram_pixel(10 + i, 10)
        );
    }
}

#[rustfmt::skip]
#[test]
fn a3_y_flip_do_e1h_bit13_percorre_v_para_tras_comecando_no_proprio_v() {
    let mut gpu = Gpu::new();
    prepara_coluna_de_texels(&mut gpu);
    draw_mode(&mut gpu, TEXPAGE_15BPP | Y_FLIP);

    retangulo_texturizado(&mut gpu, 10, 10, 0, 3, 1, 4);

    for i in 0..4u16 {
        let esperado = 0x2000 + (3 - i);
        assert_eq!(
            gpu.vram_pixel(10, 10 + i), esperado,
            "A3 Y-Flip (GP0(E1h).13): 03-gpu.md L421-423. O eixo vertical NAO tem o \
             deslocamento de um texel do eixo horizontal — o gabarito de hardware \
             texture-flip/vram.png bate exatamente com v base decrescente. Com v base=3 e \
             altura 4, o pixel(y={}) tem de amostrar texel v={} (0x{:04X}); obtido 0x{:04X}",
            10 + i, 3 - i, esperado, gpu.vram_pixel(10, 10 + i)
        );
    }
}

#[rustfmt::skip]
#[test]
fn a4_flip_nao_afeta_poligono_texturizado() {
    let mut gpu = Gpu::new();
    prepara_linha_de_texels(&mut gpu);
    draw_mode(&mut gpu, TEXPAGE_15BPP | X_FLIP | Y_FLIP);

    gpu.write32(0, 0x2500_0000);
    gpu.write32(0, (10_u32 << 16) | 10_u32);
    gpu.write32(0, 0x0080_0000);
    gpu.write32(0, (10_u32 << 16) | 14_u32);
    let stat = gpu.stat();
    let texpage: u32 = (stat & 0x3FF) | (((stat >> 15) & 1) << 11);
    gpu.write32(0, (texpage << 16) | 4);
    gpu.write32(0, (14_u32 << 16) | 10_u32);
    gpu.write32(0, 4 << 8);

    assert_eq!(
        gpu.vram_pixel(10, 10), 0x1000,
        "A4: 03-gpu.md L423 — os bits de X/Y-Flip afetam SO retangulos, nao poligonos. \
         Com flip ligado, o poligono ainda amostra texel u=0 em (10,10); obtido 0x{:04X}",
        gpu.vram_pixel(10, 10)
    );
}

#[rustfmt::skip]
#[test]
fn a5_gp1_00h_reset_limpa_os_bits_de_flip() {
    let mut gpu = Gpu::new();
    prepara_linha_de_texels(&mut gpu);
    draw_mode(&mut gpu, TEXPAGE_15BPP | X_FLIP);

    gpu.write32(4, 0x0000_0000);
    draw_mode(&mut gpu, TEXPAGE_15BPP);

    retangulo_texturizado(&mut gpu, 10, 10, 0, 0, 4, 1);

    assert_eq!(
        gpu.vram_pixel(13, 10), 0x1003,
        "A5: GP1(00h) reseta o draw mode, logo o X-Flip anterior nao pode sobreviver; \
         pixel(13) tem de amostrar texel u=3 (0x1003), obtido 0x{:04X}",
        gpu.vram_pixel(13, 10)
    );
}
