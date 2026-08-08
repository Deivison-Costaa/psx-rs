use psx_core::gpu::Gpu;

const TEXPAGE_15BPP_X512: u32 = 0x108;

fn modula(texel: u16, cor24: u32) -> u16 {
    let mut gpu = Gpu::new();
    gpu.write32(0, (0xE1u32 << 24) | TEXPAGE_15BPP_X512);
    gpu.write32(0, 0xA0u32 << 24);
    gpu.write32(0, 512u32);
    gpu.write32(0, 0x0001_0001);
    gpu.write32(0, texel as u32);
    gpu.write32(0, (0x64u32 << 24) | cor24);
    gpu.write32(0, (10u32 << 16) | 10u32);
    gpu.write32(0, 0);
    gpu.write32(0, (1u32 << 16) | 1u32);
    gpu.vram_pixel(10, 10)
}

fn canal(pixel: u16, indice: u32) -> u16 {
    (pixel >> (5 * indice)) & 0x1F
}

#[rustfmt::skip]
#[test]
fn a1_vermelho_25_com_cor_0x87_da_26_e_nao_25() {
    let obtido = canal(modula(25, 0x0000_0087), 0);
    assert_eq!(
        obtido, 26,
        "A1: 03-gpu.md L1610 define a modulacao sobre canais de 8 BITS \
         (texel*vertexColour/128) e L1080 da a formula do GPU v2 como (color*texel)/16. \
         Com texel=25 e cor=0x87 o hardware faz 25*135/16=210 em 8 bits, que viram 26 nos \
         5 bits do framebuffer. Cortar a cor para 5 bits ANTES do produto e o comportamento \
         do GPU v0 (L1098-1099), que nao e o que emulamos, e da 25. Obtido {obtido}"
    );
}

#[rustfmt::skip]
#[test]
fn a2_verde_20_com_cor_0x8f_da_22_e_nao_21() {
    let obtido = canal(modula(20 << 5, 0x0000_8F00), 1);
    assert_eq!(
        obtido, 22,
        "A2: mesmo defeito no canal verde, para provar que a correcao nao e so do vermelho: \
         20*143/16=178 em 8 bits -> 22 em 5 bits. Com a cor cortada para 5 bits antes \
         (0x8F>>3=17) sai 20*17/16=21. Obtido {obtido}"
    );
}

#[rustfmt::skip]
#[test]
fn a3_azul_13_com_cor_0x9e_da_16_e_nao_15() {
    let obtido = canal(modula(13 << 10, 0x009E_0000), 2);
    assert_eq!(
        obtido, 16,
        "A3: canal azul: 13*158/16=128 em 8 bits -> 16 em 5 bits; cor cortada antes \
         (0x9E>>3=19) daria 13*19/16=15. Obtido {obtido}"
    );
}

#[rustfmt::skip]
#[test]
fn a4_cor_neutra_0x80_devolve_o_texel_intacto() {
    let obtido = canal(modula(25, 0x0000_0080), 0);
    assert_eq!(
        obtido, 25,
        "A4: controle. 03-gpu.md L465 diz que 80h e o ponto neutro da modulacao \
         (25*128/16=200 -> 25). Ampliar a precisao nao pode mexer nesse ponto. Obtido {obtido}"
    );
}

#[rustfmt::skip]
#[test]
fn a5_produto_trunca_nao_arredonda() {
    let obtido = canal(modula(17, 0x0000_0087), 0);
    assert_eq!(
        obtido, 17,
        "A5: controle de arredondamento. 17*135/16=143.4 -> 143 em 8 bits, e 143>>3=17.875 \
         -> 17 em 5 bits. Arredondar em qualquer um dos dois estagios daria 18. Obtido {obtido}"
    );
}

#[rustfmt::skip]
#[test]
fn a7_produto_120_trunca_para_zero_e_nao_para_um() {
    let obtido = canal(modula(15, 0x0000_0008), 0);
    assert_eq!(
        obtido, 0,
        "A7: segundo ponto de truncamento, escolhido onde arredondar o PRODUTO muda o \
         resultado: 15*8=120, e 120/16=7.5 -> 7 em 8 bits, que da 0 em 5 bits. Somar meio \
         antes de dividir por 16 levaria a 8 em 8 bits e 1 em 5 bits. Obtido {obtido}"
    );
}

#[rustfmt::skip]
#[test]
fn a6_satura_em_31_sem_dar_a_volta() {
    let obtido = canal(modula(31, 0x0000_00FF), 0);
    assert_eq!(
        obtido, 31,
        "A6: controle de saturacao. 03-gpu.md L465-469: cores acima de 80h deixam a \
         primitiva 'mais brilhante que o brilho', mas o valor de 5 bits satura em 1Fh. \
         31*255/16=494, muito acima de 255. Obtido {obtido}"
    );
}
