use psx_core::gpu::Gpu;

#[rustfmt::skip]
#[test]
fn linha_horizontal_inclui_ponta_inferior_direita_a1() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x40FF_FFFF;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x000A_000A);
    gpu.write32(0, 0x000A_0014);

    for x in 10..=20 {
        assert_eq!(gpu.vram_pixel(x, 10), 0x7FFF,
            "A1: pixel({},10) deve ser pintado (regra inclusiva da ponta, L361-362)", x);
    }
    assert_eq!(gpu.vram_pixel(21, 10), 0,
        "A1: pixel(21,10) nao deve ser pintado (ponta inclusive so vai ate 20)");
    assert_eq!(gpu.vram_pixel(9, 10), 0,
        "A1: pixel(9,10) antes do inicio da linha");
    assert_eq!(gpu.vram_pixel(10, 9), 0,
        "A1: pixel(10,9) acima da linha");
    assert_eq!(gpu.vram_pixel(10, 11), 0,
        "A1: pixel(10,11) abaixo da linha");
}

#[rustfmt::skip]
#[test]
fn linha_diagonal_tracado_esperado_a2() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x40FF_FFFF;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0003_0005);

    let esperados: &[(u16, u16)] = &[
        (0, 0), (1, 1), (2, 1), (3, 2), (4, 2), (5, 3),
    ];
    for &(x, y) in esperados {
        assert_eq!(gpu.vram_pixel(x, y), 0x7FFF,
            "A2: pixel({},{}) deve ser pintado no traçado Bresenham de (0,0)-(5,3)", x, y);
    }
    assert_eq!(gpu.vram_pixel(2, 2), 0,
        "A2: pixel(2,2) nao faz parte do traçado");
    assert_eq!(gpu.vram_pixel(3, 1), 0,
        "A2: pixel(3,1) nao faz parte do traçado");
}

#[rustfmt::skip]
#[test]
fn linha_diagonal_steep_tracado_esperado_a2b() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x40FF_FFFF;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0005_0003);

    let esperados: &[(u16, u16)] = &[
        (0, 0), (1, 1), (1, 2), (2, 3), (2, 4), (3, 5),
    ];
    for &(x, y) in esperados {
        assert_eq!(gpu.vram_pixel(x, y), 0x7FFF,
            "A2b: pixel({},{}) deve ser pintado no traçado de (0,0)-(3,5)", x, y);
    }
    assert_eq!(gpu.vram_pixel(2, 2), 0,
        "A2b: pixel(2,2) nao faz parte do traçado");
    assert_eq!(gpu.vram_pixel(0, 2), 0,
        "A2b: pixel(0,2) nao faz parte do traçado");
}

#[rustfmt::skip]
#[test]
fn linha_vertical_inclui_ponta_inferior_a2c() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x40FF_FFFF;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x0005_0000);
    gpu.write32(0, 0x000A_0000);

    for y in 5..=10 {
        assert_eq!(gpu.vram_pixel(0, y), 0x7FFF,
            "A2c: pixel(0,{}) deve ser pintado (vertical inclusive na ponta inferior)", y);
    }
    assert_eq!(gpu.vram_pixel(0, 4), 0,
        "A2c: pixel(0,4) antes do inicio");
    assert_eq!(gpu.vram_pixel(0, 11), 0,
        "A2c: pixel(0,11) apos a ponta");
    assert_eq!(gpu.vram_pixel(1, 6), 0,
        "A2c: pixel(1,6) fora da linha vertical");
}

#[rustfmt::skip]
#[test]
fn linha_gouraud_cor_no_meio_derivada_da_interpolacao_a3() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x5000_00FF;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x00FF_0000);
    gpu.write32(0, 0x0000_000A);

    assert_eq!(gpu.vram_pixel(0, 0), 0x001F,
        "A3: pixel(0,0) inicio = vermelho puro (R=1F, G=0, B=0)");
    assert_eq!(gpu.vram_pixel(10, 0), 0x7C00,
        "A3: pixel(10,0) fim = azul puro (R=0, G=0, B=1F)");
    let mid = gpu.vram_pixel(5, 0);
    assert_eq!(mid, 0x3C10,
        "A3: pixel(5,0) meio = interpolacao (R=0x10, G=0, B=0x0F), obtido 0x{:04X}", mid);
}

#[rustfmt::skip]
#[test]
fn vertices_coincidentes_1x1_pixel_cor_primeiro_vertice_a4() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x4000_00FF;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x0005_0005);
    gpu.write32(0, 0x0005_0005);

    assert_eq!(gpu.vram_pixel(5, 5), 0x001F,
        "A4: pixel(5,5) deve ser pintado com cor do primeiro vertice (vermelho)");
    assert_eq!(gpu.vram_pixel(5, 6), 0,
        "A4: pixel(5,6) adjacente nao pintado");
    assert_eq!(gpu.vram_pixel(6, 5), 0,
        "A4: pixel(6,5) adjacente nao pintado");
    assert_eq!(gpu.vram_pixel(4, 5), 0,
        "A4: pixel(4,5) adjacente nao pintado");
    assert_eq!(gpu.vram_pixel(5, 4), 0,
        "A4: pixel(5,4) adjacente nao pintado");
}

