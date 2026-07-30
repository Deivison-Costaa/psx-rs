use psx_core::gpu::Gpu;

const COPIA: u32 = 0x80 << 24;

fn coord(x: u16, y: u16) -> u32 {
    ((y as u32) << 16) | x as u32
}

fn pinta(gpu: &mut Gpu, x: u16, y: u16, cor: u16) {
    gpu.vram_raw_mut()[y as usize * 1024 + x as usize] = cor;
}

fn blit(gpu: &mut Gpu, src: u32, dst: u32, tam: u32) {
    gpu.write32(0, COPIA);
    gpu.write32(0, src);
    gpu.write32(0, dst);
    gpu.write32(0, tam);
}

#[test]
fn copia_retangulo_dentro_da_vram() {
    let mut gpu = Gpu::new();
    pinta(&mut gpu, 10, 20, 0x1234);
    pinta(&mut gpu, 11, 20, 0x5678);
    pinta(&mut gpu, 10, 21, 0x2468);
    pinta(&mut gpu, 11, 21, 0x1357);

    blit(&mut gpu, coord(10, 20), coord(100, 200), coord(2, 2));

    assert_eq!(gpu.vram_pixel(100, 200), 0x1234, "GP0(80h) copia o pixel de origem");
    assert_eq!(gpu.vram_pixel(101, 200), 0x5678);
    assert_eq!(gpu.vram_pixel(100, 201), 0x2468);
    assert_eq!(gpu.vram_pixel(101, 201), 0x1357);
    assert_eq!(
        gpu.vram_pixel(10, 20),
        0x1234,
        "a origem continua intacta: e copia, nao movimento"
    );
}

#[test]
fn nao_escreve_fora_do_retangulo_pedido() {
    let mut gpu = Gpu::new();
    pinta(&mut gpu, 10, 20, 0x7FFF);

    blit(&mut gpu, coord(10, 20), coord(100, 200), coord(1, 1));

    assert_eq!(gpu.vram_pixel(100, 200), 0x7FFF);
    assert_eq!(gpu.vram_pixel(101, 200), 0, "coluna seguinte fica intacta");
    assert_eq!(gpu.vram_pixel(100, 201), 0, "linha seguinte fica intacta");
    assert_eq!(gpu.vram_pixel(99, 200), 0, "coluna anterior fica intacta");
}

#[test]
fn coordenadas_sao_absolutas_e_ignoram_o_drawing_offset() {
    let mut gpu = Gpu::new();
    gpu.write32(0, (0xE5u32 << 24) | (40u32 << 11) | 30);
    pinta(&mut gpu, 10, 20, 0x0AAA);

    blit(&mut gpu, coord(10, 20), coord(100, 200), coord(1, 1));

    assert_eq!(
        gpu.vram_pixel(100, 200),
        0x0AAA,
        "`docs/reference/03-gpu.md` L692-693: as coordenadas das transferencias de VRAM sao \
         enderecos absolutos do framebuffer, NAO relativos ao Draw Offset"
    );
    assert_eq!(
        gpu.vram_pixel(130, 240),
        0,
        "se o offset tivesse sido somado, o destino teria caido aqui"
    );
}

#[test]
fn coordenadas_nao_sao_recortadas_pela_drawing_area() {
    let mut gpu = Gpu::new();
    gpu.write32(0, (0xE3u32 << 24) | (0u32 << 10) | 0);
    gpu.write32(0, (0xE4u32 << 24) | (5u32 << 10) | 5);
    pinta(&mut gpu, 10, 20, 0x0BBB);

    blit(&mut gpu, coord(10, 20), coord(100, 200), coord(1, 1));

    assert_eq!(
        gpu.vram_pixel(100, 200),
        0x0BBB,
        "`docs/reference/03-gpu.md` L692-693: nao sao recortadas pela Draw Area. A area foi posta \
         em 0..5 e o destino esta muito fora dela; recortar aqui apagaria o blit inteiro"
    );
}

#[test]
fn tamanho_zero_vale_o_maximo() {
    let mut gpu = Gpu::new();
    pinta(&mut gpu, 0, 0, 0x3FFF);
    pinta(&mut gpu, 1023, 0, 0x2AAA);

    blit(&mut gpu, coord(0, 0), coord(0, 300), coord(0, 1));

    assert_eq!(
        gpu.vram_pixel(1023, 300),
        0x2AAA,
        "`docs/reference/03-gpu.md` L669-670: Xsiz=((Xsiz-1) AND 3FFh)+1, entao Size=0 vale o \
         maximo (400h = 1024 halfwords), nao zero"
    );
    assert_eq!(gpu.vram_pixel(0, 300), 0x3FFF);
}

#[test]
fn posicoes_sao_mascaradas_em_10_e_9_bits() {
    let mut gpu = Gpu::new();
    pinta(&mut gpu, 5, 7, 0x0CCC);

    blit(&mut gpu, coord(0x405, 0x207), coord(100, 200), coord(1, 1));

    assert_eq!(
        gpu.vram_pixel(100, 200),
        0x0CCC,
        "`docs/reference/03-gpu.md` L667-668: Xpos=(Xpos AND 3FFh), Ypos=(Ypos AND 1FFh). \
         Origem 0x405,0x207 mascara para 5,7"
    );
}

#[test]
fn copia_envolve_na_borda_sem_carry_de_x_para_y() {
    let mut gpu = Gpu::new();
    pinta(&mut gpu, 1023, 100, 0x0111);
    pinta(&mut gpu, 0, 100, 0x0222);

    blit(&mut gpu, coord(1023, 100), coord(1023, 200), coord(2, 1));

    assert_eq!(gpu.vram_pixel(1023, 200), 0x0111);
    assert_eq!(
        gpu.vram_pixel(0, 200),
        0x0222,
        "`docs/reference/03-gpu.md` L697-700: passar da borda envolve para a borda oposta SEM \
         carry de X para Y. O segundo pixel volta para x=0 da MESMA linha (y=200), nao para a \
         linha seguinte"
    );
    assert_eq!(
        gpu.vram_pixel(0, 201),
        0,
        "se houvesse carry de X para Y, o pixel teria caido aqui"
    );
}

#[test]
fn origem_e_lida_antes_de_qualquer_escrita_em_regiao_sobreposta() {
    let mut gpu = Gpu::new();
    for x in 0..4u16 {
        pinta(&mut gpu, x, 50, 0x0100 + x);
    }

    blit(&mut gpu, coord(0, 50), coord(1, 50), coord(4, 1));

    assert_eq!(
        [
            gpu.vram_pixel(1, 50),
            gpu.vram_pixel(2, 50),
            gpu.vram_pixel(3, 50),
            gpu.vram_pixel(4, 50),
        ],
        [0x0100, 0x0101, 0x0102, 0x0103],
        "COMPORTAMENTO ASSUMIDO: a spec nao diz a ordem de varredura do GP0(80h), entao regiao \
         sobreposta e indefinida por ela. Fixamos 'le tudo antes de escrever', que e o unico \
         resultado que nao depende da ordem. Copia in-place linha a linha da esquerda para a \
         direita arrastaria 0x0100 pelos quatro pixels — se algum teste de hardware disser que e \
         isso que acontece, este teste e que muda. Ver invariante 18."
    );
}
