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
