use psx_core::gpu::Gpu;

fn vertice(x: u16, y: u16) -> u32 {
    ((y as u32) << 16) | x as u32
}

fn linha_mono(gpu: &mut Gpu, a: (u16, u16), b: (u16, u16)) {
    for palavra in [0x4000_0000, vertice(a.0, a.1), vertice(b.0, b.1)] {
        gpu.write32(0, palavra);
    }
}

fn linha_gouraud(gpu: &mut Gpu, a: (u16, u16), ca: u32, b: (u16, u16), cb: u32) {
    for palavra in [0x5000_0000 | ca, vertice(a.0, a.1), cb, vertice(b.0, b.1)] {
        gpu.write32(0, palavra);
    }
}

fn gpu_branca() -> Gpu {
    let mut gpu = Gpu::new();
    gpu.vram_raw_mut().fill(0x7FFF);
    gpu
}

fn pretos_na_linha(gpu: &Gpu, y: u16, x0: u16, x1: u16) -> Vec<u16> {
    (x0..=x1).filter(|&x| gpu.vram_pixel(x, y) == 0).collect()
}

#[test]
fn empate_no_meio_da_linha_ingreme_fica_na_coluna_de_partida() {
    let mut gpu = gpu_branca();
    linha_mono(&mut gpu, (118, 16), (119, 96));

    let colunas: Vec<Vec<u16>> = (54..=58)
        .map(|y| pretos_na_linha(&gpu, y, 116, 121))
        .collect();
    assert_eq!(
        colunas,
        vec![vec![118], vec![118], vec![118], vec![119], vec![119]],
        "ps1-tests gpu/lines (vram.png do hardware, y=54..58): no empate exato (t=40/80) o \
         passo de X ainda nao aconteceu; Bresenham simetrico pula uma linha antes"
    );
}

#[test]
fn linha_da_direita_para_a_esquerda_segue_o_hardware() {
    let mut gpu = gpu_branca();
    linha_mono(&mut gpu, (228, 205), (224, 216));

    let colunas: Vec<u16> = (205..=216)
        .map(|y| pretos_na_linha(&gpu, y, 220, 232)[0])
        .collect();
    assert_eq!(
        colunas,
        vec![228, 228, 227, 227, 227, 226, 226, 225, 225, 225, 224, 224],
        "ps1-tests gpu/lines (circulo, segmento (228,205)->(224,216)): colunas do hardware"
    );
}

#[test]
fn gouraud_sem_dither_interpola_como_o_hardware() {
    let mut gpu = gpu_branca();
    gpu.write32(0, 0xE100_0400);
    linha_gouraud(&mut gpu, (16, 190), 0x00_0000, (40, 190), 0x00_00FF);

    let vermelhos: Vec<u16> = (16..=40).map(|x| gpu.vram_pixel(x, 190) & 0x1F).collect();
    assert_eq!(
        vermelhos,
        vec![
            0, 1, 2, 4, 5, 6, 8, 9, 10, 12, 13, 14, 16, 17, 18, 19, 21, 22, 23, 25, 26, 27, 29, 30,
            31
        ],
        "ps1-tests gpu/lines (vram.png do hardware, y=190): preto->vermelho em 25 pixels"
    );
}

#[test]
fn gouraud_com_dither_interpola_como_o_hardware() {
    let mut gpu = gpu_branca();
    gpu.write32(0, 0xE100_0600);
    linha_gouraud(&mut gpu, (84, 190), 0x00_0000, (108, 190), 0x00_00FF);

    let vermelhos: Vec<u16> = (84..=108).map(|x| gpu.vram_pixel(x, 190) & 0x1F).collect();
    assert_eq!(
        vermelhos,
        vec![
            0, 1, 2, 4, 5, 6, 7, 9, 10, 12, 12, 14, 15, 17, 18, 19, 20, 22, 23, 25, 26, 28, 28, 30,
            31
        ],
        "ps1-tests gpu/lines (vram.png do hardware, y=190, dither ligado)"
    );
}

fn polilinha_gouraud(gpu: &mut Gpu, comando: u32, x: u16, cor3: u32) {
    let palavras = [
        comando | 0x00_00FF,
        vertice(x, 140),
        0x00_FF00,
        vertice(x + 32, 140),
        0xFF_FF00,
        vertice(x + 32, 172),
        cor3,
        vertice(x, 140),
        0x5555_5555,
    ];
    for palavra in palavras {
        gpu.write32(0, palavra);
    }
}

fn diagonal(gpu: &Gpu, x0: u16) -> Vec<u16> {
    (0..=32)
        .map(|i| gpu.vram_pixel(x0 + i, 140 + i) & 0x7FFF)
        .collect()
}

#[test]
fn polilinha_gouraud_com_dither_bate_com_o_hardware_dada_a_cor_do_quarto_vertice() {
    let mut gpu = gpu_branca();
    gpu.write32(0, 0xE100_0600);
    let cor3_da_pilha_no_hardware = 0x64_00D2;
    polilinha_gouraud(&mut gpu, 0x5C00_0000, 150, cor3_da_pilha_no_hardware);
    polilinha_gouraud(&mut gpu, 0x5E00_0000, 210, cor3_da_pilha_no_hardware);

    let opaca: Vec<u16> = vec![
        0x3019, 0x3419, 0x3438, 0x3857, 0x3876, 0x3C96, 0x3CB5, 0x40D4, 0x44F3, 0x4512, 0x4931,
        0x4D51, 0x4D70, 0x518F, 0x51AE, 0x55CD, 0x55EC, 0x5A0C, 0x5E2B, 0x5E4A, 0x6269, 0x6688,
        0x66A7, 0x6AC7, 0x6AE6, 0x6F05, 0x6F24, 0x7344, 0x7762, 0x7782, 0x7BA1, 0x7FC0, 0x7FE0,
    ];
    let transparente: Vec<u16> = vec![
        0x34FC, 0x59FC, 0x5A1B, 0x5A1B, 0x5A3A, 0x5E3A, 0x5E5A, 0x5E59, 0x6279, 0x6278, 0x6298,
        0x6698, 0x66B7, 0x66B7, 0x66D6, 0x6AD6, 0x6AF5, 0x6AF5, 0x6F15, 0x6F14, 0x6F34, 0x7333,
        0x7353, 0x7353, 0x7372, 0x7772, 0x7791, 0x7791, 0x7BB0, 0x7BB0, 0x7BD0, 0x7FCF, 0x7FE7,
    ];
    assert_eq!(
        diagonal(&gpu, 150),
        opaca,
        "ps1-tests gpu/lines (vram.png do hardware, x=150..182 y=140..172): o main.c nunca chama \
         setRGB3, e a palavra que o hardware leu da pilha foi 0x6400D2"
    );
    assert_eq!(
        diagonal(&gpu, 210),
        transparente,
        "ps1-tests gpu/lines (vram.png do hardware, x=210..242): semitransparente, com os \
         vertices compartilhados misturados duas vezes"
    );
}
