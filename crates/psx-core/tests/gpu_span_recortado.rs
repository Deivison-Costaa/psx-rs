use psx_core::gpu::Gpu;

fn gp0_e1h(gpu: &mut Gpu, param: u32) {
    gpu.write32(0, (0xE1u32 << 24) | (param & 0x3FFF));
}

fn gp0_e3h_x1(gpu: &mut Gpu, x1: u16) {
    gpu.write32(0, (0xE3u32 << 24) | (x1 as u32 & 0x3FF));
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

// Triangulo largo (topo x=100,y=0 - meio x=0,y=50 - base x=200,y=100) escolhido
// para que, na scanline y=75, a aresta esquerda visivel comece em x=100 e a
// direita em x=175 (span original de 75px) — o suficiente pra recortar so a
// borda esquerda com GP0(E3h).X1=130 sem eliminar o pixel de amostra x=150.
const V_TOPO: (i16, i16) = (100, 0);
const V_MEIO: (i16, i16) = (0, 50);
const V_BASE: (i16, i16) = (200, 100);
const X_AMOSTRA: i32 = 150;
const Y_AMOSTRA: i32 = 75;
const X1_RECORTE: u16 = 130;

#[test]
fn gouraud_sobre_span_recortado_da_a_mesma_cor_do_span_inteiro() {
    let mut inteiro = Gpu::new();
    desenha_gouraud_triangulo(
        &mut inteiro,
        V_TOPO,
        V_MEIO,
        V_BASE,
        0x0000FF,
        0x00FF00,
        0xFF0000,
    );
    let esperado = inteiro.vram_pixel(X_AMOSTRA as u16, Y_AMOSTRA as u16);
    assert_ne!(
        esperado, 0,
        "pixel de amostra precisa estar dentro do triangulo no render sem recorte"
    );

    let mut recortado = Gpu::new();
    gp0_e3h_x1(&mut recortado, X1_RECORTE);
    desenha_gouraud_triangulo(
        &mut recortado,
        V_TOPO,
        V_MEIO,
        V_BASE,
        0x0000FF,
        0x00FF00,
        0xFF0000,
    );

    assert_eq!(
        recortado.vram_pixel(110, Y_AMOSTRA as u16),
        0,
        "sanidade: x=110 fica a esquerda de X1=130, tem que estar fora da area de desenho"
    );
    assert_eq!(
        recortado.vram_pixel(X_AMOSTRA as u16, Y_AMOSTRA as u16),
        esperado,
        "spec § Vertex, GPU Rendering Attributes (03-gpu.md L452): recortar a area de \
         desenho muda so QUAIS pixels sao visiveis, nao a cor interpolada nos \
         pixels que continuam visiveis — a cor em (150,75) tem que ser igual \
         nos dois renders"
    );
}

fn escreve_linha_15bpp(gpu: &mut Gpu, y: u16, largura: u16, texel_em: impl Fn(u16) -> u16) {
    gpu.write32(0, 0xA0u32 << 24);
    gpu.write32(0, (y as u32) << 16);
    gpu.write32(0, (1u32 << 16) | (largura as u32));
    let mut u = 0u16;
    while u < largura {
        let lo = texel_em(u) as u32;
        let hi = if u + 1 < largura {
            texel_em(u + 1) as u32
        } else {
            0
        };
        gpu.write32(0, lo | (hi << 16));
        u += 2;
    }
}

fn prepara_triangulo_texturizado_raw(gpu: &mut Gpu, vertices_uvs: &[((i16, i16), u8, u8)]) {
    let cmd: u32 = 0x2500_0000;
    gpu.write32(0, cmd);
    for (idx, &((sx, sy), u, v)) in vertices_uvs.iter().enumerate() {
        let coord_word: u32 = ((sy as u16 as u32) << 16) | (sx as u16 as u32);
        gpu.write32(0, coord_word);
        let mut uv_word: u32 = ((v as u32) << 8) | (u as u32);
        if idx == 0 {
            uv_word |= 0x0000_0000;
        } else if idx == 1 {
            let stat = gpu.stat();
            let texpage: u32 = (stat & 0x3FF) | (((stat >> 15) & 1) << 11);
            uv_word |= (texpage & 0xFF_FFFF) << 16;
        }
        gpu.write32(0, uv_word);
    }
}

#[test]
fn uv_sobre_span_recortado_amostra_o_mesmo_texel_do_span_inteiro() {
    let texel_em = |u: u16| 0x8000u16 | u;

    let mut inteiro = Gpu::new();
    escreve_linha_15bpp(&mut inteiro, 0, 256, texel_em);
    gp0_e1h(&mut inteiro, 2 << 7);
    prepara_triangulo_texturizado_raw(
        &mut inteiro,
        &[(V_TOPO, 0, 0), (V_MEIO, 100, 0), (V_BASE, 250, 0)],
    );
    let esperado = inteiro.vram_pixel(X_AMOSTRA as u16, Y_AMOSTRA as u16);
    assert_ne!(
        esperado, 0,
        "pixel de amostra precisa estar dentro do triangulo no render sem recorte"
    );

    let mut recortado = Gpu::new();
    escreve_linha_15bpp(&mut recortado, 0, 256, texel_em);
    gp0_e1h(&mut recortado, 2 << 7);
    gp0_e3h_x1(&mut recortado, X1_RECORTE);
    prepara_triangulo_texturizado_raw(
        &mut recortado,
        &[(V_TOPO, 0, 0), (V_MEIO, 100, 0), (V_BASE, 250, 0)],
    );

    assert_eq!(
        recortado.vram_pixel(110, Y_AMOSTRA as u16),
        0,
        "sanidade: x=110 fica a esquerda de X1=130, tem que estar fora da area de desenho"
    );
    assert_eq!(
        recortado.vram_pixel(X_AMOSTRA as u16, Y_AMOSTRA as u16),
        esperado,
        "spec § Vertex, GPU Rendering Attributes (03-gpu.md L452): recortar a area de \
         desenho nao pode mudar o texel amostrado nos pixels que continuam \
         visiveis — o U/V em (150,75) tem que ser igual nos dois renders"
    );
}

// Espelha o teste de U acima, mas variando V (com U fixo em 0) — o teste de U
// nao pega uma regressao isolada na interpolacao de V, porque com v0=v1=v2
// iguais o valor interpolado de V nao muda nunca, mutante ou nao.
fn escreve_coluna_15bpp(gpu: &mut Gpu, altura: u16, texel_em: impl Fn(u16) -> u16) {
    for v in 0..altura {
        gpu.write32(0, 0xA0u32 << 24);
        gpu.write32(0, (v as u32) << 16);
        gpu.write32(0, 0x0001_0001);
        gpu.write32(0, texel_em(v) as u32);
    }
}

#[test]
fn v_sobre_span_recortado_amostra_o_mesmo_texel_do_span_inteiro() {
    let texel_em = |v: u16| 0x8000u16 | v;

    let mut inteiro = Gpu::new();
    escreve_coluna_15bpp(&mut inteiro, 251, texel_em);
    gp0_e1h(&mut inteiro, 2 << 7);
    prepara_triangulo_texturizado_raw(
        &mut inteiro,
        &[(V_TOPO, 0, 0), (V_MEIO, 0, 100), (V_BASE, 0, 250)],
    );
    let esperado = inteiro.vram_pixel(X_AMOSTRA as u16, Y_AMOSTRA as u16);
    assert_ne!(
        esperado, 0,
        "pixel de amostra precisa estar dentro do triangulo no render sem recorte"
    );

    let mut recortado = Gpu::new();
    escreve_coluna_15bpp(&mut recortado, 251, texel_em);
    gp0_e1h(&mut recortado, 2 << 7);
    gp0_e3h_x1(&mut recortado, X1_RECORTE);
    prepara_triangulo_texturizado_raw(
        &mut recortado,
        &[(V_TOPO, 0, 0), (V_MEIO, 0, 100), (V_BASE, 0, 250)],
    );

    assert_eq!(
        recortado.vram_pixel(110, Y_AMOSTRA as u16),
        0,
        "sanidade: x=110 fica a esquerda de X1=130, tem que estar fora da area de desenho"
    );
    assert_eq!(
        recortado.vram_pixel(X_AMOSTRA as u16, Y_AMOSTRA as u16),
        esperado,
        "spec § Vertex, GPU Rendering Attributes (03-gpu.md L452): mesma garantia do \
         teste de U, agora para V — o texel amostrado em (150,75) tem que ser \
         igual nos dois renders"
    );
}
