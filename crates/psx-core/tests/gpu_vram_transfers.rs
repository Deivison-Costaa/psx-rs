use psx_core::gpu::Gpu;

#[test]
fn a1_fill_16x1_converte_cor_24_para_15_bits() {
    let mut gpu = Gpu::new();

    gpu.write32(0, (0x02u32 << 24) | 0x00F81820);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0001_0010);

    let expected: u16 = 0x7C64;
    assert_eq!(
        gpu.vram_pixel(0, 0),
        expected,
        "A1: pixel(0,0) deve ser 0x7C64, obtido 0x{:04X}",
        gpu.vram_pixel(0, 0)
    );
    assert_eq!(
        gpu.vram_pixel(15, 0),
        expected,
        "A1: pixel(15,0) deve ser 0x7C64, obtido 0x{:04X}",
        gpu.vram_pixel(15, 0)
    );
}

#[test]
fn a2_fill_arredonda_xpos_e_xsiz() {
    let mut gpu = Gpu::new();

    gpu.write32(0, (0x02u32 << 24) | 0x00FF_FFFF);
    gpu.write32(0, 0x0000_001F);
    gpu.write32(0, 0x0001_0011);

    let val = gpu.vram_pixel(0x10, 0);
    assert_eq!(val, 0x7FFF, "A2: Xpos=0x1F arredondado para 0x10, pixel obtido 0x{:04X}", val);

    let val = gpu.vram_pixel(0x2F, 0);
    assert_eq!(val, 0x7FFF, "A2: Xsiz=0x11 arredondado para 0x20, pixel(0x2F) obtido 0x{:04X}", val);

    assert_eq!(gpu.vram_pixel(0x0F, 0), 0, "A2: pixel antes da area (0x0F) deve ser 0");
    assert_eq!(gpu.vram_pixel(0x30, 0), 0, "A2: pixel depois da area (0x30) deve ser 0");
}

#[test]
fn a3_fill_com_ysiz_zero_ou_512_nao_escreve_nada() {
    let mut gpu = Gpu::new();

    gpu.write32(0, (0x02u32 << 24) | 0x0000_FFFF);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0000_0010);
    assert_eq!(gpu.vram_pixel(0, 0), 0, "A3: Ysiz=0 nao deve escrever");

    let mut gpu = Gpu::new();
    gpu.write32(0, (0x02u32 << 24) | 0x0000_FFFF);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0200_0010);
    assert_eq!(gpu.vram_pixel(0, 0), 0, "A3: Ysiz=512 AND 0x1FF=0 nao deve escrever");
}

#[test]
fn a4_fill_nao_respeita_mask_bit() {
    let mut gpu = Gpu::new();

    let gp0_addr: u32 = 0;
    let gp1_addr: u32 = 4;

    gpu.write32(gp1_addr, 0x00 << 24);

    gpu.write32(gp0_addr, (0xE6u32 << 24) | 0x3);
    let stat = gpu.read32(gp1_addr);
    assert_eq!((stat >> 11) & 1, 1, "A4: depois de GP0(E6h) com bits0-1=3, bit11 deve ser 1");

    gpu.write32(0, (0x02u32 << 24) | 0x00FF_FFFF);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0001_0010);

    let pixel = gpu.vram_pixel(0, 0);
    assert_eq!(
        pixel & 0x8000,
        0,
        "A4: fill nao respeita mask: bit15 do pixel deve ser 0, obtido 0x{:04X}",
        pixel
    );
}

#[test]
fn a5_a0h_cpu_para_vram_2x1_escreve_baixo_primeiro() {
    let mut gpu = Gpu::new();

    let cmd_top3_5: u32 = (0xA0u32) << 24;
    gpu.write32(0, cmd_top3_5);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0001_0002);
    gpu.write32(0, 0x0001_C0DE);

    assert_eq!(gpu.vram_pixel(0, 0), 0xC0DE, "A5: primeiro pixel (low) deve ser 0xC0DE");
    assert_eq!(gpu.vram_pixel(1, 0), 0x0001, "A5: segundo pixel (high) deve ser 0x0001");
}

