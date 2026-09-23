use psx_core::gpu::Gpu;

const DITHER: u32 = 1 << 9;
const TEXPAGE_15BPP: u32 = 0x0180;

fn vertice(x: i16, y: i16) -> u32 {
    ((y as u16 as u32) << 16) | (x as u16 as u32)
}

fn modo_de_desenho(gpu: &mut Gpu, param: u32) {
    gpu.write32(0, (0xE1u32 << 24) | param);
}

fn triangulo_gouraud(gpu: &mut Gpu, v: [(i16, i16); 3], c: [u32; 3]) {
    gpu.write32(0, (0x30u32 << 24) | c[0]);
    gpu.write32(0, vertice(v[0].0, v[0].1));
    gpu.write32(0, c[1]);
    gpu.write32(0, vertice(v[1].0, v[1].1));
    gpu.write32(0, c[2]);
    gpu.write32(0, vertice(v[2].0, v[2].1));
}

fn quad_flat(gpu: &mut Gpu, cmd: u32, cor: u32, v: [(i16, i16); 4]) {
    gpu.write32(0, (cmd << 24) | cor);
    for (x, y) in v {
        gpu.write32(0, vertice(x, y));
    }
}

fn quad_texturizado(gpu: &mut Gpu, cmd: u32, cor: u32, x0: i16, y0: i16, lado: i16) {
    gpu.write32(0, (cmd << 24) | cor);
    let cantos = [(0, 0), (lado, 0), (0, lado), (lado, lado)];
    for (i, (dx, dy)) in cantos.into_iter().enumerate() {
        gpu.write32(0, vertice(x0 + dx, y0 + dy));
        let page = if i == 1 { TEXPAGE_15BPP << 16 } else { 0 };
        gpu.write32(0, page | ((dy as u32) << 8) | dx as u32);
    }
}

fn cobertura(gpu: &Gpu, y: u16) -> Option<(u16, u16)> {
    let xs: Vec<u16> = (0..1024).filter(|&x| gpu.vram_pixel(x, y) != 0).collect();
    Some((*xs.first()?, *xs.last()?))
}

const RGB: [u32; 3] = [0x0000FF, 0x00FF00, 0xFF0000];

// (y, primeiro x, ultimo x, cor no primeiro, cor no ultimo, cor em x=160), lidos do
// vram.png de ps1-tests/gpu/triangle (hardware real).
#[rustfmt::skip]
const TRIANGULO_HW: [(u16, u16, u16, u16, u16, u16); 7] = [
    (17, 160, 160, 0x7C00, 0x7C00, 0x7C00),
    (20, 158, 162, 0x7C00, 0x7C00, 0x7C00),
    (50, 141, 179, 0x6805, 0x68A0, 0x6842),
    (100, 112, 208, 0x480C, 0x4980, 0x48C6),
    (150, 83, 237, 0x2C14, 0x2E80, 0x2D4A),
    (200, 54, 266, 0x0C1C, 0x0F80, 0x0DCE),
    (222, 41, 279, 0x001F, 0x03E0, 0x01EF),
];

#[rustfmt::skip]
const TRIANGULO_DITHER_HW: [(u16, u16, u16, u16, u16, u16); 7] = [
    (257, 160, 160, 0x7C00, 0x7C00, 0x7C00),
    (260, 158, 162, 0x7800, 0x7800, 0x7800),
    (290, 141, 179, 0x6805, 0x68A0, 0x6842),
    (340, 112, 208, 0x480C, 0x4980, 0x48C6),
    (390, 83, 237, 0x2C14, 0x2E80, 0x294A),
    (440, 54, 266, 0x0C1B, 0x0F60, 0x0DAD),
    (462, 41, 279, 0x001F, 0x03E0, 0x01EF),
];

fn confere_triangulo(gpu: &Gpu, gabarito: &[(u16, u16, u16, u16, u16, u16)]) {
    for &(y, xl, xr, cl, cr, cm) in gabarito {
        assert_eq!(
            cobertura(gpu, y),
            Some((xl, xr)),
            "cobertura da linha y={y}"
        );
        assert_eq!(gpu.vram_pixel(xl, y), cl, "cor na borda esquerda de y={y}");
        assert_eq!(gpu.vram_pixel(xr, y), cr, "cor na borda direita de y={y}");
        assert_eq!(gpu.vram_pixel(160, y), cm, "cor em x=160, y={y}");
    }
}