#[rustfmt::skip]
#[test]
fn vertices_coincidentes_gouraud_cor_do_primeiro_vertice_a4b() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x5000_00FF;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x0005_0005);
    gpu.write32(0, 0x00FF_0000);
    gpu.write32(0, 0x0005_0005);

    assert_eq!(gpu.vram_pixel(5, 5), 0x001F,
        "A4b: pixel(5,5) deve ter cor do PRIMEIRO vertice (vermelho), nao do segundo (azul)");
}

#[rustfmt::skip]
#[test]
fn polyline_flat_4_vertices_terminador_fifo_alinhado_a5() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x48FF_FFFF;
    gpu.write32(0, cmd);
    let vertices: [(u16, u16); 4] = [(0, 0), (10, 0), (10, 10), (0, 10)];
    for &(x, y) in &vertices {
        gpu.write32(0, ((y as u32) << 16) | (x as u32));
    }
    gpu.write32(0, 0x5000_5000);

    for x in 1..10 {
        assert_eq!(gpu.vram_pixel(x, 0), 0x7FFF,
            "A5: pixel({},0) segmento 1 horizontal superior", x);
    }
    for y in 1..10 {
        assert_eq!(gpu.vram_pixel(10, y), 0x7FFF,
            "A5: pixel(10,{}) segmento 2 vertical direito", y);
    }
    for x in 1..10 {
        assert_eq!(gpu.vram_pixel(x, 10), 0x7FFF,
            "A5: pixel({},10) segmento 3 horizontal inferior", x);
    }

    let fill_cmd: u32 = 0x0200_00FF;
    let fill_pos: u32 = 0x0014_0014;
    let fill_siz: u32 = 0x0002_0002;
    gpu.write32(0, fill_cmd);
    gpu.write32(0, fill_pos);
    gpu.write32(0, fill_siz);

    assert_eq!(gpu.vram_pixel(21, 20), 0x001F,
        "A5: pixel(21,20) do fill prova que o FIFO ficou alinhado apos o terminador");
    assert_eq!(gpu.vram_pixel(20, 20), 0x001F,
        "A5: pixel(20,20) do fill");
}

#[rustfmt::skip]
#[test]
fn polyline_gouraud_terminador_na_palavra_de_cor_a6() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x5800_00FF;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0000_FF00);
    gpu.write32(0, 0x0000_000A);

    gpu.write32(0, 0x5000_5000);

    for x in 1..10 {
        let pixel = gpu.vram_pixel(x, 0);
        assert_ne!(pixel, 0,
            "A6: pixel({},0) deve ser pintado (polyline gouraud com terminador na cor)", x);
    }
    assert_eq!(gpu.vram_pixel(0, 0), 0x001F,
        "A6: pixel(0,0) inicio vermelho");
    assert_eq!(gpu.vram_pixel(10, 0), 0x03E0,
        "A6: pixel(10,0) fim verde");

    let fill_cmd: u32 = 0x0200_00FF;
    let fill_pos: u32 = 0x0014_0014;
    let fill_siz: u32 = 0x0002_0002;
    gpu.write32(0, fill_cmd);
    gpu.write32(0, fill_pos);
    gpu.write32(0, fill_siz);

    assert_eq!(gpu.vram_pixel(20, 20), 0x001F,
        "A6: fill pos-terminador prova FIFO alinhado");
}

#[rustfmt::skip]
#[test]
fn polyline_flat_vertice_com_bits_28_a_31_iguais_a_5_nao_e_terminador() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x48FF_FFFF;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0000_000A);
    gpu.write32(0, 0x5005_000A);
    gpu.write32(0, 0x5000_5000);

    for x in 1..10 {
        assert_eq!(gpu.vram_pixel(x, 0), 0x7FFF,
            "m7: pixel({},0) segmento horizontal deve ser pintado", x);
    }
    for y in 1..5 {
        assert_eq!(gpu.vram_pixel(10, y), 0x7FFF,
            "m7: pixel(10,{}) segmento vertical deve ser pintado", y);
    }
}

#[rustfmt::skip]
#[test]
fn retangulo_1x1_um_pixel_a7() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x6800_00FF;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x0005_0005);

    assert_eq!(gpu.vram_pixel(5, 5), 0x001F,
        "A7: pixel(5,5) pintado (retangulo 1x1)");
    assert_eq!(gpu.vram_pixel(5, 6), 0,
        "A7: pixel(5,6) adjacente nao pintado");
    assert_eq!(gpu.vram_pixel(6, 5), 0,
        "A7: pixel(6,5) adjacente nao pintado");
}

