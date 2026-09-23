use psx_core::gpu::Gpu;

fn coord(x: u16, y: u16) -> u32 {
    ((y as u32) << 16) | x as u32
}

fn bloco_gradiente(gpu: &mut Gpu, x0: u16, y0: u16, lado: u16) {
    for y in 0..lado {
        for x in 0..lado {
            let cor = (x * 2) | ((y * 2) << 5);
            gpu.vram_raw_mut()[(y0 + y) as usize * 1024 + (x0 + x) as usize] = cor;
        }
    }
}

fn blit(gpu: &mut Gpu, src: u32, dst: u32, tam: u32) {
    for palavra in [0x8000_0000, src, dst, tam] {
        gpu.write32(0, palavra);
    }
}

fn linha(gpu: &Gpu, x0: u16, y: u16, n: u16) -> Vec<u16> {
    (x0..x0 + n).map(|x| gpu.vram_pixel(x, y)).collect()
}

#[test]
fn copia_uma_linha_para_baixo_propaga_a_primeira_linha() {
    let mut gpu = Gpu::new();
    bloco_gradiente(&mut gpu, 760, 130, 16);

    blit(&mut gpu, coord(760, 130), coord(760, 131), coord(16, 16));

    let primeira: Vec<u16> = (0..16).map(|x| x * 2).collect();
    for y in 131..147 {
        assert_eq!(
            linha(&gpu, 760, y, 16),
            primeira,
            "ps1-tests gpu/vram-to-vram-overlap (vram.png do hardware, celula BLOCK 16 0:1): \
             o GP0(80h) copia linha a linha de cima para baixo, lendo a origem ja reescrita; \
             com dy=+1 a primeira linha escorre pelo bloco inteiro (linha {y})"
        );
    }
}

#[test]
fn copia_diagonal_para_baixo_forma_escada() {
    let mut gpu = Gpu::new();
    bloco_gradiente(&mut gpu, 844, 130, 16);

    blit(&mut gpu, coord(844, 130), coord(846, 131), coord(16, 16));

    assert_eq!(
        linha(&gpu, 844, 132, 8),
        vec![
            0x0080, 0x0082, 0x0040, 0x0042, 0x0000, 0x0002, 0x0004, 0x0006
        ],
        "hardware (celula BLOCK 16 2:1, linha 132): 0004 0204 0002 0202 0000 0200 0400 0600 \
         em (R,G) de 5 bits — a linha 131 ja copiada e a fonte da linha 132"
    );
    assert_eq!(
        linha(&gpu, 846, 146, 4),
        vec![0x03C0, 0x03C2, 0x0380, 0x0382],
        "hardware (celula BLOCK 16 2:1, linha 146): 0030 0230 0028 0228"
    );
}

#[test]
fn copia_para_cima_nao_propaga() {
    let mut gpu = Gpu::new();
    bloco_gradiente(&mut gpu, 100, 50, 4);

    blit(&mut gpu, coord(100, 50), coord(100, 49), coord(4, 4));

    for y in 0..4u16 {
        let esperado: Vec<u16> = (0..4).map(|x| (x * 2) | ((y * 2) << 5)).collect();
        assert_eq!(
            linha(&gpu, 100, 49 + y, 4),
            esperado,
            "hardware (celulas dy=-1): de cima para baixo, a linha lida nunca foi escrita antes"
        );
    }
}