#[test]
fn triangulo_gouraud_bate_com_o_hardware_na_cobertura_e_na_cor() {
    let mut gpu = Gpu::new();
    triangulo_gouraud(&mut gpu, [(40, 223), (280, 223), (160, 16)], RGB);
    assert_eq!(
        cobertura(&gpu, 16),
        None,
        "a linha do vertice de cima fica vazia"
    );
    assert_eq!(cobertura(&gpu, 223), None, "a linha de baixo e excluida");
    confere_triangulo(&gpu, &TRIANGULO_HW);
}

#[test]
fn triangulo_gouraud_com_dither_bate_com_o_hardware() {
    let mut gpu = Gpu::new();
    modo_de_desenho(&mut gpu, DITHER);
    triangulo_gouraud(&mut gpu, [(40, 463), (280, 463), (160, 256)], RGB);
    assert_eq!(cobertura(&gpu, 256), None);
    assert_eq!(cobertura(&gpu, 463), None);
    confere_triangulo(&gpu, &TRIANGULO_DITHER_HW);
}

// Geometria de ps1-tests/gpu/quad: cinco quads que ladrilham [0,320)x[0,240) sem buraco.
const LADRILHOS: [[(i16, i16); 4]; 5] = [
    [(0, 0), (320, 0), (48, 48), (176, 32)],
    [(0, 0), (48, 48), (0, 240), (64, 144)],
    [(48, 48), (176, 32), (64, 144), (208, 160)],
    [(176, 32), (320, 0), (208, 160), (320, 240)],
    [(64, 144), (208, 160), (0, 240), (320, 240)],
];

#[test]
fn quads_vizinhos_cobrem_cada_pixel_exatamente_uma_vez() {
    let mut gpu = Gpu::new();
    modo_de_desenho(&mut gpu, 1 << 5);
    for quad in LADRILHOS {
        quad_flat(&mut gpu, 0x2A, 0x080808, quad);
    }
    for y in 0..240u16 {
        for x in 0..320u16 {
            assert_eq!(
                gpu.vram_pixel(x, y),
                0x0421,
                "modo aditivo com cor 1: ({x},{y}) tem que somar exatamente uma camada"
            );
        }
    }
    assert_eq!(gpu.vram_pixel(320, 100), 0);
    assert_eq!(gpu.vram_pixel(100, 240), 0);
}

const CORES_LADRILHOS: [u32; 5] = [0x00FF00, 0x0000FF, 0x000000, 0xFF0000, 0xFF00FF];

// Pixels da costura em que o dono (e portanto a media com o fundo 0x7FFF) difere entre uma
// regra de preenchimento ingenua e o hardware; valores do vram.png de ps1-tests/gpu/quad.
#[rustfmt::skip]
const COSTURAS_HW: [(u16, u16, u16); 20] = [
    (176, 33, 0x3DEF), (48, 49, 0x3DFF), (49, 56, 0x3DFF), (183, 63, 0x3DEF),
    (51, 71, 0x3DFF), (53, 79, 0x3DFF), (189, 86, 0x3DEF), (55, 94, 0x3DFF),
    (193, 101, 0x3DEF), (195, 109, 0x3DEF), (59, 117, 0x3DFF), (60, 124, 0x3DFF),
    (200, 131, 0x3DEF), (202, 139, 0x3DEF), (205, 150, 0x3DEF), (216, 166, 0x7DFF),
    (237, 181, 0x7DFF), (258, 196, 0x7DFF), (279, 211, 0x7DFF), (300, 226, 0x7DFF),
];

