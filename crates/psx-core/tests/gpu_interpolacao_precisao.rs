use psx_core::gpu::Gpu;

const VERMELHO: u16 = 0x001F;
const VERDE: u16 = 0x03E0;
const TEXPAGE_15BPP: u32 = 0x0180;

fn gpu_com_textura_de_dois_texels() -> Gpu {
    let mut gpu = Gpu::new();
    gpu.vram_raw_mut()[0] = VERMELHO;
    gpu.vram_raw_mut()[1] = VERDE;
    gpu
}

fn faixa_texturizada(gpu: &mut Gpu, largura: i16, y: i16) {
    gpu.write32(0, 0x2D00_0000);
    let cantos: [(i16, i16, u8); 4] = [
        (0, y, 0),
        (largura, y, 1),
        (0, y + 1, 0),
        (largura, y + 1, 1),
    ];
    for (idx, &(sx, sy, u)) in cantos.iter().enumerate() {
        gpu.write32(0, ((sy as u32 & 0xFFFF) << 16) | (sx as u32 & 0xFFFF));
        let mut uv = u as u32;
        if idx == 1 {
            uv |= TEXPAGE_15BPP << 16;
        }
        gpu.write32(0, uv);
    }
}

fn primeiro_verde(gpu: &Gpu, largura: i16, y: i16) -> Option<i16> {
    (0..largura).find(|&x| gpu.vram_pixel(x as u16, y as u16) == VERDE)
}

#[test]
fn u_vira_1_no_texel_medido_no_hardware_para_span_de_200() {
    let mut gpu = gpu_com_textura_de_dois_texels();
    faixa_texturizada(&mut gpu, 200, 200);

    assert_eq!(
        gpu.vram_pixel(102, 200),
        VERMELHO,
        "x=102 de um span de 200 ainda amostra u=0 no hardware (vram.png de uv-interpolation)"
    );
    assert_eq!(
        gpu.vram_pixel(103, 200),
        VERDE,
        "x=103 ja amostra u=1: o passo e trunc(4096*du/dx)=20 e 103*20 e o primeiro >= 2048"
    );
}

#[test]
fn a_borda_do_u_segue_o_gradiente_de_12_bits_em_varios_spans() {
    let esperado: [(i16, i16); 8] = [
        (2, 1),
        (6, 4),
        (8, 4),
        (10, 6),
        (20, 11),
        (60, 31),
        (120, 61),
        (255, 128),
    ];
    for (largura, borda) in esperado {
        let mut gpu = gpu_com_textura_de_dois_texels();
        faixa_texturizada(&mut gpu, largura, 300);
        assert_eq!(
            primeiro_verde(&gpu, largura, 300),
            Some(borda),
            "span={largura}: a escada medida em hardware poe a troca de texel em x={borda}"
        );
    }
}

fn faixa_gouraud(gpu: &mut Gpu, largura: i16, y: i16) {
    const RED24: u32 = 0x0000_00FF;
    const GRN24: u32 = 0x0000_FF00;
    gpu.write32(0, 0x3800_0000 | RED24);
    let cantos: [(i16, i16, u32); 4] = [
        (0, y, RED24),
        (largura, y, GRN24),
        (0, y + 1, RED24),
        (largura, y + 1, GRN24),
    ];
    for (idx, &(sx, sy, cor)) in cantos.iter().enumerate() {
        if idx > 0 {
            gpu.write32(0, cor);
        }
        gpu.write32(0, ((sy as u32 & 0xFFFF) << 16) | (sx as u32 & 0xFFFF));
    }
}

#[test]
fn gouraud_interpola_em_8_bits_antes_de_cortar_para_5() {
    let mut gpu = Gpu::new();
    faixa_gouraud(&mut gpu, 200, 456);

    let esperado: [(u16, u16, u16); 8] = [
        (0, 31, 0),
        (1, 31, 0),
        (50, 23, 8),
        (99, 16, 15),
        (100, 16, 15),
        (101, 15, 16),
        (150, 8, 23),
        (199, 0, 31),
    ];
    for (x, r5, g5) in esperado {
        assert_eq!(
            gpu.vram_pixel(x, 456),
            r5 | (g5 << 5),
            "x={x} de um span de 200: vram.png de uv-interpolation mede R5={r5} G5={g5}"
        );
    }
}

#[test]
fn span_potencia_de_dois_troca_exatamente_na_metade() {
    for expoente in 1..=7u32 {
        let largura = (1i16) << expoente;
        let mut gpu = gpu_com_textura_de_dois_texels();
        faixa_texturizada(&mut gpu, largura, 400);
        assert_eq!(
            primeiro_verde(&gpu, largura, 400),
            Some(largura / 2),
            "span={largura} divide 4096 exatamente, entao o empate em 0,5 sobe"
        );
    }
}
