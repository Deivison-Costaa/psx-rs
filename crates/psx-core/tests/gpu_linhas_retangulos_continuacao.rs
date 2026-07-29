use psx_core::gpu::Gpu;

#[rustfmt::skip]
#[test]
fn polyline_17_vertices_nao_estoura_e_renderiza_16_segmentos_d1() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x48FF_FFFF;
    gpu.write32(0, cmd);
    for i in 0..17u32 {
        gpu.write32(0, i);
    }
    gpu.write32(0, 0x5000_5000);

    for x in 1..17 {
        assert_eq!(gpu.vram_pixel(x as u16, 0), 0x7FFF,
            "D1: pixel({},0) do segmento {} deve ser pintado", x, x);
    }
}

#[rustfmt::skip]
#[test]
fn retangulo_texturizado_nao_desenha_com_cor_d2() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x7400_00FF;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x000A_000A);
    let uv: u32 = 0x0000_0000;
    gpu.write32(0, uv);

    assert_eq!(gpu.vram_pixel(10, 10), 0,
        "D2: pixel(10,10) nao deve ser pintado (texturizado ignora cor, spec L402)");
}

#[rustfmt::skip]
#[test]
fn retangulo_texturizado_variavel_nao_desenha_com_cor_d2b() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x6400_00FF;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x000A_000A);
    let uv: u32 = 0x0000_0000;
    gpu.write32(0, uv);
    let dims: u32 = (2u32 << 16) | 3u32;
    gpu.write32(0, dims);

    for y in 10..=11 {
        for x in 10..=12 {
            assert_eq!(gpu.vram_pixel(x, y), 0,
                "D2b: pixel({},{}) nao deve ser pintado (texturizado variavel)", x, y);
        }
    }
}

#[rustfmt::skip]
#[test]
fn linha_simples_dy_600_nao_renderiza_d3() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x40FF_FFFF;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0258_0000);

    assert_eq!(gpu.vram_pixel(0, 300), 0,
        "D3: pixel(0,300) nao deve ser pintado (dy=600 > 511, spec L447-451)");
}

#[rustfmt::skip]
#[test]
fn linha_simples_dy_500_renderiza_d3b() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x40FF_FFFF;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x01F4_0000);

    assert_ne!(gpu.vram_pixel(0, 250), 0,
        "D3b: pixel(0,250) deve ser pintado (dy=500 <= 511, spec L447-451)");
}

#[rustfmt::skip]
#[test]
fn linha_simples_gouraud_dy_600_nao_renderiza_d3c() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x5000_00FF;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0000_FF00);
    gpu.write32(0, 0x0258_0000);

    assert_eq!(gpu.vram_pixel(0, 300), 0,
        "D3c: pixel(0,300) linha gouraud dy=600 nao deve ser pintado");
}

#[rustfmt::skip]
#[test]
fn retangulo_variavel_width_height_ffff_termina_d4() {
    let mut gpu = Gpu::new();

    let cmd: u32 = 0x6000_00FF;
    gpu.write32(0, cmd);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0xFFFF_FFFF);

    assert_eq!(gpu.vram_pixel(1023, 511), 0x001F,
        "D4: pixel(1023,511) pintado (area visivel do retangulo FFFFxFFFF)");
    assert_eq!(gpu.vram_pixel(511, 0), 0x001F,
        "D4: pixel(511,0) pintado dentro da area de desenho");
    assert_ne!(gpu.stat() & (1 << 26), 0,
        "D4: bit 26 do stat deve estar setado (comando completou)");
}
