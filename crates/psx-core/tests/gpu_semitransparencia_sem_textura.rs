use psx_core::gpu::Gpu;

const BACK: u16 = 0x1110;
const COR24: u32 = 0x00C0_8040;
const OPACO: u16 = 0x6208;

fn escreve_halfword(gpu: &mut Gpu, x: u16, y: u16, val: u16) {
    gpu.write32(0, 0xA0u32 << 24);
    gpu.write32(0, (y as u32) << 16 | (x as u32));
    gpu.write32(0, 0x0001_0001);
    gpu.write32(0, val as u32);
}

fn espera_idle(gpu: &mut Gpu) {
    while gpu.read32(4) & (1 << 26) == 0 {}
}

fn modo_semi(gpu: &mut Gpu, modo: u32) {
    gpu.write32(0, (0xE1u32 << 24) | (modo << 5));
}

fn desenha_rect(gpu: &mut Gpu, cmd: u32, x: u16, y: u16) {
    espera_idle(gpu);
    gpu.write32(0, (cmd << 24) | COR24);
    gpu.write32(0, ((y as u32) << 16) | (x as u32));
    gpu.write32(0, 0x0001_0001);
    espera_idle(gpu);
}

fn desenha_tri(gpu: &mut Gpu, cmd: u32) {
    espera_idle(gpu);
    gpu.write32(0, (cmd << 24) | COR24);
    gpu.write32(0, (10u32 << 16) | 10u32);
    gpu.write32(0, (10u32 << 16) | 20u32);
    gpu.write32(0, (20u32 << 16) | 10u32);
    espera_idle(gpu);
}

fn cena(modo: u32) -> Gpu {
    let mut gpu = Gpu::new();
    escreve_halfword(&mut gpu, 10, 10, BACK);
    modo_semi(&mut gpu, modo);
    gpu
}

#[test]
fn t1_rect_modo_0_metade_back_mais_metade_front() {
    let mut gpu = cena(0);
    desenha_rect(&mut gpu, 0x62, 10, 10);
    assert_eq!(
        gpu.vram_pixel(10, 10),
        0x398C,
        "03-gpu.md L1516 (bit25 vale para All Render Types) + L1592 (0.5xB+0.5xF): \
         B=(16,8,4) F=(8,16,24) -> (12,12,14) = 0x398C, bit15=0 (L587)",
    );
}

#[test]
fn t2_rect_modo_1_aditivo() {
    let mut gpu = cena(1);
    desenha_rect(&mut gpu, 0x62, 10, 10);
    assert_eq!(
        gpu.vram_pixel(10, 10),
        0x7318,
        "03-gpu.md L1593 (1.0xB+1.0xF): B=(16,8,4) F=(8,16,24) -> (24,24,28) = 0x7318",
    );
}

#[test]
fn t3_rect_modo_2_subtrativo_com_clamp_zero() {
    let mut gpu = cena(2);
    desenha_rect(&mut gpu, 0x62, 10, 10);
    assert_eq!(
        gpu.vram_pixel(10, 10),
        0x0008,
        "03-gpu.md L1594 (1.0xB-1.0xF) + L1601 (clamp em 0): \
         B=(16,8,4) F=(8,16,24) -> (8,0,0) = 0x0008",
    );
}

#[test]
fn t4_rect_modo_3_back_mais_um_quarto_de_front() {
    let mut gpu = cena(3);
    desenha_rect(&mut gpu, 0x62, 10, 10);
    assert_eq!(
        gpu.vram_pixel(10, 10),
        0x2992,
        "03-gpu.md L1595 (1.0xB+0.25xF): B=(16,8,4) F=(8,16,24) -> (18,12,10) = 0x2992",
    );
}

#[test]
fn t5_controle_rect_opaco_bit25_zero_nao_mistura() {
    let mut gpu = cena(0);
    desenha_rect(&mut gpu, 0x60, 10, 10);
    assert_eq!(
        gpu.vram_pixel(10, 10),
        OPACO,
        "03-gpu.md L1516: bit25=0 desliga a semi-transparencia; a cor de front vai crua",
    );
}

#[test]
fn t6_poligono_plano_modo_0_mistura() {
    let mut gpu = cena(0);
    desenha_tri(&mut gpu, 0x22);
    assert_eq!(
        gpu.vram_pixel(10, 10),
        0x398C,
        "03-gpu.md L1516: bit25 vale tambem para poligono monocromatico sem textura",
    );
}

#[test]
fn t7_controle_poligono_plano_opaco() {
    let mut gpu = cena(0);
    desenha_tri(&mut gpu, 0x20);
    assert_eq!(
        gpu.vram_pixel(10, 10),
        OPACO,
        "03-gpu.md L1516: poligono com bit25=0 escreve a cor crua",
    );
}

#[test]
fn t8_rect_modo_1_satura_em_31() {
    let mut gpu = Gpu::new();
    escreve_halfword(&mut gpu, 10, 10, 0x7FFF);
    modo_semi(&mut gpu, 1);
    desenha_rect(&mut gpu, 0x62, 10, 10);
    assert_eq!(
        gpu.vram_pixel(10, 10),
        0x7FFF,
        "03-gpu.md L1600 (clamp em 255/31 no aditivo): B=(31,31,31) + F -> (31,31,31)",
    );
}

#[test]
fn t9_linha_monocromatica_modo_0_mistura() {
    let mut gpu = cena(0);
    espera_idle(&mut gpu);
    gpu.write32(0, (0x42u32 << 24) | COR24);
    gpu.write32(0, (10u32 << 16) | 10u32);
    gpu.write32(0, (10u32 << 16) | 20u32);
    espera_idle(&mut gpu);
    assert_eq!(
        gpu.vram_pixel(10, 10),
        0x398C,
        "03-gpu.md L1516: bit25 vale para All Render Types, linhas incluidas",
    );
}
