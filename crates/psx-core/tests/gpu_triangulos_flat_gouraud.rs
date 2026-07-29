use psx_core::gpu::Gpu;

#[rustfmt::skip]
#[test]
fn flat_triangle_de_3_vertices_preenche_pixels_internos() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x2000_F818;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x000A_000A);
    gpu.write32(0, 0x000A_001E);
    gpu.write32(0, 0x001E_000A);

    let expected: u16 = 0x03E3;
    assert_eq!(gpu.vram_pixel(11, 11), expected,
        "B1: pixel(11,11) interior deve ser 0x03E3, obtido 0x{:04X}", gpu.vram_pixel(11, 11));
    assert_eq!(gpu.vram_pixel(15, 15), expected,
        "B1: pixel(15,15) interior deve ser 0x03E3, obtido 0x{:04X}", gpu.vram_pixel(15, 15));
    assert_eq!(gpu.vram_pixel(10, 15), expected,
        "B1: pixel(10,15) borda esquerda deve ser 0x03E3, obtido 0x{:04X}", gpu.vram_pixel(10, 15));
    assert_eq!(gpu.vram_pixel(9, 10), 0,
        "B1: pixel(9,10) fora a esquerda");
    assert_eq!(gpu.vram_pixel(10, 9), 0,
        "B1: pixel(10,9) fora acima");
    assert_eq!(gpu.vram_pixel(11, 31), 0,
        "B1: pixel(11,31) fora abaixo");
    assert_eq!(gpu.vram_pixel(10, 30), 0,
        "B1: pixel(10,30) vertice inferior excluido pela regra lower-right (L323)");
}

#[rustfmt::skip]
#[test]
fn flat_triangle_inclui_borda_superior_e_esquerda() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x2000_FFFF;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x000A_000A);
    gpu.write32(0, 0x000A_001E);
    gpu.write32(0, 0x001E_000A);

    assert_eq!(gpu.vram_pixel(10, 10), 0x03FF,
        "B2: pixel(10,10) canto superior-esquerdo deve ser 0x03FF");
    assert_eq!(gpu.vram_pixel(15, 10), 0x03FF,
        "B2: pixel(15,10) borda superior deve ser 0x03FF");
}

#[rustfmt::skip]
#[test]
fn flat_triangle_de_2x2_preenche_apenas_3_pixels() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x2000_FFFF;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0000_0002);
    gpu.write32(0, 0x0002_0000);

    assert_eq!(gpu.vram_pixel(0, 0), 0x03FF,
        "B3: pixel(0,0) incluso");
    assert_eq!(gpu.vram_pixel(1, 0), 0x03FF,
        "B3: pixel(1,0) incluso");
    assert_eq!(gpu.vram_pixel(0, 1), 0x03FF,
        "B3: pixel(0,1) incluso");

    let cheios = [gpu.vram_pixel(0,0), gpu.vram_pixel(1,0), gpu.vram_pixel(0,1), gpu.vram_pixel(1,1)];
    let preenchidos: u32 = cheios.iter().filter(|&&p| p != 0).count() as u32;
    assert_eq!(preenchidos, 3,
        "B3: exatamente 3 pixels preenchidos (lower-right excluido, L323), obtidos {}", preenchidos);

    assert_eq!(gpu.vram_pixel(2, 0), 0,
        "B3: y=0, x=2 (aresta inferior-direita, x=xr) excluido");
    assert_eq!(gpu.vram_pixel(0, 2), 0,
        "B3: y=2 (linha inferior, y=yb) toda excluida");
    assert_eq!(gpu.vram_pixel(1, 1), 0,
        "B3: y=1, x=1 (aresta inferior-direita, hipotenusa) excluido");
}

#[rustfmt::skip]
#[test]
fn flat_quad_de_4_vertices_preenche_como_dois_triangulos() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x2800_F818;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x000A_000A);
    gpu.write32(0, 0x000A_001E);
    gpu.write32(0, 0x001E_001E);
    gpu.write32(0, 0x001E_000A);

    let expected: u16 = 0x03E3;
    assert_eq!(gpu.vram_pixel(12, 12), expected,
        "B4: pixel(12,12) no triangulo V0V1V2 (acima da diagonal), obtido 0x{:04X}",
        gpu.vram_pixel(12, 12));
    assert_eq!(gpu.vram_pixel(25, 18), expected,
        "B4: pixel(25,18) no triangulo V1V2V3 (abaixo da diagonal), obtido 0x{:04X}",
        gpu.vram_pixel(25, 18));
    assert_eq!(gpu.vram_pixel(5, 5), 0,
        "B4: pixel(5,5) fora do quad");
}

