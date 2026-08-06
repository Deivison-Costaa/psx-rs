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

fn triangulo_com_cor(gpu: &mut Gpu, cmd_byte: u32, cor24: u32, verts: &[((i16, i16), u8, u8); 3]) {
    gpu.write32(0, (cmd_byte << 24) | (cor24 & 0x00FF_FFFF));
    for (idx, &((sx, sy), u, v)) in verts.iter().enumerate() {
        gpu.write32(0, ((sy as u32 & 0xFFFF) << 16) | (sx as u32 & 0xFFFF));
        let mut uv = ((v as u32) << 8) | (u as u32);
        if idx == 0 {
            uv |= 0x0080_0000;
        } else if idx == 1 {
            let stat = gpu.stat();
            let texpage: u32 = (stat & 0x3FF) | (((stat >> 15) & 1) << 11);
            uv |= (texpage & 0xFF_FFFF) << 16;
        }
        gpu.write32(0, uv);
    }
}

fn verts_uv() -> [((i16, i16), u8, u8); 3] {
    [
        ((10_i16, 10_i16), 0_u8, 0_u8),
        ((26_i16, 10_i16), 16_u8, 0_u8),
        ((10_i16, 26_i16), 0_u8, 16_u8),
    ]
}

#[test]
fn modulacao_escurece_pelo_canal_da_cor_do_vertice() {
    let mut gpu = Gpu::new();
    escreve_vram_halfword(&mut gpu, 0, 0, 0x2310); // r=16,g=24,b=8
    stat_com_e1h(&mut gpu, 2 << 7);

    triangulo_com_cor(&mut gpu, 0x24, 0x0080_4020, &verts_uv()); // r=32,g=64,b=128 (8bit)

    assert_eq!(
        gpu.vram_pixel(10, 10),
        0x2184,
        "spec § Modulation (03-gpu.md L1604): finalChannel = texel*cor/128 por canal — \
         r=16*4/16=4, g=24*8/16=12, b=8*16/16=8, obtido 0x{:04X}",
        gpu.vram_pixel(10, 10)
    );
}

#[test]
fn cor_neutra_808080_nao_muda_o_texel() {
    let mut gpu = Gpu::new();
    escreve_vram_halfword(&mut gpu, 0, 0, 0x2310);
    stat_com_e1h(&mut gpu, 2 << 7);

    triangulo_com_cor(&mut gpu, 0x24, 0x0080_8080, &verts_uv());

    assert_eq!(
        gpu.vram_pixel(10, 10),
        0x2310,
        "spec § Modulation (03-gpu.md L1604): cor 808080h e equivalente a nao modular"
    );
}

#[test]
fn bit_raw_ignora_a_cor_do_vertice_mesmo_preta() {
    let mut gpu = Gpu::new();
    escreve_vram_halfword(&mut gpu, 0, 0, 0x2310);
    stat_com_e1h(&mut gpu, 2 << 7);

    triangulo_com_cor(&mut gpu, 0x25, 0x0000_0000, &verts_uv());

    assert_eq!(
        gpu.vram_pixel(10, 10),
        0x2310,
        "GP0 bit24=1 (raw texture, 03-gpu.md L264): cor do vertice e ignorada — \
         com o bug antigo (sempre raw) este teste passava por acidente; o que prova \
         a diferenca e o par com modulacao_escurece_pelo_canal_da_cor_do_vertice"
    );
}

#[test]
fn modulacao_preserva_o_bit_15_stp_do_texel() {
    let mut gpu = Gpu::new();
    escreve_vram_halfword(&mut gpu, 0, 0, 0xA310); // bit15=1, r=16,g=24,b=8
    stat_com_e1h(&mut gpu, 2 << 7);

    triangulo_com_cor(&mut gpu, 0x24, 0x0080_4020, &verts_uv());

    assert_eq!(
        gpu.vram_pixel(10, 10),
        0xA184,
        "bit 15 (mascara/STP) do texel sobrevive a modulacao — RGB modulado (0x2184) \
         com bit15 do texel original, obtido 0x{:04X}",
        gpu.vram_pixel(10, 10)
    );
}
