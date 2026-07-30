use psx_core::gpu::Gpu;

fn escreve_vram_halfword(gpu: &mut Gpu, x: u16, y: u16, val: u16) {
    gpu.write32(0, 0xA0u32 << 24);
    gpu.write32(0, (y as u32) << 16 | (x as u32));
    gpu.write32(0, 0x0001_0001);
    gpu.write32(0, val as u32);
}

fn prepara_triangulo_texturizado(gpu: &mut Gpu, cmd: u32, vertices_uvs: &[((i16, i16), u8, u8)]) {
    let n = vertices_uvs.len();
    gpu.write32(0, cmd);
    for (idx, &((sx, sy), u, v)) in vertices_uvs.iter().enumerate() {
        let coord_word: u32 = ((sy as u16 as u32) << 16) | (sx as u16 as u32);
        gpu.write32(0, coord_word);
        let mut uv_word: u32 = ((v as u32) << 8) | (u as u32);
        if idx == 0 && n >= 2 {
            uv_word |= 0x0080_0000;
        } else if idx == 1 && n >= 2 {
            let stat = gpu.stat();
            let texpage: u32 = (stat & 0x3FF) | ((stat >> 15) & 1) << 11;
            uv_word |= (texpage & 0xFF_FFFF) << 16;
        }
        gpu.write32(0, uv_word);
    }
}

fn stat_com_e1h(gpu: &mut Gpu, param: u32) -> u32 {
    gpu.write32(0, (0xE1 << 24) | (param & 0x3FFF));
    let gp1_addr: u32 = 4;
    gpu.read32(gp1_addr)
}

#[rustfmt::skip]
#[test]
fn a1_gp0_e1h_todos_os_campos_setados_refletem_no_gpustat() {
    let mut gpu = Gpu::new();

    gpu.write32(4, (0x09 << 24) | 0x01);
    let stat = stat_com_e1h(&mut gpu, 0x3FFF);

    assert_eq!(
        stat & 0xF, 0xF,
        "A1: GPUSTAT.0-3 (TexPage X Base) = F"
    );
    assert_eq!(
        (stat >> 5) & 3, 3,
        "A1: GPUSTAT.5-6 (Semi-transparency) = 3"
    );
    assert_eq!(
        (stat >> 7) & 3, 3,
        "A1: GPUSTAT.7-8 (TexPage Colors) = 3"
    );
    assert_eq!(
        (stat >> 9) & 1, 1,
        "A1: GPUSTAT.9 (Dither) = 1"
    );
    assert_eq!(
        (stat >> 10) & 1, 1,
        "A1: GPUSTAT.10 (Draw to display) = 1"
    );
    assert_eq!(
        (stat >> 15) & 1, 1,
        "A1: GPUSTAT.15 (TexPage Y Base 2, do bit 11 do E1h) = 1"
    );
}

#[rustfmt::skip]
#[test]
fn a2_texpage_colors_3_se_comporta_como_15bpp() {
    let mut gpu = Gpu::new();

    escreve_vram_halfword(&mut gpu, 0, 0, 0x1234);
    escreve_vram_halfword(&mut gpu, 0, 1, 0x5678);

    stat_com_e1h(&mut gpu, 3 << 7);

    let cmd: u32 = 0x2400_0000;
    let verts = [
        ((10_i16, 10_i16), 0_u8, 0_u8),
        ((11_i16, 10_i16), 0_u8, 0_u8),
        ((10_i16, 11_i16), 0_u8, 0_u8),
    ];
    prepara_triangulo_texturizado(&mut gpu, cmd, &verts);

    assert_eq!(
        gpu.vram_pixel(10, 10), 0x1234,
        "A2: texpage colors=3 (reserved) amostra como 15bpp, obtido 0x{:04X}",
        gpu.vram_pixel(10, 10)
    );
}

#[rustfmt::skip]
#[test]
fn a3_texpage_attribute_nao_muda_bits_9_10_12_13() {
    let mut gpu = Gpu::new();

    stat_com_e1h(&mut gpu, (1 << 9) | (1 << 10) | 2 << 7 | (1 << 11));
    let stat_antes = gpu.read32(4);
    assert_eq!(
        (stat_antes >> 9) & 1, 1,
        "A3: dither (bit9) = 1 antes do poligono texturizado"
    );
    assert_eq!(
        (stat_antes >> 10) & 1, 1,
        "A3: draw to display (bit10) = 1 antes do poligono texturizado"
    );

    escreve_vram_halfword(&mut gpu, 0, 0, 0x1111);

    let cmd: u32 = 0x2400_0000;
    gpu.write32(0, cmd);
    gpu.write32(0, (10_u32 << 16) | 10_u32);
    let texpage_attr: u32 = 2 << 7;
    gpu.write32(0, texpage_attr << 16);
    gpu.write32(0, (12_u32 << 16) | 12_u32);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, (12_u32 << 16) | 10_u32);
    gpu.write32(0, 0x0000_0000);

    let stat_depois = gpu.read32(4);
    assert_eq!(
        (stat_depois >> 9) & 1, 1,
        "A3: dither (bit9) = 1 apos poligono texturizado (texpage attr nao muda bits 9-10)"
    );
    assert_eq!(
        (stat_depois >> 10) & 1, 1,
        "A3: draw to display (bit10) = 1 apos poligono texturizado (texpage attr nao muda bits 9-10)"
    );
}