#[rustfmt::skip]
#[test]
fn gouraud_triangle_cor_no_vertice_e_exata() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x3000_0000 | 0xF8;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x000A_000A);
    gpu.write32(0, 0x00_F8_00_00);
    gpu.write32(0, 0x000A_001E);
    gpu.write32(0, 0x00_00_F8_00);
    gpu.write32(0, 0x001E_000A);

    assert_eq!(gpu.vram_pixel(10, 10), 0x001F,
        "B5-G: vertice0 (10,10) deve ser 0x001F, obtido 0x{:04X}", gpu.vram_pixel(10, 10));
    let v0_color = gpu.vram_pixel(12, 12);
    assert!(v0_color != 0,
        "B5-G: pixels interiores devem ser coloridos, obtido 0x{:04X}", v0_color);
    let vtx_colors = [
        gpu.vram_pixel(10, 10),
        gpu.vram_pixel(10, 29),
        gpu.vram_pixel(29, 10),
    ];
    let filled: u32 = vtx_colors.iter().filter(|&&c| c != 0).count() as u32;
    assert!(filled >= 2,
        "B5-G: pelo menos 2 dos 3 vertices visiveis, obtidos {}", filled);
}

#[rustfmt::skip]
#[test]
fn gouraud_triangle_interpola_para_ponto_medio_da_aresta_superior() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x3000_0000 | 0xF8;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x000A_000A);
    gpu.write32(0, 0x00_00_F8_00);
    gpu.write32(0, 0x000A_001E);
    gpu.write32(0, 0x00_00_00_F8);
    gpu.write32(0, 0x001E_000A);

    let val = gpu.vram_pixel(20, 10);
    assert!(val != 0,
        "B6-G: pixel (20,10) deve ser colorido, obtido 0x{:04X}", val);
}

#[rustfmt::skip]
#[test]
fn gouraud_triangle_interpola_na_aresta_longa_entre_cores_diferentes() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x3000_0000 | 0xF8;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x000A_000A);
    gpu.write32(0, 0x00_F8_00_00);
    gpu.write32(0, 0x000A_001E);
    gpu.write32(0, 0x00_00_F8_00);
    gpu.write32(0, 0x001E_000A);

    assert_eq!(gpu.vram_pixel(10, 10), 0x001F,
        "B7: pixel(10,10) vertice no extremo superior (t=0): 0x001F, obtido 0x{:04X}",
        gpu.vram_pixel(10, 10));
    assert_eq!(gpu.vram_pixel(10, 20), 0x01F0,
        "B7: pixel(10,20) aresta vertical x=10 red->green (t=10, t_max=20), obtido 0x{:04X}",
        gpu.vram_pixel(10, 20));
}

#[rustfmt::skip]
#[test]
fn gouraud_triangle_produz_cores_diferentes_em_pontos_distintos() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x3000_0000 | 0xF8;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x000A_000A);
    gpu.write32(0, 0x00_F8_00_00);
    gpu.write32(0, 0x000A_001E);
    gpu.write32(0, 0x00_00_F8_00);
    gpu.write32(0, 0x001E_000A);

    let val_esq = gpu.vram_pixel(10, 15);
    let val_dir = gpu.vram_pixel(20, 10);
    assert!(val_esq != 0 && val_dir != 0,
        "B7b-G: ambos pixels devem ser coloridos, esq=0x{:04X} dir=0x{:04X}", val_esq, val_dir);
    assert_ne!(val_esq, val_dir,
        "B7b-G: cores DEVEM ser diferentes em pontos distintos (gouraud), obtidos 0x{:04X} e 0x{:04X}",
        val_esq, val_dir);
}

#[rustfmt::skip]
#[test]
fn gouraud_short_edge_interpola_entre_cores_diferentes() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x3000_0000 | 0xF8;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x00_00_F8_00);
    gpu.write32(0, (0x000A << 16) | 0x0005);
    gpu.write32(0, 0x00_F8_00_00);
    gpu.write32(0, 0x0000_000A);

    assert_eq!(gpu.vram_pixel(0, 0), 0x001F,
        "B8s: vertice0 (0,0) vermelho 0x001F, obtido 0x{:04X}", gpu.vram_pixel(0, 0));
    assert_eq!(gpu.vram_pixel(1, 3), 0x0136,
        "B8s: aresta esquerda (x,y)=(1,3) red->green a 30%% (t=3,dx=10): 0x0136, obtido 0x{:04X}",
        gpu.vram_pixel(1, 3));
    assert_eq!(gpu.vram_pixel(3, 7), 0x02AA,
        "B8s: aresta esquerda (x,y)=(3,7) red->green a 70%% (t=7,dx=10): 0x02AA, obtido 0x{:04X}",
        gpu.vram_pixel(3, 7));
    assert_eq!(gpu.vram_pixel(7, 4), 0x3D84,
        "B8s: pixel(7,4) aresta curta direita blue->green interpolada: 0x3D84, obtido 0x{:04X}",
        gpu.vram_pixel(7, 4));
}

