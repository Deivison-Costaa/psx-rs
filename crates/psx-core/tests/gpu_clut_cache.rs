use psx_core::gpu::Gpu;

const MODO_4BIT: u32 = 0xE100_0400;
const MODO_8BIT: u32 = 0xE100_0480;
const MODO_15BIT: u32 = 0xE100_0500;
const LIMPA_CACHE: u32 = 0x0100_0000;
const BRANCO: u16 = 0x7FFF;

fn gp0(gpu: &mut Gpu, palavras: &[u32]) {
    for &p in palavras {
        gpu.write32(0, p);
    }
}

fn clut_identidade(gpu: &mut Gpu, y: u16) {
    for x in 0..256u16 {
        gpu.vram_raw_mut()[y as usize * 1024 + x as usize] = x;
    }
}

fn textura_crescente(gpu: &mut Gpu, y: u16) {
    for x in 0..128u16 {
        let lo = x * 2;
        gpu.vram_raw_mut()[y as usize * 1024 + x as usize] = lo | ((lo + 1) << 8);
    }
}

fn textura_decrescente(gpu: &mut Gpu, y: u16) {
    for x in 0..128u16 {
        let lo = 255 - x * 2;
        gpu.vram_raw_mut()[y as usize * 1024 + x as usize] = lo | ((lo - 1) << 8);
    }
}

fn retangulo(gpu: &mut Gpu, y: u16, v_textura: u16, clut_x: u16, clut_y: u16) {
    let clut = ((clut_y as u32) << 6) | (clut_x as u32 / 16);
    gp0(
        gpu,
        &[
            0x6480_8080,
            (y as u32) << 16,
            (clut << 16) | ((v_textura as u32) << 8),
            (1 << 16) | 256,
        ],
    );
}

fn preenche_branco(gpu: &mut Gpu, y: u16) {
    gp0(gpu, &[0x02FF_FFFF, (y as u32) << 16, (1 << 16) | 256]);
}

fn linha(gpu: &Gpu, y: u16, n: u16) -> Vec<u16> {
    (0..n).map(|x| gpu.vram_pixel(x, y) & 0x7FFF).collect()
}

#[test]
fn desenhar_por_cima_da_clut_usa_a_copia_em_cache() {
    let mut gpu = Gpu::new();
    gp0(&mut gpu, &[MODO_8BIT]);
    textura_decrescente(&mut gpu, 2);
    clut_identidade(&mut gpu, 52);
    retangulo(&mut gpu, 52, 2, 0, 52);

    let esperado: Vec<u16> = (0..255).map(|x| 255 - x).chain([255]).collect();
    assert_eq!(
        linha(&gpu, 52, 256),
        esperado,
        "ps1-tests gpu/clut-cache (vram.png do hardware, y=52): a paleta e lida para o cache \
         antes do desenho; a metade direita NAO ve as entradas que a esquerda acabou de pintar. \
         O ultimo pixel (indice 0 -> 0x0000) e transparente e mantem o 255 original"
    );
}

#[test]
fn preenchimento_nao_invalida_o_cache() {
    let mut gpu = Gpu::new();
    gp0(&mut gpu, &[MODO_8BIT]);
    textura_crescente(&mut gpu, 1);
    clut_identidade(&mut gpu, 84);
    retangulo(&mut gpu, 86, 1, 0, 84);
    preenche_branco(&mut gpu, 84);
    retangulo(&mut gpu, 88, 1, 0, 84);

    assert_eq!(
        linha(&gpu, 88, 8),
        vec![0, 1, 2, 3, 4, 5, 6, 7],
        "hardware (y=88): GP0(02h) sobre a CLUT nao recarrega o cache com a mesma CLUT"
    );
}

#[test]
fn limpar_cache_forca_recarga() {
    let mut gpu = Gpu::new();
    gp0(&mut gpu, &[MODO_8BIT]);
    textura_crescente(&mut gpu, 1);
    clut_identidade(&mut gpu, 100);
    retangulo(&mut gpu, 102, 1, 0, 100);
    gp0(&mut gpu, &[0x40FF_FFFF, 100 << 16, (100 << 16) | 256]);
    gp0(&mut gpu, &[LIMPA_CACHE]);
    retangulo(&mut gpu, 104, 1, 0, 100);

    assert_eq!(
        linha(&gpu, 104, 8),
        vec![BRANCO; 8],
        "hardware (y=104): GP0(01h) invalida o cache de CLUT"
    );
}

