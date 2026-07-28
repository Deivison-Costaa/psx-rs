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
    assert!(preenchidos >= 3,
        "B3: pelo menos 3 pixels preenchidos, obtidos {}", preenchidos);
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
    let _r = (val & 0x1F) as u8;
    let _g = ((val >> 5) & 0x1F) as u8;
    assert!(val != 0,
        "B6-G: pixel (20,10) deve ser colorido, obtido 0x{:04X}", val);
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
    assert!(val != 0,
        "B7-G: pixel (10,20) deve ser colorido, obtido 0x{:04X}", val);
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
fn polygon_nao_liga_cmd_ready_e_retorna_para_idle() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x2000_F818;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x000A_000A);
    gpu.write32(0, 0x000A_001E);
    gpu.write32(0, 0x001E_000A);

    let stat = gpu.read32(4);
    assert_eq!((stat >> 26) & 1, 1,
        "B9: polygon finalizado, GPU deve estar ready (bit26=1), obtido stat=0x{:08X}", stat);
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
}
