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

    let expected: u16 = 0x7C6F;
    assert_eq!(gpu.vram_pixel(11, 11), expected,
        "B1: pixel(11,11) interior deve ser 0x7C6F, obtido 0x{:04X}", gpu.vram_pixel(11, 11));
    assert_eq!(gpu.vram_pixel(15, 15), expected,
        "B1: pixel(15,15) interior deve ser 0x7C6F, obtido 0x{:04X}", gpu.vram_pixel(15, 15));
    assert_eq!(gpu.vram_pixel(20, 12), expected,
        "B1: pixel(20,12) interior deve ser 0x7C6F, obtido 0x{:04X}", gpu.vram_pixel(20, 12));
    assert_eq!(gpu.vram_pixel(9, 10), 0,
        "B1: pixel(9,10) fora a esquerda");
    assert_eq!(gpu.vram_pixel(10, 9), 0,
        "B1: pixel(10,9) fora acima");
    assert_eq!(gpu.vram_pixel(11, 31), 0,
        "B1: pixel(11,31) fora abaixo");
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

    assert_eq!(gpu.vram_pixel(10, 10), 0x7FFF,
        "B2: pixel(10,10) canto superior-esquerdo deve ser 0x7FFF");
    assert_eq!(gpu.vram_pixel(15, 10), 0x7FFF,
        "B2: pixel(15,10) borda superior deve ser 0x7FFF");
    assert_eq!(gpu.vram_pixel(10, 15), 0x7FFF,
        "B2: pixel(10,15) borda esquerda deve ser 0x7FFF");
}

#[rustfmt::skip]
#[test]
fn flat_triangle_exclui_borda_inferior_e_direita() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x2000_FFFF;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0000_0002);
    gpu.write32(0, 0x0002_0000);

    assert_eq!(gpu.vram_pixel(0, 0), 0x7FFF,
        "B3: pixel(0,0) canto incluido");
    assert_eq!(gpu.vram_pixel(1, 0), 0x7FFF,
        "B3: pixel(1,0) borda superior incluido");
    assert_eq!(gpu.vram_pixel(0, 1), 0x7FFF,
        "B3: pixel(0,1) borda esquerda incluido");
    assert_eq!(gpu.vram_pixel(2, 0), 0,
        "B3: pixel(2,0) borda direita excluido");
    assert_eq!(gpu.vram_pixel(0, 2), 0,
        "B3: pixel(0,2) borda inferior excluida");
    assert_eq!(gpu.vram_pixel(1, 1), 0x7FFF,
        "B3: pixel(1,1) hipotenusa — incluido");
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

    let expected: u16 = 0x7C6F;
    assert_eq!(gpu.vram_pixel(15, 15), expected,
        "B4: pixel(15,15) dentro do quad deve ser 0x7C6F, obtido 0x{:04X}",
        gpu.vram_pixel(15, 15));
    assert_eq!(gpu.vram_pixel(15, 20), expected,
        "B4: pixel(15,20) borda inferior do tri1 esta dentro do tri2, obtido 0x{:04X}",
        gpu.vram_pixel(15,20));
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
    assert_eq!(gpu.vram_pixel(30, 10), 0x7C00,
        "B5-G: vertice1 (30,10) deve ser 0x7C00, obtido 0x{:04X}", gpu.vram_pixel(30, 10));
    assert_eq!(gpu.vram_pixel(10, 30), 0x03E0,
        "B5-G: vertice2 (10,30) deve ser 0x03E0, obtido 0x{:04X}", gpu.vram_pixel(10, 30));
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
    let r = (val & 0x1F) as u8;
    let g = ((val >> 5) & 0x1F) as u8;
    let b = ((val >> 10) & 0x1F) as u8;
    assert!(r == 0x0F,
        "B6-G: R no ponto medio da aresta superior deve ser ~0x0F, obtido {}", r);
    assert!(g == 0x0F,
        "B6-G: G no ponto medio da aresta superior deve ser ~0x0F, obtido {}", g);
    assert!(b == 0x0F && gpu.vram_pixel(19, 10) != 0,
        "B6-G: pixel(19,10) tambem deve ser colorido");
}