#[rustfmt::skip]
#[test]
fn gouraud_quad_interpola_cores_nos_vertices() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x3800_0000 | 0xF8;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x000A_000A);
    gpu.write32(0, 0x00_00_F8_00);
    gpu.write32(0, 0x000A_001E);
    gpu.write32(0, 0xF8_00_00_00);
    gpu.write32(0, 0x001E_001E);
    gpu.write32(0, 0x00_00_00_F8);
    gpu.write32(0, 0x001E_000A);

    assert_ne!(gpu.vram_pixel(10, 10), 0,
        "B8-G: V0 (10,10) deve ser colorido");
    assert_ne!(gpu.vram_pixel(12, 12), 0,
        "B8-G: pixel(12,12) interiores devem ser coloridos");
    assert_ne!(gpu.vram_pixel(25, 18), 0,
        "B8-G: pixel(25,18) no triangulo V1V2V3");
}

#[rustfmt::skip]
#[test]
fn bit26_cai_entre_as_palavras_de_vertice_do_poligono() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x2000_F818;
    gpu.write32(0, cmd);
    assert_eq!((gpu.read32(4) >> 26) & 1, 0,
        "GP0(polygon) recebido, faltam vertices: bit26=0");

    gpu.write32(0, 0x000A_000A);
    assert_eq!((gpu.read32(4) >> 26) & 1, 0,
        "vertice0 recebido, faltam 2 vertices: bit26=0");

    gpu.write32(0, 0x000A_001E);
    assert_eq!((gpu.read32(4) >> 26) & 1, 0,
        "vertice1 recebido, falta 1 vertice: bit26=0");

    gpu.write32(0, 0x001E_000A);
    assert_eq!((gpu.read32(4) >> 26) & 1, 1,
        "vertice2 recebido, poligono completo: bit26=1 (ready)");
}

#[rustfmt::skip]
#[test]
fn polygon_vertices_fora_da_vram_nao_causam_panic() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x2000_F818;
    gpu.write32(0, cmd);
    let x0: u16 = (-16i16) as u16;
    let x1: u16 = 15u16;
    let x2: u16 = 16u16;
    gpu.write32(0, (10u32 << 16) | x0 as u32);
    gpu.write32(0, (15u32 << 16) | x1 as u32);
    gpu.write32(0, (20u32 << 16) | x2 as u32);

    assert_eq!(gpu.vram_pixel(0, 13), 0x03E3,
        "B10: x negativo recortado, pixel(0,13) pintado (primeira scanline com x>=0)");
    assert_ne!(gpu.vram_pixel(5, 15), 0,
        "B10: pixel(5,15) interior pintado dentro da VRAM");

    let mut gpu = Gpu::new();
    let cmd: u32 = 0x2000_F818;
    gpu.write32(0, cmd);
    gpu.write32(0, (0x00F5 << 16) | 0x03F0);
    gpu.write32(0, (0x00FA << 16) | 0x03F0);
    gpu.write32(0, (0x00F7 << 16) | 0x03FE);

    assert_eq!(gpu.vram_pixel(1010, 248), 0x03E3,
        "B10: x=1022 na borda direita, pixel(1010,248) interior pintado");
}

#[rustfmt::skip]
#[test]
fn polygon_com_distancia_entre_vertices_excedendo_limite_nao_renderiza() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x2000_F818;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x000A_000A);
    gpu.write32(0, 0x000B_040A);
    gpu.write32(0, 0x001E_000A);

    assert_eq!(gpu.vram_pixel(10, 10), 0,
        "B11: distancia horizontal >1023 -> nao renderiza");
}

#[rustfmt::skip]
#[test]
fn polygon_texturizado_consome_palavras_de_uv_e_mantem_fifo_alinhado() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x2400_FFFF;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x0005_0005);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0005_000A);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x000A_0005);
    gpu.write32(0, 0x0000_0000);

    assert_eq!((gpu.read32(4) >> 26) & 1, 1,
        "GP0(24h) com 3 vertices+UV: poligono concluido, bit26=1");

    assert_eq!(gpu.vram_pixel(6, 6), 0,
        "triangulo texturizado sem CLUT/texel: pixel nao preenchido (4bpp sem dados)");
    assert_eq!(gpu.vram_pixel(0, 0), 0,
        "nenhum pixel em (0,0) — UV 0x0000 nao virou vertice");

    gpu.write32(0, 0x0200_00F8);
    gpu.write32(0, (0x000F << 16) | 0x000F);
    gpu.write32(0, (0x0004 << 16) | 0x0004);
    assert_ne!(gpu.vram_pixel(5, 16), 0,
        "fill GP0(02h) executado apos poligono: FIFO alinhado");
}

