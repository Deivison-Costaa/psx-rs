use psx_core::gpu::Gpu;

fn escreve_vram_halfword(gpu: &mut Gpu, x: u16, y: u16, val: u16) {
    gpu.write32(0, 0xA0u32 << 24);
    gpu.write32(0, (y as u32) << 16 | (x as u32));
    gpu.write32(0, 0x0001_0001);
    gpu.write32(0, val as u32);
}

fn stat_com_e1h(gpu: &mut Gpu, param: u32) -> u32 {
    gpu.write32(0, (0xE1 << 24) | (param & 0x3FFF));
    gpu.read32(4)
}

fn prepara_triangulo_texturizado_clut(
    gpu: &mut Gpu,
    cmd: u32,
    clut_attr: u16,
    vertices_uvs: &[((i16, i16), u8, u8)],
) {
    gpu.write32(0, cmd);
    for (idx, &((sx, sy), u, v)) in vertices_uvs.iter().enumerate() {
        let coord_word: u32 = ((sy as u16 as u32) << 16) | (sx as u16 as u32);
        gpu.write32(0, coord_word);
        let mut uv_word: u32 = ((v as u32) << 8) | (u as u32);
        if idx == 0 {
            uv_word |= (clut_attr as u32) << 16;
        } else if idx == 1 {
            let stat = gpu.stat();
            let texpage: u32 = (stat & 0x3FF) | ((stat >> 15) & 1) << 11;
            uv_word |= (texpage & 0xFF_FFFF) << 16;
        }
        gpu.write32(0, uv_word);
    }
}

#[rustfmt::skip]
#[test]
fn b1_8bpp_amostra_palette_index_e_clut() {
    let mut gpu = Gpu::new();

    escreve_vram_halfword(&mut gpu, 3, 0, 0x1234);
    escreve_vram_halfword(&mut gpu, 5, 0, 0x5678);

    let texel_hw0: u32 = 3 | (5 << 8);
    gpu.write32(0, 0xA0u32 << 24);
    gpu.write32(0, 0u32);
    gpu.write32(0, 0x0001_0001);
    gpu.write32(0, texel_hw0);

    stat_com_e1h(&mut gpu, 1 << 7);

    let cmd: u32 = 0x2500_0000;
    let verts = [
        ((10_i16, 10_i16), 0_u8, 0_u8),
        ((12_i16, 10_i16), 2_u8, 0_u8),
        ((10_i16, 12_i16), 0_u8, 2_u8),
    ];
    prepara_triangulo_texturizado_clut(&mut gpu, cmd, 0x0000, &verts);

    assert_eq!(
        gpu.vram_pixel(10, 10), 0x1234,
        "B1: pixel(10,10) → texel(0,0)=idx3 → CLUT[3]=0x1234, obtido 0x{:04X}",
        gpu.vram_pixel(10, 10)
    );
    assert_eq!(
        gpu.vram_pixel(11, 10), 0x5678,
        "B1: pixel(11,10) → texel(1,0)=idx5 → CLUT[5]=0x5678, obtido 0x{:04X}",
        gpu.vram_pixel(11, 10)
    );
}

#[rustfmt::skip]
#[test]
fn b2_4bpp_amostra_4_pixels_por_halfword() {
    let mut gpu = Gpu::new();

    escreve_vram_halfword(&mut gpu, 2, 0, 0xAAAA);
    escreve_vram_halfword(&mut gpu, 4, 0, 0xBBBB);
    escreve_vram_halfword(&mut gpu, 6, 0, 0xCCCC);
    escreve_vram_halfword(&mut gpu, 8, 0, 0xDDDD);

    let hw: u32 = 2 | (4 << 4) | (6 << 8) | (8 << 12);
    gpu.write32(0, 0xA0u32 << 24);
    gpu.write32(0, 0u32);
    gpu.write32(0, 0x0001_0001);
    gpu.write32(0, hw);

    stat_com_e1h(&mut gpu, 0);

    let cmd: u32 = 0x2500_0000;
    let verts = [
        ((10_i16, 10_i16), 0_u8, 0_u8),
        ((14_i16, 10_i16), 4_u8, 0_u8),
        ((10_i16, 14_i16), 0_u8, 4_u8),
    ];
    prepara_triangulo_texturizado_clut(&mut gpu, cmd, 0x0000, &verts);

    assert_eq!(
        gpu.vram_pixel(10, 10), 0xAAAA,
        "B2: pixel(10,10) → u=0 → nibble 0-3=2 → CLUT[2]=0xAAAA, obtido 0x{:04X}",
        gpu.vram_pixel(10, 10)
    );
    assert_eq!(
        gpu.vram_pixel(11, 10), 0xBBBB,
        "B2: pixel(11,10) → u=1 → nibble 4-7=4 → CLUT[4]=0xBBBB, obtido 0x{:04X}",
        gpu.vram_pixel(11, 10)
    );
    assert_eq!(
        gpu.vram_pixel(12, 10), 0xCCCC,
        "B2: pixel(12,10) → u=2 → nibble 8-11=6 → CLUT[6]=0xCCCC, obtido 0x{:04X}",
        gpu.vram_pixel(12, 10)
    );
    assert_eq!(
        gpu.vram_pixel(13, 10), 0xDDDD,
        "B2: pixel(13,10) → u=3 → nibble 12-15=8 → CLUT[8]=0xDDDD, obtido 0x{:04X}",
        gpu.vram_pixel(13, 10)
    );
}