#[rustfmt::skip]
#[test]
fn retangulo_8x8_cantos_certos_a8() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x7000_00FF;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x000A_000A);

    assert_eq!(gpu.vram_pixel(10, 10), 0x001F,
        "A8: pixel(10,10) canto superior-esquerdo do retangulo 8x8");
    assert_eq!(gpu.vram_pixel(17, 17), 0x001F,
        "A8: pixel(17,17) canto inferior-direito do retangulo 8x8");
    assert_eq!(gpu.vram_pixel(18, 17), 0,
        "A8: pixel(18,17) fora pela direita");
    assert_eq!(gpu.vram_pixel(17, 18), 0,
        "A8: pixel(17,18) fora por baixo");
    assert_eq!(gpu.vram_pixel(9, 10), 0,
        "A8: pixel(9,10) fora pela esquerda");
    assert_eq!(gpu.vram_pixel(10, 9), 0,
        "A8: pixel(10,9) fora por cima");
}

#[rustfmt::skip]
#[test]
fn retangulo_16x16_cantos_certos_a8b() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x7800_FF00;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x0000_0000);

    assert_eq!(gpu.vram_pixel(0, 0), 0x03E0,
        "A8b: pixel(0,0) canto superior-esquerdo 16x16 verde");
    assert_eq!(gpu.vram_pixel(15, 15), 0x03E0,
        "A8b: pixel(15,15) canto inferior-direito 16x16 verde");
    assert_eq!(gpu.vram_pixel(16, 15), 0,
        "A8b: pixel(16,15) fora pela direita");
    assert_eq!(gpu.vram_pixel(15, 16), 0,
        "A8b: pixel(15,16) fora por baixo");
}

#[rustfmt::skip]
#[test]
fn retangulo_variavel_3x2_seis_pixels_comando_seguinte_executa_a9() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x6000_00FF;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x000A_000A);
    let dims: u32 = (2u32 << 16) | 3u32;
    gpu.write32(0, dims);

    for y in 10..=11 {
        for x in 10..=12 {
            assert_eq!(gpu.vram_pixel(x, y), 0x001F,
                "A9: pixel({},{}) deve ser pintado (retangulo 3x2)", x, y);
        }
    }
    assert_eq!(gpu.vram_pixel(13, 10), 0,
        "A9: pixel(13,10) fora pela direita");
    assert_eq!(gpu.vram_pixel(10, 12), 0,
        "A9: pixel(10,12) fora por baixo");

    let fill_cmd: u32 = 0x0200_00FF;
    let fill_pos: u32 = 0x0014_0014;
    let fill_siz: u32 = 0x0002_0002;
    gpu.write32(0, fill_cmd);
    gpu.write32(0, fill_pos);
    gpu.write32(0, fill_siz);

    assert_eq!(gpu.vram_pixel(20, 20), 0x001F,
        "A9: fill pos-retangulo variavel prova FIFO alinhado");
}

#[rustfmt::skip]
#[test]
fn retangulo_variavel_large_1023x511_a9b() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x60FF_FFFF;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x0000_0000);
    let dims: u32 = (511u32 << 16) | 1023u32;
    gpu.write32(0, dims);

    assert_eq!(gpu.vram_pixel(0, 0), 0x7FFF,
        "A9b: pixel(0,0) pintado (canto superior-esq)");
    assert_eq!(gpu.vram_pixel(1022, 0), 0x7FFF,
        "A9b: pixel(1022,0) pintado (topo direito)");
    assert_eq!(gpu.vram_pixel(1022, 510), 0x7FFF,
        "A9b: pixel(1022,510) pintado (canto inferior-dir)");

    let total: u32 = (0..1023u16).flat_map(|x| (0..511u16).map(move |y| (x, y)))
        .filter(|&(x, y)| gpu.vram_pixel(x, y) != 0)
        .count() as u32;
    assert_eq!(total, 1023 * 511,
        "A9b: todos os {} pixels de 1023x511 devem ser pintados, obtidos {}", 1023*511, total);
}

#[rustfmt::skip]
#[test]
fn retangulo_texturizado_8x8_uv_consumido_comando_seguinte_executa_a10() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x7400_0000;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x000A_000A);
    let uv: u32 = 0x0000_0000;
    gpu.write32(0, uv);

    let fill_cmd: u32 = 0x0200_00FF;
    let fill_pos: u32 = 0x0014_0014;
    let fill_siz: u32 = 0x0002_0002;
    gpu.write32(0, fill_cmd);
    gpu.write32(0, fill_pos);
    gpu.write32(0, fill_siz);

    assert_eq!(gpu.vram_pixel(20, 20), 0x001F,
        "A10: fill pos-retangulo texturizado prova que UV foi consumido e FIFO alinhado");
}

#[rustfmt::skip]
#[test]
fn retangulo_texturizado_variavel_uv_e_dims_consumidos_a10b() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x6400_0000;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x000A_000A);
    let uv: u32 = 0x0000_0000;
    gpu.write32(0, uv);
    let dims: u32 = (2u32 << 16) | 3u32;
    gpu.write32(0, dims);

    let fill_cmd: u32 = 0x0200_00FF;
    let fill_pos: u32 = 0x0014_0014;
    let fill_siz: u32 = 0x0002_0002;
    gpu.write32(0, fill_cmd);
    gpu.write32(0, fill_pos);
    gpu.write32(0, fill_siz);

    assert_eq!(gpu.vram_pixel(20, 20), 0x001F,
        "A10b: fill pos-retangulo texturizado variavel prova UV e dims consumidos");
}