#[rustfmt::skip]
#[test]
fn a4_poligono_texturizado_15bpp_confere_texel_exato() {
    let mut gpu = Gpu::new();

    escreve_vram_halfword(&mut gpu, 0, 0, 0x1234);
    escreve_vram_halfword(&mut gpu, 8, 0, 0x5678);
    escreve_vram_halfword(&mut gpu, 0, 8, 0x9ABC);
    escreve_vram_halfword(&mut gpu, 4, 4, 0xDEF0);

    stat_com_e1h(&mut gpu, 2 << 7);

    let cmd: u32 = 0x2400_0000;
    let verts = [
        ((10_i16, 10_i16), 0_u8, 0_u8),
        ((26_i16, 10_i16), 16_u8, 0_u8),
        ((10_i16, 26_i16), 0_u8, 16_u8),
    ];
    prepara_triangulo_texturizado(&mut gpu, cmd, &verts);

    assert_eq!(
        gpu.vram_pixel(10, 10), 0x1234,
        "A4: pixel(10,10) -> texel(0,0)=0x1234, obtido 0x{:04X}",
        gpu.vram_pixel(10, 10)
    );
    assert_eq!(
        gpu.vram_pixel(18, 10), 0x5678,
        "A4: pixel(18,10) -> texel(8,0)=0x5678, obtido 0x{:04X}",
        gpu.vram_pixel(18, 10)
    );
    assert_eq!(
        gpu.vram_pixel(10, 18), 0x9ABC,
        "A4: pixel(10,18) -> texel(0,8)=0x9ABC, obtido 0x{:04X}",
        gpu.vram_pixel(10, 18)
    );
    assert_eq!(
        gpu.vram_pixel(14, 14), 0xDEF0,
        "A4: pixel(14,14) -> texel(4,4)=0xDEF0, obtido 0x{:04X}",
        gpu.vram_pixel(14, 14)
    );
}

#[rustfmt::skip]
#[test]
fn a5_texel_0000h_nao_e_desenhado() {
    let mut gpu = Gpu::new();

    let bg_color: u32 = 0x0010_2008;
    gpu.write32(0, (0x02u32 << 24) | bg_color);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0050_0050);

    escreve_vram_halfword(&mut gpu, 0, 0, 0x0000);
    escreve_vram_halfword(&mut gpu, 1, 0, 0x7FFF);
    escreve_vram_halfword(&mut gpu, 2, 0, 0xFFFF);

    stat_com_e1h(&mut gpu, 2 << 7);

    let cmd: u32 = 0x2400_0000;
    let verts = [
        ((10_i16, 10_i16), 0_u8, 0_u8),
        ((14_i16, 10_i16), 4_u8, 0_u8),
        ((10_i16, 14_i16), 0_u8, 4_u8),
    ];
    prepara_triangulo_texturizado(&mut gpu, cmd, &verts);

    let bg_expected: u16 = ((bg_color >> 3) & 0x1F) as u16
        | (((bg_color >> 11) & 0x1F) as u16) << 5
        | (((bg_color >> 19) & 0x1F) as u16) << 10;

    assert_eq!(
        gpu.vram_pixel(10, 10), bg_expected,
        "A5: texel 0x0000 nao desenha; fundo sobrevive em (10,10), obtido 0x{:04X}",
        gpu.vram_pixel(10, 10)
    );
    assert_eq!(
        gpu.vram_pixel(11, 10), 0x7FFF,
        "A5: texel 0x7FFF desenha opaco em (11,10), obtido 0x{:04X}",
        gpu.vram_pixel(11, 10)
    );
    assert_eq!(
        gpu.vram_pixel(12, 10), 0xFFFF,
        "A5: texel 0xFFFF (bit15=1) desenha opaco em (12,10), obtido 0x{:04X}",
        gpu.vram_pixel(12, 10)
    );
}

#[rustfmt::skip]
#[test]
fn a6_texpage_base_diferente_de_zero() {
    let mut gpu = Gpu::new();

    escreve_vram_halfword(&mut gpu, 64, 0, 0xAAAA);
    escreve_vram_halfword(&mut gpu, 64, 256, 0xBBBB);

    stat_com_e1h(
        &mut gpu,
        (1 << 0) | (1 << 4) | 2 << 7,
    );

    let cmd: u32 = 0x2400_0000;
    let verts = [
        ((10_i16, 10_i16), 0_u8, 0_u8),
        ((11_i16, 10_i16), 0_u8, 0_u8),
        ((10_i16, 11_i16), 0_u8, 0_u8),
    ];
    prepara_triangulo_texturizado(&mut gpu, cmd, &verts);

    assert_eq!(
        gpu.vram_pixel(10, 10), 0xBBBB,
        "A6: page X=64, Y=256, texel(0,0) -> VRAM[256*1024+64]=0xBBBB, obtido 0x{:04X}",
        gpu.vram_pixel(10, 10)
    );

    let verts2 = [
        ((20_i16, 20_i16), 0_u8, 0_u8),
        ((21_i16, 20_i16), 0_u8, 0_u8),
        ((20_i16, 21_i16), 0_u8, 0_u8),
    ];
    stat_com_e1h(
        &mut gpu,
        (1 << 0) | 2 << 7,
    );
    prepara_triangulo_texturizado(&mut gpu, cmd, &verts2);

    assert_eq!(
        gpu.vram_pixel(20, 20), 0xAAAA,
        "A6: page X=64, Y=0, texel(0,0) -> VRAM[0*1024+64]=0xAAAA, obtido 0x{:04X}",
        gpu.vram_pixel(20, 20)
    );
}