#[rustfmt::skip]
#[test]
fn b3_clut_em_posicao_arbitraria() {
    let mut gpu = Gpu::new();

    escreve_vram_halfword(&mut gpu, 51, 50, 0x9ABC);
    escreve_vram_halfword(&mut gpu, 53, 50, 0xDEF0);

    let texel_hw0: u32 = 3 | (5 << 8);
    gpu.write32(0, 0xA0u32 << 24);
    gpu.write32(0, 0u32);
    gpu.write32(0, 0x0001_0001);
    gpu.write32(0, texel_hw0);

    stat_com_e1h(&mut gpu, 1 << 7);

    let clut_attr: u16 = 3 | (50 << 6);

    let cmd: u32 = 0x2500_0000;
    let verts = [
        ((10_i16, 10_i16), 0_u8, 0_u8),
        ((12_i16, 10_i16), 2_u8, 0_u8),
        ((10_i16, 12_i16), 0_u8, 2_u8),
    ];
    prepara_triangulo_texturizado_clut(&mut gpu, cmd, clut_attr, &verts);

    assert_eq!(
        gpu.vram_pixel(10, 10), 0x9ABC,
        "B3: pixel(10,10) → idx3 → CLUT@(48,50)[3]=0x9ABC, obtido 0x{:04X}",
        gpu.vram_pixel(10, 10)
    );
    assert_eq!(
        gpu.vram_pixel(11, 10), 0xDEF0,
        "B3: pixel(11,10) → idx5 → CLUT@(48,50)[5]=0xDEF0, obtido 0x{:04X}",
        gpu.vram_pixel(11, 10)
    );
}

#[rustfmt::skip]
#[test]
fn b4_clut_entry_0000h_nao_e_desenhado() {
    let mut gpu = Gpu::new();

    let bg_color: u32 = 0x0010_2008;
    gpu.write32(0, (0x02u32 << 24) | bg_color);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0050_0050);

    escreve_vram_halfword(&mut gpu, 33, 0, 0x7FFF);

    let texel_hw0: u32 = 1;
    gpu.write32(0, 0xA0u32 << 24);
    gpu.write32(0, 0u32);
    gpu.write32(0, 0x0001_0001);
    gpu.write32(0, texel_hw0);

    stat_com_e1h(&mut gpu, 1 << 7);

    let clut_attr: u16 = 2;

    let cmd: u32 = 0x2500_0000;
    let verts = [
        ((10_i16, 10_i16), 0_u8, 0_u8),
        ((12_i16, 10_i16), 2_u8, 0_u8),
        ((10_i16, 12_i16), 0_u8, 2_u8),
    ];
    prepara_triangulo_texturizado_clut(&mut gpu, cmd, clut_attr, &verts);

    let bg_expected: u16 = ((bg_color >> 3) & 0x1F) as u16
        | (((bg_color >> 11) & 0x1F) as u16) << 5
        | (((bg_color >> 19) & 0x1F) as u16) << 10;

    assert_eq!(
        gpu.vram_pixel(10, 10), 0x7FFF,
        "B4: pixel(10,10) → idx1=0x7FFF (opaco), obtido 0x{:04X}",
        gpu.vram_pixel(10, 10)
    );
    assert_eq!(
        gpu.vram_pixel(11, 10), bg_expected,
        "B4: pixel(11,10) → idx0=0x0000 (transparente), fundo sobrevive, obtido 0x{:04X}",
        gpu.vram_pixel(11, 10)
    );
}