#[test]
fn quads_vizinhos_dividem_a_costura_como_o_hardware() {
    let mut gpu = Gpu::new();
    for y in 0..240 {
        for x in 0..320 {
            gpu.vram_raw_mut()[y * 1024 + x] = 0x7FFF;
        }
    }
    modo_de_desenho(&mut gpu, 0);
    for (quad, cor) in LADRILHOS.into_iter().zip(CORES_LADRILHOS) {
        quad_flat(&mut gpu, 0x2A, cor, quad);
    }
    for (x, y, esperado) in COSTURAS_HW {
        assert_eq!(
            gpu.vram_pixel(x, y),
            esperado,
            "dono do pixel ({x},{y}) da costura"
        );
    }
}

#[test]
fn quad_grande_so_e_descartado_pela_metade_que_passa_do_limite() {
    let mut gpu = Gpu::new();
    quad_flat(
        &mut gpu,
        0x28,
        0x0000FF,
        [(-300, 0), (300, 0), (300, 100), (800, 100)],
    );
    assert_eq!(
        gpu.vram_pixel(200, 10),
        0x001F,
        "(v0,v1,v2) tem 600 de largura"
    );
    assert_eq!(
        gpu.vram_pixel(700, 90),
        0x001F,
        "(v1,v2,v3) tem 500 de largura"
    );
}

fn textura_uniforme(gpu: &mut Gpu, texel: u16) {
    for y in 0..8 {
        for x in 0..8 {
            gpu.vram_raw_mut()[y * 1024 + x] = texel;
        }
    }
}

const TEXEL_16: u16 = 16 | (16 << 5) | (16 << 10);
const OFFSETS: [[i32; 4]; 4] = [
    [-4, 0, -3, 1],
    [2, -2, 3, -1],
    [-3, 1, -4, 0],
    [3, -1, 2, -2],
];

#[test]
fn textura_modulada_recebe_dither() {
    let mut gpu = Gpu::new();
    textura_uniforme(&mut gpu, TEXEL_16);
    modo_de_desenho(&mut gpu, DITHER);
    quad_texturizado(&mut gpu, 0x2C, 0x808080, 100, 100, 8);
    for (dy, linha) in OFFSETS.iter().enumerate() {
        for (dx, &off) in linha.iter().enumerate() {
            let canal = (((16 * 128) >> 4) + off) as u16 >> 3;
            let esperado = canal | (canal << 5) | (canal << 10);
            let (x, y) = (100 + dx as u16, 100 + dy as u16);
            assert_eq!(gpu.vram_pixel(x, y), esperado, "({x},{y}) offset {off}");
        }
    }
}

#[test]
fn textura_crua_e_textura_sem_dither_nao_mudam() {
    let mut crua = Gpu::new();
    textura_uniforme(&mut crua, TEXEL_16);
    modo_de_desenho(&mut crua, DITHER);
    quad_texturizado(&mut crua, 0x2D, 0x808080, 100, 100, 8);
    let mut sem_dither = Gpu::new();
    textura_uniforme(&mut sem_dither, TEXEL_16);
    quad_texturizado(&mut sem_dither, 0x2C, 0x808080, 100, 100, 8);
    for y in 100..104 {
        for x in 100..104 {
            assert_eq!(crua.vram_pixel(x, y), TEXEL_16, "crua ({x},{y})");
            assert_eq!(
                sem_dither.vram_pixel(x, y),
                TEXEL_16,
                "sem dither ({x},{y})"
            );
        }
    }
}

#[test]
fn gouraud_com_dither_nao_reinterpola_sobre_o_span_recortado() {
    let desenha = |x1: u32| {
        let mut gpu = Gpu::new();
        modo_de_desenho(&mut gpu, DITHER);
        gpu.write32(0, (0xE3u32 << 24) | x1);
        triangulo_gouraud(&mut gpu, [(100, 0), (0, 50), (200, 100)], RGB);
        gpu
    };
    let inteiro = desenha(0);
    let recortado = desenha(130);
    assert_eq!(recortado.vram_pixel(129, 75), 0);
    for x in 130..175 {
        assert_eq!(
            recortado.vram_pixel(x, 75),
            inteiro.vram_pixel(x, 75),
            "a cor em ({x},75) nao depende de onde a area de desenho corta o span"
        );
    }
}