#[test]
fn a6_a0h_impar_descarta_halfword_extra() {
    let mut gpu = Gpu::new();

    let cmd_top3_5: u32 = (0xA0u32) << 24;
    gpu.write32(0, cmd_top3_5);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0001_0003);
    gpu.write32(0, 0x0002_0001);
    gpu.write32(0, 0xDEAD_0004);

    assert_eq!(gpu.vram_pixel(0, 0), 0x0001, "A6: pixel(0,0) da 1a palavra");
    assert_eq!(gpu.vram_pixel(1, 0), 0x0002, "A6: pixel(1,0) da 1a palavra");
    assert_eq!(gpu.vram_pixel(2, 0), 0x0004, "A6: pixel(2,0) da 2a palavra (halfword extra descartada)");
    assert_eq!(gpu.vram_pixel(3, 0), 0, "A6: pixel(3,0) nao deve ter sido escrito");
}

#[test]
fn a7_a0h_com_xsiz_zero_transfere_max_0x400() {
    let mut gpu = Gpu::new();

    let cmd_top3_5: u32 = (0xA0u32) << 24;
    gpu.write32(0, cmd_top3_5);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0001_0000);

    for i in 0..512 {
        let low = (i as u32 * 2) as u16;
        let high = (i as u32 * 2 + 1) as u16;
        gpu.write32(0, (high as u32) << 16 | low as u32);
    }

    for col in 0..1024u16 {
        assert_eq!(
            gpu.vram_pixel(col, 0),
            col,
            "A7: pixel({},0) deve ser {}, obtido {}",
            col,
            col,
            gpu.vram_pixel(col, 0)
        );
    }
}

#[test]
fn a8_c0h_devolve_pelo_gpuread_o_que_a0h_escreveu() {
    let mut gpu = Gpu::new();

    let cmd_a0: u32 = (0xA0u32) << 24;
    gpu.write32(0, cmd_a0);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0003_0004);
    gpu.write32(0, 0x0002_0001);
    gpu.write32(0, 0x0004_0003);
    gpu.write32(0, 0x0006_0005);
    gpu.write32(0, 0x0008_0007);
    gpu.write32(0, 0x000A_0009);
    gpu.write32(0, 0x000C_000B);

    let cmd_c0: u32 = (0xC0u32) << 24;
    gpu.write32(0, cmd_c0);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0003_0004);

    let w0 = gpu.read32(0);
    assert_eq!(w0, 0x0002_0001, "A8: palavra 0 lida deve ser 0x0002_0001, obtida 0x{:08X}", w0);
    let w1 = gpu.read32(0);
    assert_eq!(w1, 0x0004_0003, "A8: palavra 1 lida deve ser 0x0004_0003, obtida 0x{:08X}", w1);
    let w2 = gpu.read32(0);
    assert_eq!(w2, 0x0006_0005, "A8: palavra 2 lida deve ser 0x0006_0005, obtida 0x{:08X}", w2);
    let w3 = gpu.read32(0);
    assert_eq!(w3, 0x0008_0007, "A8: palavra 3 lida deve ser 0x0008_0007, obtida 0x{:08X}", w3);
    let w4 = gpu.read32(0);
    assert_eq!(w4, 0x000A_0009, "A8: palavra 4 lida deve ser 0x000A_0009, obtida 0x{:08X}", w4);
    let w5 = gpu.read32(0);
    assert_eq!(w5, 0x000C_000B, "A8: palavra 5 (12 pixels=par, ambos halfwords), obtida 0x{:08X}", w5);
}