#[rustfmt::skip]
#[test]
fn b5_8bpp_com_page_nao_zero() {
    let mut gpu = Gpu::new();

    escreve_vram_halfword(&mut gpu, 3, 0, 0x1111);

    gpu.write32(0, 0xA0u32 << 24);
    gpu.write32(0, (256u32 << 16) | 64u32);
    gpu.write32(0, 0x0001_0001);
    gpu.write32(0, 3u32);

    stat_com_e1h(
        &mut gpu,
        1 | (1 << 4) | 1 << 7,
    );

    let cmd: u32 = 0x2500_0000;
    let verts = [
        ((10_i16, 10_i16), 0_u8, 0_u8),
        ((11_i16, 10_i16), 0_u8, 0_u8),
        ((10_i16, 11_i16), 0_u8, 0_u8),
    ];
    prepara_triangulo_texturizado_clut(&mut gpu, cmd, 0x0000, &verts);

    assert_eq!(
        gpu.vram_pixel(10, 10), 0x1111,
        "B5: page X=64 Y=256, texel(0,0) 8bpp → idx3 → CLUT[3]=0x1111, obtido 0x{:04X}",
        gpu.vram_pixel(10, 10)
    );
}

#[rustfmt::skip]
#[test]
fn b6_modos_misturados_8bpp_4bpp_15bpp() {
    let mut gpu = Gpu::new();

    escreve_vram_halfword(&mut gpu, 3, 0, 0xAAAA);

    let texel_8bpp: u32 = 3;
    gpu.write32(0, 0xA0u32 << 24);
    gpu.write32(0, 0u32);
    gpu.write32(0, 0x0001_0001);
    gpu.write32(0, texel_8bpp);

    stat_com_e1h(&mut gpu, 1 << 7);
    let cmd: u32 = 0x2500_0000;
    let verts_8bpp = [
        ((10_i16, 10_i16), 0_u8, 0_u8),
        ((11_i16, 10_i16), 0_u8, 0_u8),
        ((10_i16, 11_i16), 0_u8, 0_u8),
    ];
    prepara_triangulo_texturizado_clut(&mut gpu, cmd, 0x0000, &verts_8bpp);
    assert_eq!(gpu.vram_pixel(10, 10), 0xAAAA, "B6-A: 8bpp ok");

    escreve_vram_halfword(&mut gpu, 17, 0, 0x9ABC);

    let texel_4bpp: u32 = 1;
    gpu.write32(0, 0xA0u32 << 24);
    gpu.write32(0, 256u32 << 16);
    gpu.write32(0, 0x0001_0001);
    gpu.write32(0, texel_4bpp);

    stat_com_e1h(&mut gpu, 1 << 4);
    let verts_4bpp = [
        ((20_i16, 20_i16), 0_u8, 0_u8),
        ((21_i16, 20_i16), 0_u8, 0_u8),
        ((20_i16, 21_i16), 0_u8, 0_u8),
    ];
    let clut_attr_4bpp: u16 = 1;
    prepara_triangulo_texturizado_clut(&mut gpu, cmd, clut_attr_4bpp, &verts_4bpp);
    assert_eq!(gpu.vram_pixel(20, 20), 0x9ABC, "B6-B: 4bpp ok");

    escreve_vram_halfword(&mut gpu, 0, 0, 0x5555);
    stat_com_e1h(&mut gpu, 2 << 7);
    let verts_15bpp = [
        ((30_i16, 30_i16), 0_u8, 0_u8),
        ((31_i16, 30_i16), 0_u8, 0_u8),
        ((30_i16, 31_i16), 0_u8, 0_u8),
    ];
    let cmd_15bpp: u32 = 0x2500_0000;
    gpu.write32(0, cmd_15bpp);
    for (idx, &((sx, sy), u, v)) in verts_15bpp.iter().enumerate() {
        gpu.write32(0, ((sy as u16 as u32) << 16) | (sx as u16 as u32));
        let mut uv_word: u32 = ((v as u32) << 8) | (u as u32);
        if idx == 1 {
            let stat = gpu.stat();
            let texpage: u32 = (stat & 0x3FF) | ((stat >> 15) & 1) << 11;
            uv_word |= (texpage & 0xFF_FFFF) << 16;
        }
        gpu.write32(0, uv_word);
    }
    assert_eq!(gpu.vram_pixel(30, 30), 0x5555, "B6-C: 15bpp ok");
}
