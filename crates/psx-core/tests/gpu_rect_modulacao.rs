use psx_core::gpu::Gpu;

fn escreve_vram_halfword(gpu: &mut Gpu, x: u16, y: u16, val: u16) {
    gpu.write32(0, 0xA0u32 << 24);
    gpu.write32(0, (y as u32) << 16 | (x as u32));
    gpu.write32(0, 0x0001_0001);
    gpu.write32(0, val as u32);
}

fn retangulo(gpu: &mut Gpu, cmd: u8, cor: u32, x: i16, y: i16) {
    gpu.write32(0, ((cmd as u32) << 24) | (cor & 0x00FF_FFFF));
    gpu.write32(0, ((y as u16 as u32) << 16) | (x as u16 as u32));
    gpu.write32(0, 0);
    gpu.write32(0, 0x0001_0001);
}

fn cena(cmd: u8, cor: u32) -> u16 {
    let mut gpu = Gpu::new();
    escreve_vram_halfword(&mut gpu, 0, 0, 0x7FFF);
    gpu.write32(0, (0xE1 << 24) | (2 << 7));
    retangulo(&mut gpu, cmd, cor, 10, 10);
    gpu.vram_pixel(10, 10)
}

#[rustfmt::skip]
#[test]
fn b1_retangulo_raw_texture_65h_ignora_a_cor_do_comando() {
    let obtido = cena(0x65, 0x0000_0000);
    assert_eq!(
        obtido, 0x7FFF,
        "B1: 03-gpu.md L381-390 — no comando de retangulo o bit24=1 e 'raw texture'. \
         Com cor 000000 o texel 0x7FFF tem de sair intacto; obtido 0x{:04X}",
        obtido
    );
}

#[rustfmt::skip]
#[test]
fn b2_retangulo_64h_com_cor_808080_e_modulacao_neutra() {
    let obtido = cena(0x64, 0x0080_8080);
    assert_eq!(
        obtido, 0x7FFF,
        "B2: 03-gpu.md L1615 — cor 0x808080 equivale a nao modular. O texel 0x7FFF tem de \
         sair intacto; obtido 0x{:04X}",
        obtido
    );
}

#[rustfmt::skip]
#[test]
fn b3_retangulo_64h_com_cor_404040_reduz_o_texel_a_metade() {
    let obtido = cena(0x64, 0x0040_4040);
    assert_eq!(
        obtido, 0x3DEF,
        "B3: 03-gpu.md L1604-1611 — modulacao e (texel*cor)/128 por canal. Com cor 0x404040 \
         o texel 0x7FFF (31,31,31) tem de virar (15,15,15)=0x3DEF; obtido 0x{:04X}. \
         O retangulo texturizado do psx-rs descartava a cor do comando: 64h se comportava \
         como 65h.",
        obtido
    );
}

#[rustfmt::skip]
#[test]
fn b4_retangulo_64h_com_cor_000000_apaga_o_texel() {
    let obtido = cena(0x64, 0x0000_0000);
    assert_eq!(
        obtido, 0x0000,
        "B4: modulacao por cor 0 zera os tres canais (o bit15 do texel 0x7FFF ja e 0); \
         obtido 0x{:04X}",
        obtido
    );
}
