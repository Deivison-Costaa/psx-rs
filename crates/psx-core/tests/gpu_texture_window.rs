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

fn escreve_e2h(gpu: &mut Gpu, mask_x: u32, mask_y: u32, offset_x: u32, offset_y: u32) {
    let param = (mask_x & 0x1F)
        | ((mask_y & 0x1F) << 5)
        | ((offset_x & 0x1F) << 10)
        | ((offset_y & 0x1F) << 15);
    gpu.write32(0, (0xE2 << 24) | param);
}

#[rustfmt::skip]
#[test]
fn t1_u_0_7_amostra_16_23_com_mask_31_offset_2() {
    let mut gpu = Gpu::new();

    for i in 0..8u16 {
        escreve_vram_halfword(&mut gpu, 16 + i, 0, 0x100 | i);
    }
    for i in 0..8u16 {
        escreve_vram_halfword(&mut gpu, i, 0, 0x8000 | i);
    }

    escreve_e2h(&mut gpu, 31, 0, 2, 0);

    stat_com_e1h(&mut gpu, 2 << 7);

    let cmd: u32 = 0x2400_0000;
    gpu.write32(0, cmd);
    let verts: [((i16, i16), u8, u8); 3] = [
        ((10_i16, 10_i16), 0, 0),
        ((14_i16, 10_i16), 4, 0),
        ((10_i16, 14_i16), 0, 4),
    ];
    for (idx, &((sx, sy), u, v)) in verts.iter().enumerate() {
        gpu.write32(0, ((sy as u16 as u32) << 16) | (sx as u16 as u32));
        let mut uv_word: u32 = ((v as u32) << 8) | (u as u32);
        if idx == 1 {
            let stat = gpu.stat();
            let texpage: u32 = (stat & 0x3FF) | ((stat >> 15) & 1) << 11;
            uv_word |= (texpage & 0xFF_FFFF) << 16;
        }
        gpu.write32(0, uv_word);
    }

    assert_eq!(
        gpu.vram_pixel(10, 10), 0x0100,
        "T1: pixel(10,10) U=0 → janela → U=16, cor 0x0100, obtido 0x{:04X}",
        gpu.vram_pixel(10, 10)
    );
    assert_eq!(
        gpu.vram_pixel(11, 10), 0x0101,
        "T1: pixel(11,10) U=1 → janela → U=17, cor 0x0101, obtido 0x{:04X}",
        gpu.vram_pixel(11, 10)
    );
    assert_eq!(
        gpu.vram_pixel(12, 10), 0x0102,
        "T1: pixel(12,10) U=2 → janela → U=18, cor 0x0102, obtido 0x{:04X}",
        gpu.vram_pixel(12, 10)
    );
}

#[rustfmt::skip]
#[test]
fn t2_mask_x_7_offset_x_6_janela_48_55() {
    let mut gpu = Gpu::new();

    for i in 0..8u16 {
        escreve_vram_halfword(&mut gpu, 48 + i, 0, 0x0B00 | i);
    }
    escreve_vram_halfword(&mut gpu, 16, 0, 0x9999);

    escreve_e2h(&mut gpu, 7, 0, 6, 0);

    stat_com_e1h(&mut gpu, 2 << 7);

    let cmd: u32 = 0x2400_0000;
    gpu.write32(0, cmd);
    let verts: [((i16, i16), u8, u8); 3] = [
        ((10_i16, 10_i16), 0, 0),
        ((14_i16, 10_i16), 4, 0),
        ((10_i16, 14_i16), 0, 4),
    ];
    for (idx, &((sx, sy), u, v)) in verts.iter().enumerate() {
        gpu.write32(0, ((sy as u16 as u32) << 16) | (sx as u16 as u32));
        let mut uv_word: u32 = ((v as u32) << 8) | (u as u32);
        if idx == 1 {
            let stat = gpu.stat();
            let texpage: u32 = (stat & 0x3FF) | ((stat >> 15) & 1) << 11;
            uv_word |= (texpage & 0xFF_FFFF) << 16;
        }
        gpu.write32(0, uv_word);
    }

    assert_eq!(
        gpu.vram_pixel(10, 10), 0x0B00,
        "T2: U=0 → (0&~56)|(6*8)=48, cor 0x0B00, obtido 0x{:04X}",
        gpu.vram_pixel(10, 10)
    );
    assert_eq!(
        gpu.vram_pixel(11, 10), 0x0B01,
        "T2: U=1 → 49, cor 0x0B01, obtido 0x{:04X}",
        gpu.vram_pixel(11, 10)
    );
    assert_eq!(
        gpu.vram_pixel(12, 10), 0x0B02,
        "T2: U=2 → 50, cor 0x0B02, obtido 0x{:04X}",
        gpu.vram_pixel(12, 10)
    );
    assert_eq!(
        gpu.vram_pixel(13, 10), 0x0B03,
        "T2: U=3 → 51, cor 0x0B03, obtido 0x{:04X}",
        gpu.vram_pixel(13, 10)
    );
}