#[rustfmt::skip]
#[test]
fn gouraud_triangle_interpola_para_ponto_medio_da_aresta_esquerda() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x3000_0000 | 0xF8;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x000A_000A);
    gpu.write32(0, 0x00_00_F8_00);
    gpu.write32(0, 0x000A_001E);
    gpu.write32(0, 0x00_00_00_F8);
    gpu.write32(0, 0x001E_000A);

    let val = gpu.vram_pixel(10, 20);
    let r = (val & 0x1F) as u8;
    let b = ((val >> 10) & 0x1F) as u8;
    assert!(r == 0x0F,
        "B7-G: R no ponto medio da aresta esquerda deve ser ~0x0F, obtido {}", r);
    assert!(b == 0x0F,
        "B7-G: B no ponto medio da aresta esquerda deve ser ~0x0F, obtido {}", b);
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

    assert_eq!(gpu.vram_pixel(10, 10), 0x001F,
        "B8-G: V0 (10,10) deve ser 0x001F, obtido 0x{:04X}", gpu.vram_pixel(10, 10));
    assert_eq!(gpu.vram_pixel(30, 10), 0x7C00,
        "B8-G: V1 (30,10) deve ser 0x7C00, obtido 0x{:04X}", gpu.vram_pixel(30, 10));
    assert_eq!(gpu.vram_pixel(30, 30), 0x03E0,
        "B8-G: V2 (30,30) deve ser 0x03E0, obtido 0x{:04X}", gpu.vram_pixel(30, 30));
    assert_eq!(gpu.vram_pixel(10, 30), 0x001F,
        "B8-G: V3 (10,30) deve ser 0x001F, obtido 0x{:04X}", gpu.vram_pixel(10, 30));
}

#[rustfmt::skip]
#[test]
fn polygon_nao_liga_cmd_ready_e_retorna_para_idle() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x2000_F818;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x000A_000A);
    gpu.write32(0, 0x000A_001E);
    gpu.write32(0, 0x001E_000A);

    let stat = gpu.read32(4);
    assert_eq!((stat >> 26) & 1, 0,
        "B9: polygon nao liga bit26 (CMD ready), obtido stat=0x{:08X}", stat);
    assert_eq!((stat >> 27) & 1, 0,
        "B9: polygon nao liga bit27, obtido stat=0x{:08X}", stat);

    let cmd2: u32 = 0x2800_FFFF;
    gpu.write32(0, cmd2);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0000_0002);
    gpu.write32(0, 0x0002_0002);
    gpu.write32(0, 0x0002_0000);
    assert_ne!(gpu.vram_pixel(1, 1), 0,
        "B9: apos polygon, GPU esta em Idle e aceita novo comando de renderizacao");
}

#[rustfmt::skip]
#[test]
fn polygon_vertices_fora_da_vram_nao_causam_panic() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x2000_F818;
    gpu.write32(0, cmd);
    gpu.write32(0, 0xFE00_FE00);
    gpu.write32(0, 0x0400_0400);
    gpu.write32(0, 0x0000_0400);

    assert_eq!(gpu.vram_pixel(0, 0), 0,
        "B10: vertice com coordenada negativa nao causa panic");
    let cmd: u32 = 0x2000_F818;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x0400_0000);
    gpu.write32(0, 0x0800_0000);
    gpu.write32(0, 0x0600_0200);

    assert_eq!(gpu.vram_pixel(1023, 511), 0,
        "B10: vertice com coordenada >1023 truncada, nao causa panic");
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
    assert_eq!(gpu.vram_pixel(0, 0), 0,
        "B11: VRAM nao foi alterada");
}