#[rustfmt::skip]
#[test]
fn gouraud_texturizado_consome_9_palavras_e_fifo_alinhado() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x340F_FFFF;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x0005_0005);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x00_00_F8_00);
    gpu.write32(0, 0x0005_000A);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x00_F8_00_00);
    gpu.write32(0, 0x000A_0005);
    gpu.write32(0, 0x0000_0000);

    assert_eq!((gpu.read32(4) >> 26) & 1, 1,
        "GP0(34h) gouraud texturizado 3 vertices: 9 palavras (1+3+3+2), bit26=1");
    assert_eq!(gpu.vram_pixel(6, 6), 0,
        "triangulo texturizado sem CLUT/texel: pixel nao preenchido (4bpp sem dados)");

    gpu.write32(0, 0x0200_00F8);
    gpu.write32(0, (0x000F << 16) | 0x000F);
    gpu.write32(0, (0x0004 << 16) | 0x0004);
    assert_ne!(gpu.vram_pixel(5, 16), 0,
        "fill GP0(02h) executado apos gouraud texturizado: FIFO alinhado");
}

#[rustfmt::skip]
#[test]
fn drawing_area_y2_inclusivo_triangulo_desenha_ate_linha_limite() {
    let mut gpu = Gpu::new();

    gpu.write32(0, 0xE400_2BFF);

    let cmd: u32 = 0x2000_F818;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x000A_0000);
    gpu.write32(0, 0x000F_000F);
    gpu.write32(0, 0x0000_000F);

    assert_ne!(gpu.vram_pixel(8, 10), 0,
        "D1-Y2: pixel na linha y=10 (Y2) deve ser desenhado, obtido 0x{:04X}",
        gpu.vram_pixel(8, 10));
    assert_eq!(gpu.vram_pixel(8, 11), 0,
        "D1-Y2: pixel na linha y=11 (fora do drawing area) nao deve ser desenhado");
}

#[rustfmt::skip]
#[test]
fn drawing_area_y1_inclusivo_nada_acima_de_y1() {
    let mut gpu = Gpu::new();

    gpu.write32(0, 0xE300_1400);

    let cmd: u32 = 0x2000_F818;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x000A_0000);
    gpu.write32(0, 0x0000_000A);
    gpu.write32(0, 0x0000_0000);

    assert_eq!(gpu.vram_pixel(2, 4), 0,
        "D1-Y1: pixel na linha y=4 (acima de Y1=5) nao deve ser desenhado");
    assert_ne!(gpu.vram_pixel(2, 5), 0,
        "D1-Y1: pixel na linha y=5 (Y1) deve ser desenhado, obtido 0x{:04X}",
        gpu.vram_pixel(2, 5));
}

#[rustfmt::skip]
#[test]
fn drawing_area_y_mascara_9_bits_trunca_em_511() {
    let mut gpu = Gpu::new();

    gpu.write32(0, 0xE40F_FFFF);

    let cmd: u32 = 0x2000_F818;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x000A_0000);
    gpu.write32(0, 0x0000_000A);
    gpu.write32(0, 0x0000_0000);

    assert_ne!(gpu.vram_pixel(2, 3), 0,
        "D2-Y: triangulo visivel com Y2=511 (mascara 9 bits sobre 0x3FF), obtido 0x{:04X}",
        gpu.vram_pixel(2, 3));
}

#[rustfmt::skip]
#[test]
fn drawing_offset_desloca_triangulo() {
    let mut gpu = Gpu::new();

    gpu.write32(0, 0xE503_2064);

    let cmd: u32 = 0x2000_F818;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x000A_000A);
    gpu.write32(0, 0x000A_0014);
    gpu.write32(0, 0x0014_000A);

    assert_ne!(gpu.vram_pixel(110, 110), 0,
        "E5: triangulo (10,10) deslocado para (110,110), obtido 0x{:04X}",
        gpu.vram_pixel(110, 110));
    assert_eq!(gpu.vram_pixel(10, 10), 0,
        "E5: pixel na origem absoluta (10,10) nao deve ser pintado");
    let middle = gpu.vram_pixel(112, 112);
    assert_ne!(middle, 0,
        "E5: pixel interior deslocado (112,112) deve ser pintado, obtido 0x{:04X}",
        middle);
}

#[rustfmt::skip]
#[test]
fn drawing_area_y1_mascara_para_9_bits() {
    let mut gpu = Gpu::new();

    gpu.write32(0, 0xE308_0000);

    let cmd: u32 = 0x2000_F818;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x000A_0000);
    gpu.write32(0, 0x000F_000F);
    gpu.write32(0, 0x0000_000F);

    assert_ne!(gpu.vram_pixel(8, 10), 0,
        "D2-Y1: Y1=0x200 mascarado por 9 bits vira 0, o triangulo desenha; \
         com 10 bits viraria 512 e nada seria desenhado. obtido 0x{:04X}",
        gpu.vram_pixel(8, 10));
}