#[test]
fn a9_wrap_fill_comecando_em_x_1020_sem_carry_para_proxima_linha() {
    let mut gpu = Gpu::new();

    gpu.write32(0, (0x02u32 << 24) | 0x0010_3020);
    gpu.write32(0, 0x0000_03F0);
    gpu.write32(0, 0x0001_0020);

    let expected: u16 = 0x08C4;
    assert_eq!(gpu.vram_pixel(1008, 0), expected, "A9-fill: pixel(1008,0) inicio");
    assert_eq!(gpu.vram_pixel(1023, 0), expected, "A9-fill: pixel(1023,0) antes do wrap");
    assert_eq!(gpu.vram_pixel(0, 0), expected, "A9-fill: pixel(0,0) wrap para inicio da linha");
    assert_eq!(gpu.vram_pixel(15, 0), expected, "A9-fill: pixel(15,0) fim apos wrap");
    assert_eq!(gpu.vram_pixel(16, 0), 0, "A9-fill: pixel(16,0) fora da area preenchida");
    assert_eq!(gpu.vram_pixel(0, 1), 0, "A9-fill: pixel(0,1) sem carry X->Y");
}

#[test]
fn a9_wrap_copy_comecando_em_x_1020_sem_carry_para_proxima_linha() {
    let mut gpu = Gpu::new();

    let cmd_a0: u32 = (0xA0u32) << 24;
    gpu.write32(0, cmd_a0);
    gpu.write32(0, 0x0000_03FC);
    gpu.write32(0, 0x0001_0008);
    for i in 0..4 {
        let low = (i * 2) as u32;
        let high = (i * 2 + 1) as u32;
        gpu.write32(0, (high << 16) | low);
    }

    for i in 0..4 {
        let px = (1020 + i) & 0x3FF;
        let expected = i as u16;
        assert_eq!(
            gpu.vram_pixel(px, 0),
            expected,
            "A9-copy: pixel({},0) deve ser {}, obtido {}",
            px,
            expected,
            gpu.vram_pixel(px, 0)
        );
    }
    assert_eq!(gpu.vram_pixel(4, 0) & 0x3FF, 0, "A9-copy: pixel(4,0) deve ser 0");
    assert_eq!(gpu.vram_pixel(0, 1), 0, "A9-copy: sem carry X->Y");
}

#[test]
fn a10_gpustat_bit27_c0h() {
    let mut gpu = Gpu::new();

    let stat = gpu.read32(4);
    assert_eq!((stat >> 27) & 1, 0, "A10: antes do C0h, bit27 deve ser 0");

    let cmd_c0: u32 = (0xC0u32) << 24;
    gpu.write32(0, cmd_c0);
    let stat = gpu.read32(4);
    assert_eq!((stat >> 27) & 1, 0, "A10: depois da 1a palavra, bit27 ainda deve ser 0");

    gpu.write32(0, 0x0000_0000);
    let stat = gpu.read32(4);
    assert_eq!((stat >> 27) & 1, 0, "A10: depois da 2a palavra, bit27 ainda deve ser 0");

    gpu.write32(0, 0x0001_0004);
    let stat = gpu.read32(4);
    assert_eq!((stat >> 27) & 1, 1, "A10: depois do cabecalho completo (3 palavras), bit27 deve ser 1");

    gpu.read32(0);
    let stat = gpu.read32(4);
    assert_eq!((stat >> 27) & 1, 1, "A10: depois de ler a 1a palavra, bit27 continua 1 (ainda ha dados)");

    gpu.read32(0);
    let stat = gpu.read32(4);
    assert_eq!(
        (stat >> 27) & 1,
        0,
        "A10: depois da ultima palavra lida (2 de 2), bit27 deve voltar a 0"
    );
}

