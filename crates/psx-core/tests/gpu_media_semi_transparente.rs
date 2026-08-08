use psx_core::gpu::Gpu;

fn escreve_halfword(gpu: &mut Gpu, x: u16, y: u16, val: u16) {
    gpu.write32(0, 0xA0u32 << 24);
    gpu.write32(0, (y as u32) << 16 | (x as u32));
    gpu.write32(0, 0x0001_0001);
    gpu.write32(0, val as u32);
}

fn modo_de_desenho_media(gpu: &mut Gpu) {
    gpu.write32(0, 0xE1u32 << 24);
}

fn quad_monocromatico_semi_transparente(gpu: &mut Gpu, cor24: u32) {
    gpu.write32(0, (0x2Au32 << 24) | cor24);
    gpu.write32(0, (10u32 << 16) | 10u32);
    gpu.write32(0, (10u32 << 16) | 14u32);
    gpu.write32(0, (14u32 << 16) | 10u32);
    gpu.write32(0, (14u32 << 16) | 14u32);
}

fn media(fundo5: u16, cor24: u32) -> u16 {
    let mut gpu = Gpu::new();
    modo_de_desenho_media(&mut gpu);
    escreve_halfword(&mut gpu, 11, 11, fundo5);
    quad_monocromatico_semi_transparente(&mut gpu, cor24);
    gpu.vram_pixel(11, 11) & 0x1F
}

#[rustfmt::skip]
#[test]
fn a1_back_31_front_31_da_31_e_nao_30() {
    assert_eq!(
        media(0x1F, 0x0000_00FF), 31,
        "A1: 03-gpu.md L1591 define o modo 0 como '0.5 x B + 0.5 x F', ou seja a media da \
         SOMA — nao a soma das metades. O gabarito de hardware \
         tests/exes/ps1-tests/gpu/quad/vram.png mede fundo branco (B=31) sob quad \
         semi-transparente 0x2A de cor 0xFF (F=31) e le 31 (0xF8) no PNG; (B>>1)+(F>>1) \
         daria 30 (0xF0). Obtido {}",
        media(0x1F, 0x0000_00FF)
    );
}

#[rustfmt::skip]
#[test]
fn a2_back_15_front_31_da_23_e_nao_22() {
    assert_eq!(
        media(0x0F, 0x0000_00FF), 23,
        "A2: mesmo gabarito, na faixa magenta onde o canal do fundo vale 15: (15+31)/2=23 \
         (0xB8 no PNG). A soma das metades daria 7+15=22 (0xB0). Obtido {}",
        media(0x0F, 0x0000_00FF)
    );
}

#[rustfmt::skip]
#[test]
fn a3_soma_impar_trunca_para_baixo_31_e_20_da_25() {
    assert_eq!(
        media(0x1F, 0x0000_00A0), 25,
        "A3: a media trunca, nao arredonda. Os swatches de cor 0xA0 (F=20) sobre fundo \
         B=31 valem 25 (0xC8) no gabarito de hardware; (31+20)/2=25.5 arredondado daria \
         26 (0xD0). Obtido {}",
        media(0x1F, 0x0000_00A0)
    );
}

#[rustfmt::skip]
#[test]
fn a4_soma_impar_trunca_para_baixo_15_e_20_da_17() {
    assert_eq!(
        media(0x0F, 0x0000_00A0), 17,
        "A4: segundo ponto de truncamento do mesmo gabarito: (15+20)/2=17.5 -> 17 (0x88). \
         Arredondar daria 18 (0x90). Obtido {}",
        media(0x0F, 0x0000_00A0)
    );
}
