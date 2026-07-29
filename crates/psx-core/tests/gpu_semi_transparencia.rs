use psx_core::gpu::Gpu;

fn escreve_halfword(gpu: &mut Gpu, x: u16, y: u16, val: u16) {
    gpu.write32(0, 0xA0u32 << 24);
    gpu.write32(0, (y as u32) << 16 | (x as u32));
    gpu.write32(0, 0x0001_0001);
    gpu.write32(0, val as u32);
}

fn espera_idle(gpu: &mut Gpu) {
    while gpu.read32(4) & (1 << 26) == 0 {}
}

#[rustfmt::skip]
#[test]
fn t1_modo_0_media_back_e_front() {
    let mut gpu = Gpu::new();

    escreve_halfword(&mut gpu, 10, 10, 0x0010);
    escreve_halfword(&mut gpu, 0, 0, 0x8008);

    let cmd: u32 = (0x26 << 24) | 0x808080;
    espera_idle(&mut gpu);
    gpu.write32(0, cmd);

    gpu.write32(0, ((10u32) << 16) | 10u32);
    gpu.write32(0, 0x00000000);

    gpu.write32(0, ((10u32) << 16) | 14u32);
    let texpage: u32 = 2u32 << 7;
    gpu.write32(0, 0x0000_0004 | (texpage << 16));

    gpu.write32(0, ((14u32) << 16) | 10u32);
    gpu.write32(0, 0x0004_0000);

    espera_idle(&mut gpu);
    assert_eq!(
        gpu.vram_pixel(10, 10), 0x000C,
        "T1 modo 0: back R=16, front R=8, B/2+F/2 → R=(16/2)+(8/2)=8+4=12=0x000C",
    );
}

#[rustfmt::skip]
#[test]
fn t2_modo_1_aditivo_com_clamp() {
    let mut gpu = Gpu::new();

    escreve_halfword(&mut gpu, 10, 10, 0x0018);
    escreve_halfword(&mut gpu, 0, 0, 0x8018);

    let cmd: u32 = (0x26 << 24) | 0x808080;
    espera_idle(&mut gpu);
    gpu.write32(0, cmd);

    gpu.write32(0, ((10u32) << 16) | 10u32);
    gpu.write32(0, 0x00000000);

    gpu.write32(0, ((10u32) << 16) | 14u32);
    let texpage: u32 = (2u32 << 7) | (1u32 << 5);
    gpu.write32(0, 0x0000_0004 | (texpage << 16));

    gpu.write32(0, ((14u32) << 16) | 10u32);
    gpu.write32(0, 0x0004_0000);

    espera_idle(&mut gpu);
    assert_eq!(
        gpu.vram_pixel(10, 10), 0x001F,
        "T2 modo 1: back R=24, front R=24, B+F → R=min(31,24+24)=31, esperado 0x001F",
    );
}

#[rustfmt::skip]
#[test]
fn t3_modo_2_subtrativo_com_clamp_zero() {
    let mut gpu = Gpu::new();

    escreve_halfword(&mut gpu, 10, 10, 0x0008);
    escreve_halfword(&mut gpu, 0, 0, 0x8018);

    let cmd: u32 = (0x26 << 24) | 0x808080;
    espera_idle(&mut gpu);
    gpu.write32(0, cmd);

    gpu.write32(0, ((10u32) << 16) | 10u32);
    gpu.write32(0, 0x00000000);

    gpu.write32(0, ((10u32) << 16) | 14u32);
    let texpage: u32 = (2u32 << 7) | (2u32 << 5);
    gpu.write32(0, 0x0000_0004 | (texpage << 16));

    gpu.write32(0, ((14u32) << 16) | 10u32);
    gpu.write32(0, 0x0004_0000);

    espera_idle(&mut gpu);
    assert_eq!(
        gpu.vram_pixel(10, 10), 0x0000,
        "T3 modo 2: back R=8, front R=24, B-F → R=max(0,8-24)=0, esperado 0x0000",
    );
}

#[rustfmt::skip]
#[test]
fn t4_modo_3_back_mais_um_quarto_de_front() {
    let mut gpu = Gpu::new();

    escreve_halfword(&mut gpu, 10, 10, 0x0010);
    escreve_halfword(&mut gpu, 0, 0, 0x8018);

    let cmd: u32 = (0x26 << 24) | 0x808080;
    espera_idle(&mut gpu);
    gpu.write32(0, cmd);

    gpu.write32(0, ((10u32) << 16) | 10u32);
    gpu.write32(0, 0x00000000);

    gpu.write32(0, ((10u32) << 16) | 14u32);
    let texpage: u32 = (2u32 << 7) | (3u32 << 5);
    gpu.write32(0, 0x0000_0004 | (texpage << 16));

    gpu.write32(0, ((14u32) << 16) | 10u32);
    gpu.write32(0, 0x0004_0000);

    espera_idle(&mut gpu);
    assert_eq!(
        gpu.vram_pixel(10, 10), 0x0016,
        "T4 modo 3: back R=16, front R=24, B+F/4 → R=min(31,16+(24/4))=22, esperado 0x0016 (R=22)",
    );
}

#[rustfmt::skip]
#[test]
fn t5_semi_transparencia_desabilitada_bit25_zero() {
    let mut gpu = Gpu::new();

    escreve_halfword(&mut gpu, 10, 10, 0x0010);
    escreve_halfword(&mut gpu, 0, 0, 0x8008);

    let cmd: u32 = (0x24 << 24) | 0x808080;
    espera_idle(&mut gpu);
    gpu.write32(0, cmd);

    gpu.write32(0, ((10u32) << 16) | 10u32);
    gpu.write32(0, 0x00000000);

    gpu.write32(0, ((10u32) << 16) | 14u32);
    let texpage: u32 = 2u32 << 7;
    gpu.write32(0, 0x0000_0004 | (texpage << 16));

    gpu.write32(0, ((14u32) << 16) | 10u32);
    gpu.write32(0, 0x0004_0000);

    espera_idle(&mut gpu);
    assert_eq!(
        gpu.vram_pixel(10, 10), 0x8008,
        "T5: bit25=0 → sem blend, pixel escrito como texel cru 0x8008 (R=8)",
    );
}

#[rustfmt::skip]
#[test]
fn t6_pixel_nao_transparente_bit15_zero() {
    let mut gpu = Gpu::new();

    escreve_halfword(&mut gpu, 10, 10, 0x0010);

    escreve_halfword(&mut gpu, 0, 0, 0x0008);

    let cmd: u32 = (0x26 << 24) | 0x808080;
    espera_idle(&mut gpu);
    gpu.write32(0, cmd);

    gpu.write32(0, ((10u32) << 16) | 10u32);
    gpu.write32(0, 0x00000000);

    gpu.write32(0, ((10u32) << 16) | 14u32);
    let texpage: u32 = 2u32 << 7;
    gpu.write32(0, 0x0000_0004 | (texpage << 16));

    gpu.write32(0, ((14u32) << 16) | 10u32);
    gpu.write32(0, 0x0004_0000);

    espera_idle(&mut gpu);
    assert_eq!(
        gpu.vram_pixel(10, 10), 0x0008,
        "T6: bit25=1 mas texel bit15=0 → sem blend, pixel escrito cru 0x0008",
    );
}