#[test]
fn bit26_vai_a_zero_durante_fill_e_volta() {
    let mut gpu = Gpu::new();

    let stat = gpu.read32(4);
    assert_eq!((stat >> 26) & 1, 1, "No Idle, bit26=1");

    gpu.write32(0, (0x02u32 << 24) | 0x00F81820);
    let stat = gpu.read32(4);
    assert_eq!((stat >> 26) & 1, 0, "Depois da 1a palavra do fill, bit26=0");

    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0001_0010);
    let stat = gpu.read32(4);
    assert_eq!((stat >> 26) & 1, 1, "Depois do fill completo (3a palavra), bit26=1");
}

#[test]
fn bit26_vai_a_zero_durante_a0h_e_volta() {
    let mut gpu = Gpu::new();

    let cmd_a0: u32 = (0xA0u32) << 24;
    gpu.write32(0, cmd_a0);
    let stat = gpu.read32(4);
    assert_eq!((stat >> 26) & 1, 0, "Depois da 1a palavra do A0h, bit26=0");

    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0001_0002);
    let stat = gpu.read32(4);
    assert_eq!((stat >> 26) & 1, 0, "Durante dados de A0h, bit26=0");

    gpu.write32(0, 0xDEAD_BEEF);
    let stat = gpu.read32(4);
    assert_eq!((stat >> 26) & 1, 1, "Depois do ultimo dado de A0h, bit26=1");
}

#[test]
fn bit26_vai_a_zero_durante_c0h_e_volta_apos_leitura() {
    let mut gpu = Gpu::new();

    let cmd_a0: u32 = (0xA0u32) << 24;
    gpu.write32(0, cmd_a0);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0001_0002);
    gpu.write32(0, 0xDEAD_BEEF);

    let cmd_c0: u32 = (0xC0u32) << 24;
    gpu.write32(0, cmd_c0);
    let stat = gpu.read32(4);
    assert_eq!((stat >> 26) & 1, 0, "Depois da 1a palavra do C0h, bit26=0");

    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0001_0002);
    let stat = gpu.read32(4);
    assert_eq!((stat >> 26) & 1, 0, "Durante C0h (dados disponiveis), bit26=0");

    gpu.read32(0);
    let stat = gpu.read32(4);
    assert_eq!((stat >> 26) & 1, 1, "Depois de ler o unico dado de C0h, bit26=1");
}

#[test]
fn gp1_00h_reset_limpa_vram() {
    let mut gpu = Gpu::new();

    gpu.write32(0, (0x02u32 << 24) | 0x00F81820);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0001_0010);
    assert_ne!(gpu.vram_pixel(0, 0), 0, "Antes do reset, pixel deve ser nao-zero");

    gpu.write32(4, 0x00 << 24);
    assert_eq!(gpu.vram_pixel(0, 0), 0, "Depois de GP1(00h), VRAM deve ser 0");
}

#[test]
fn a0h_ysiz_513_mascara_para_1_linha() {
    let mut gpu = Gpu::new();

    let cmd_a0: u32 = (0xA0u32) << 24;
    gpu.write32(0, cmd_a0);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0201_0001);

    gpu.write32(0, 0x0000_00AA);

    assert_eq!(gpu.vram_pixel(0, 0), 0x00AA, "Ysiz=513 mascara para 1 linha: pixel(0,0)");
    assert_eq!(gpu.vram_pixel(0, 1), 0, "Ysiz=513 mascara para 1 linha: pixel(0,1) deve ser 0");
}

#[test]
fn a0h_xsiz_1024_mascara_para_0_colunas_vira_max() {
    let mut gpu = Gpu::new();

    let cmd_a0: u32 = (0xA0u32) << 24;
    gpu.write32(0, cmd_a0);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0001_0400);

    let mut word_idx: u32 = 0;
    for _ in 0..512 {
        let low = word_idx;
        word_idx += 1;
        let high = word_idx;
        word_idx += 1;
        gpu.write32(0, (high << 16) | low);
    }

    for col in 0..1024u16 {
        assert_eq!(gpu.vram_pixel(col, 0), col, "A7b: Xsiz=1024 -> max: pixel({},0)", col);
    }
}