#[test]
fn outra_posicao_de_clut_recarrega() {
    let mut gpu = Gpu::new();
    gp0(&mut gpu, &[MODO_8BIT]);
    textura_crescente(&mut gpu, 1);
    clut_identidade(&mut gpu, 116);
    retangulo(&mut gpu, 118, 1, 0, 116);
    preenche_branco(&mut gpu, 116);
    retangulo(&mut gpu, 120, 1, 16, 116);

    assert_eq!(linha(&gpu, 120, 8), vec![BRANCO; 8], "hardware (y=120)");
}

#[test]
fn carga_de_4bit_nao_serve_para_8bit() {
    let mut gpu = Gpu::new();
    textura_crescente(&mut gpu, 1);
    clut_identidade(&mut gpu, 132);
    gp0(&mut gpu, &[MODO_4BIT]);
    retangulo(&mut gpu, 134, 1, 0, 132);
    preenche_branco(&mut gpu, 132);
    gp0(&mut gpu, &[MODO_8BIT]);
    retangulo(&mut gpu, 136, 1, 0, 132);

    assert_eq!(
        linha(&gpu, 134, 8),
        vec![0, 0, 1, 0, 2, 0, 3, 0],
        "hardware (y=134): 4 bits por texel"
    );
    assert_eq!(
        linha(&gpu, 136, 8),
        vec![BRANCO; 8],
        "hardware (y=136): o cache so tinha 16 entradas; 8 bits recarrega"
    );
}

#[test]
fn carga_de_8bit_serve_para_4bit() {
    let mut gpu = Gpu::new();
    textura_crescente(&mut gpu, 1);
    clut_identidade(&mut gpu, 148);
    gp0(&mut gpu, &[MODO_8BIT]);
    retangulo(&mut gpu, 150, 1, 0, 148);
    preenche_branco(&mut gpu, 148);
    gp0(&mut gpu, &[MODO_4BIT]);
    retangulo(&mut gpu, 152, 1, 0, 148);

    assert_eq!(
        linha(&gpu, 152, 8),
        vec![0, 0, 1, 0, 2, 0, 3, 0],
        "hardware (y=152): com a mesma CLUT, o cache de 8 bits continua valido em 4 bits"
    );
}

#[test]
fn trocar_o_modo_sem_desenhar_nao_invalida() {
    let mut gpu = Gpu::new();
    textura_crescente(&mut gpu, 1);
    clut_identidade(&mut gpu, 196);
    gp0(&mut gpu, &[MODO_4BIT]);
    retangulo(&mut gpu, 198, 1, 0, 196);
    preenche_branco(&mut gpu, 196);
    gp0(&mut gpu, &[MODO_8BIT, MODO_4BIT]);
    retangulo(&mut gpu, 200, 1, 0, 196);

    assert_eq!(
        linha(&gpu, 200, 8),
        vec![0, 0, 1, 0, 2, 0, 3, 0],
        "hardware (y=200)"
    );
}

#[test]
fn modo_15bit_nao_carrega_clut() {
    let mut gpu = Gpu::new();
    textura_crescente(&mut gpu, 1);
    clut_identidade(&mut gpu, 164);
    gp0(&mut gpu, &[MODO_15BIT]);
    retangulo(&mut gpu, 166, 1, 0, 164);

    assert_eq!(
        linha(&gpu, 166, 3),
        vec![0x0100, 0x0302, 0x0504],
        "hardware (y=166): 15 bits le o texel direto"
    );
}

#[test]
fn transferencia_da_cpu_para_a_vram_recarrega_a_clut() {
    let mut gpu = Gpu::new();
    gp0(&mut gpu, &[MODO_8BIT]);
    textura_crescente(&mut gpu, 1);
    clut_identidade(&mut gpu, 300);
    retangulo(&mut gpu, 302, 1, 0, 300);
    gp0(
        &mut gpu,
        &[0xA000_0000, 300 << 16, (1 << 16) | 2, 0x0055_0044],
    );
    retangulo(&mut gpu, 304, 1, 0, 300);

    assert_eq!(
        linha(&gpu, 304, 3),
        vec![0x0044, 0x0055, 2],
        "COMPORTAMENTO ASSUMIDO: o gabarito so prova que GP0(02h) e o desenho nao invalidam; \
         jogos sobem paletas novas por DMA no mesmo lugar, entao GP0(A0h) invalida"
    );
}