#[rustfmt::skip]
#[test]
fn t3_window_reseta_com_gp1_00h() {
    let mut gpu = Gpu::new();

    escreve_vram_halfword(&mut gpu, 0, 0, 0x1111);
    escreve_vram_halfword(&mut gpu, 32, 0, 0x2222);

    escreve_e2h(&mut gpu, 31, 0, 4, 0);

    gpu.write32(0, 0x0000_0000);

    stat_com_e1h(&mut gpu, 2 << 7);

    let cmd: u32 = 0x2400_0000;
    gpu.write32(0, cmd);
    let verts: [((i16, i16), u8, u8); 3] = [
        ((10_i16, 10_i16), 0, 0),
        ((11_i16, 10_i16), 0, 0),
        ((10_i16, 11_i16), 0, 0),
    ];
    for (idx, &((sx, sy), u, v)) in verts.iter().enumerate() {
        gpu.write32(0, ((sy as u16 as u32) << 16) | (sx as u16 as u32));
        let mut uv_word: u32 = ((v as u32) << 8) | (u as u32);
        if idx == 1 {
            let stat = gpu.stat();
            let texpage: u32 = (stat & 0x3FF) | ((stat >> 15) & 1) << 11;
            uv_word |= (texpage & 0xFF_FFFF) << 16;
        }
        gpu.write32(0, uv_word);
    }

    assert_eq!(
        gpu.vram_pixel(10, 10), 0x1111,
        "T3: reset remove janela, U=0 → U=0, cor 0x1111 (nao 0x2222), obtido 0x{:04X}",
        gpu.vram_pixel(10, 10)
    );
}

#[rustfmt::skip]
#[test]
fn t4_repeticao_janela_a_cada_32_pixels_mask_3() {
    let mut gpu = Gpu::new();

    for i in 0..8u16 {
        escreve_vram_halfword(&mut gpu, 40 + i, 0, 0x0A00 | i);
    }

    escreve_e2h(&mut gpu, 3, 0, 5, 0);

    stat_com_e1h(&mut gpu, 2 << 7);

    let cmd: u32 = 0x2400_0000;
    gpu.write32(0, cmd);
    let verts: [((i16, i16), u8, u8); 3] = [
        ((10_i16, 10_i16), 8, 0),
        ((14_i16, 10_i16), 12, 0),
        ((10_i16, 14_i16), 8, 4),
    ];
    for (idx, &((sx, sy), u, v)) in verts.iter().enumerate() {
        gpu.write32(0, ((sy as u16 as u32) << 16) | (sx as u16 as u32));
        let mut uv_word: u32 = ((v as u32) << 8) | (u as u32);
        if idx == 1 {
            let stat = gpu.stat();
            let texpage: u32 = (stat & 0x3FF) | ((stat >> 15) & 1) << 11;
            uv_word |= (texpage & 0xFF_FFFF) << 16;
        }
        gpu.write32(0, uv_word);
    }

    assert_eq!(
        gpu.vram_pixel(10, 10), 0x0A00,
        "T4: U=8, Mask=3 Offset=5 → (8&~24)|(5*8)=40, cor 0x0A00, obtido 0x{:04X}",
        gpu.vram_pixel(10, 10)
    );
    assert_eq!(
        gpu.vram_pixel(11, 10), 0x0A01,
        "T4: U=9 → 41, cor 0x0A01, obtido 0x{:04X}",
        gpu.vram_pixel(11, 10)
    );
    assert_eq!(
        gpu.vram_pixel(12, 10), 0x0A02,
        "T4: U=10 → 42, cor 0x0A02, obtido 0x{:04X}",
        gpu.vram_pixel(12, 10)
    );
}

#[rustfmt::skip]
#[test]
fn t5_offset_y_independente_de_offset_x() {
    let mut gpu = Gpu::new();

    escreve_vram_halfword(&mut gpu, 0, 16, 0xABCD);

    escreve_e2h(&mut gpu, 0, 31, 0, 2);

    stat_com_e1h(&mut gpu, 2 << 7);

    let cmd: u32 = 0x2400_0000;
    gpu.write32(0, cmd);
    let verts: [((i16, i16), u8, u8); 3] = [
        ((10_i16, 10_i16), 0, 0),
        ((11_i16, 10_i16), 0, 0),
        ((10_i16, 11_i16), 0, 0),
    ];
    for (idx, &((sx, sy), u, v)) in verts.iter().enumerate() {
        gpu.write32(0, ((sy as u16 as u32) << 16) | (sx as u16 as u32));
        let mut uv_word: u32 = ((v as u32) << 8) | (u as u32);
        if idx == 1 {
            let stat = gpu.stat();
            let texpage: u32 = (stat & 0x3FF) | ((stat >> 15) & 1) << 11;
            uv_word |= (texpage & 0xFF_FFFF) << 16;
        }
        gpu.write32(0, uv_word);
    }

    assert_eq!(
        gpu.vram_pixel(10, 10), 0xABCD,
        "T5: V=0, Mask Y=31 Offset Y=2 → V janela=16, page_y=0, linha 16, cor 0xABCD, obtido 0x{:04X}",
        gpu.vram_pixel(10, 10)
    );
}
