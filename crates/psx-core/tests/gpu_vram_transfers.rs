use psx_core::gpu::Gpu;

#[rustfmt::skip]
#[test]
fn a1_fill_16x1_converte_cor_24_para_15_bits() {
    let mut gpu = Gpu::new();

    gpu.write32(0, (0x02u32 << 24) | 0x00F81820);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0001_0010);

    let expected: u16 = 0x7C64;
    assert_eq!(gpu.vram_pixel(0, 0), expected,
        "A1: pixel(0,0) deve ser 0x7C64, obtido 0x{:04X}", gpu.vram_pixel(0, 0));
    assert_eq!(gpu.vram_pixel(15, 0), expected,
        "A1: pixel(15,0) deve ser 0x7C64, obtido 0x{:04X}", gpu.vram_pixel(15, 0));
}

#[rustfmt::skip]
#[test]
fn a2_fill_arredonda_xpos_e_xsiz() {
    let mut gpu = Gpu::new();

    gpu.write32(0, (0x02u32 << 24) | 0x00FF_FFFF);
    gpu.write32(0, 0x0000_001F);
    gpu.write32(0, 0x0001_0011);

    let val = gpu.vram_pixel(0x10, 0);
    assert_eq!(val, 0x7FFF, "A2: Xpos=0x1F arredondado para 0x10, obtido 0x{:04X}", val);
    let val = gpu.vram_pixel(0x2F, 0);
    assert_eq!(val, 0x7FFF, "A2: Xsiz=0x11 arredondado para 0x20, obtido 0x{:04X}", val);
    assert_eq!(gpu.vram_pixel(0x0F, 0), 0, "A2: antes da area (0x0F) deve ser 0");
    assert_eq!(gpu.vram_pixel(0x30, 0), 0, "A2: depois da area (0x30) deve ser 0");
}

#[test]
fn a3_fill_com_ysiz_zero_ou_512_nao_escreve_nada() {
    let mut gpu = Gpu::new();

    gpu.write32(0, (0x02u32 << 24) | 0x0000_FFFF);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0000_0010);
    assert_eq!(gpu.vram_pixel(0, 0), 0, "A3: Ysiz=0 nao escreve");

    let mut gpu = Gpu::new();
    gpu.write32(0, (0x02u32 << 24) | 0x0000_FFFF);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0200_0010);
    assert_eq!(
        gpu.vram_pixel(0, 0),
        0,
        "A3: Ysiz=512 & 0x1FF=0 nao escreve"
    );
}

#[rustfmt::skip]
#[test]
fn a4_fill_nao_respeita_mask_bit() {
    let mut gpu = Gpu::new();

    let gp0_addr: u32 = 0;
    let gp1_addr: u32 = 4;

    gpu.write32(gp1_addr, 0x00 << 24);

    gpu.write32(gp0_addr, (0xE6u32 << 24) | 0x3);
    let stat = gpu.read32(gp1_addr);
    assert_eq!((stat >> 11) & 1, 1, "A4: GP0(E6h) bit0-1=3 -> bit11=1");
    assert_eq!((stat >> 12) & 1, 1, "A4: GP0(E6h) bit1 -> bit12=1 (write-protect)");

    gpu.write32(0, 0xA0u32 << 24);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0001_0002);
    gpu.write32(0, 0x8000_8000);
    assert_eq!(gpu.vram_pixel(0, 0), 0x8000, "A4: pixel com bit15=1 pre-carregado");

    gpu.write32(0, (0x02u32 << 24) | 0x00F81820);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0001_0010);

    assert_eq!(gpu.vram_pixel(0, 0), 0x7C64,
        "A4: fill sobrescreve pixel protegido e nao liga bit15, obtido 0x{:04X}",
        gpu.vram_pixel(0, 0));
    assert_eq!(gpu.vram_pixel(15, 0), 0x7C64, "A4: fim da area tambem");
}

#[test]
fn a5_a0h_cpu_para_vram_2x1_escreve_baixo_primeiro() {
    let mut gpu = Gpu::new();

    let cmd_top3_5: u32 = (0xA0u32) << 24;
    gpu.write32(0, cmd_top3_5);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0001_0002);
    gpu.write32(0, 0x0001_C0DE);

    assert_eq!(gpu.vram_pixel(0, 0), 0xC0DE, "A5: pixel(0,0) low = 0xC0DE");
    assert_eq!(gpu.vram_pixel(1, 0), 0x0001, "A5: pixel(1,0) high = 0x0001");
}

#[rustfmt::skip]
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
    assert_eq!(gpu.vram_pixel(2, 0), 0x0004, "A6: pixel(2,0) 2a palavra (extra descartada)");
    assert_eq!(gpu.vram_pixel(0, 1), 0, "A6: pixel(0,1) nao escrito (extra descartada)");
}

#[rustfmt::skip]
fn a0h_linha_cheia(raw_xsiz: u32) {
    let mut gpu = Gpu::new();

    gpu.write32(0, 0xA0u32 << 24);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0001_0000 | raw_xsiz);
    for i in 0..512u32 {
        gpu.write32(0, ((i * 2 + 1) << 16) | (i * 2));
    }

    for col in 0..1024u16 {
        assert_eq!(gpu.vram_pixel(col, 0), col,
            "Xsiz bruto {:#X} vira 1024 colunas: pixel({},0) deve ser {}, obtido {}",
            raw_xsiz, col, col, gpu.vram_pixel(col, 0));
    }
}

#[test]
fn a7_a0h_com_xsiz_zero_transfere_max_0x400() {
    a0h_linha_cheia(0);
}

#[rustfmt::skip]
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
    assert_eq!(w0, 0x0002_0001, "A8: w0=0x0002_0001, obtida 0x{:08X}", w0);
    let w1 = gpu.read32(0);
    assert_eq!(w1, 0x0004_0003, "A8: w1=0x0004_0003, obtida 0x{:08X}", w1);
    let w2 = gpu.read32(0);
    assert_eq!(w2, 0x0006_0005, "A8: w2=0x0006_0005, obtida 0x{:08X}", w2);
    let w3 = gpu.read32(0);
    assert_eq!(w3, 0x0008_0007, "A8: w3=0x0008_0007, obtida 0x{:08X}", w3);
    let w4 = gpu.read32(0);
    assert_eq!(w4, 0x000A_0009, "A8: w4=0x000A_0009, obtida 0x{:08X}", w4);
    let w5 = gpu.read32(0);
    assert_eq!(w5, 0x000C_000B, "A8: w5 (12px par), obtida 0x{:08X}", w5);
}

#[test]
fn a9_wrap_fill_comecando_em_x_1020_sem_carry_para_proxima_linha() {
    let mut gpu = Gpu::new();

    gpu.write32(0, (0x02u32 << 24) | 0x0010_3020);
    gpu.write32(0, 0x0000_03F0);
    gpu.write32(0, 0x0001_0020);

    let expected: u16 = 0x08C4;
    assert_eq!(gpu.vram_pixel(1008, 0), expected, "A9-fill: pixel(1008,0)");
    assert_eq!(
        gpu.vram_pixel(1023, 0),
        expected,
        "A9-fill: pixel(1023,0) antes wrap"
    );
    assert_eq!(gpu.vram_pixel(0, 0), expected, "A9-fill: pixel(0,0) wrap");
    assert_eq!(
        gpu.vram_pixel(15, 0),
        expected,
        "A9-fill: pixel(15,0) fim pos wrap"
    );
    assert_eq!(gpu.vram_pixel(16, 0), 0, "A9-fill: pixel(16,0) fora");
    assert_eq!(
        gpu.vram_pixel(0, 1),
        0,
        "A9-fill: pixel(0,1) sem carry X->Y"
    );
}

#[rustfmt::skip]
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

    for i in 0..4u16 {
        assert_eq!(gpu.vram_pixel(1020 + i, 0), i,
            "A9-copy: pixel({},0) antes do wrap deve ser {}", 1020 + i, i);
    }
    for i in 0..4u16 {
        assert_eq!(gpu.vram_pixel(i, 0), i + 4,
            "A9-copy: pixel({},0) DEPOIS do wrap deve ser {}, obtido {}",
            i, i + 4, gpu.vram_pixel(i, 0));
    }
    assert_eq!(gpu.vram_pixel(4, 0), 0, "A9-copy: pixel(4,0)=0");
    assert_eq!(gpu.vram_pixel(0, 1), 0, "A9-copy: sem carry X->Y");
}

#[rustfmt::skip]
#[test]
fn a10_gpustat_bit27_c0h() {
    let mut gpu = Gpu::new();

    let stat = gpu.read32(4);
    assert_eq!((stat >> 27) & 1, 0, "A10: antes do C0h, bit27=0");

    let cmd_c0: u32 = (0xC0u32) << 24;
    gpu.write32(0, cmd_c0);
    let stat = gpu.read32(4);
    assert_eq!((stat >> 27) & 1, 0, "A10: 1a palavra, bit27=0");

    gpu.write32(0, 0x0000_0000);
    let stat = gpu.read32(4);
    assert_eq!((stat >> 27) & 1, 0, "A10: 2a palavra, bit27=0");

    gpu.write32(0, 0x0001_0004);
    let stat = gpu.read32(4);
    assert_eq!((stat >> 27) & 1, 1, "A10: cabecalho completo, bit27=1");

    gpu.read32(0);
    let stat = gpu.read32(4);
    assert_eq!((stat >> 27) & 1, 1, "A10: apos ler 1a palavra, bit27=1");

    gpu.read32(0);
    let stat = gpu.read32(4);
    assert_eq!((stat >> 27) & 1, 0, "A10: ultima palavra lida, bit27=0");
}

#[test]
fn bit26_vai_a_zero_durante_fill_e_volta() {
    let mut gpu = Gpu::new();

    let stat = gpu.read32(4);
    assert_eq!((stat >> 26) & 1, 1, "Idle, bit26=1");

    gpu.write32(0, (0x02u32 << 24) | 0x00F81820);
    let stat = gpu.read32(4);
    assert_eq!((stat >> 26) & 1, 0, "1a palavra fill, bit26=0");

    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0001_0010);
    let stat = gpu.read32(4);
    assert_eq!((stat >> 26) & 1, 1, "fill completo, bit26=1");
}

#[test]
fn bit26_vai_a_zero_durante_a0h_e_volta() {
    let mut gpu = Gpu::new();

    let cmd_a0: u32 = (0xA0u32) << 24;
    gpu.write32(0, cmd_a0);
    let stat = gpu.read32(4);
    assert_eq!((stat >> 26) & 1, 0, "1a palavra A0h, bit26=0");

    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0001_0002);
    let stat = gpu.read32(4);
    assert_eq!((stat >> 26) & 1, 0, "durante dados A0h, bit26=0");

    gpu.write32(0, 0xDEAD_BEEF);
    let stat = gpu.read32(4);
    assert_eq!((stat >> 26) & 1, 1, "ultimo dado A0h, bit26=1");
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
    assert_eq!((stat >> 26) & 1, 0, "1a palavra C0h, bit26=0");

    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0001_0002);
    let stat = gpu.read32(4);
    assert_eq!((stat >> 26) & 1, 0, "durante C0h, bit26=0");

    gpu.read32(0);
    let stat = gpu.read32(4);
    assert_eq!((stat >> 26) & 1, 1, "leu unico dado C0h, bit26=1");
}

#[test]
fn gp1_00h_reset_preserva_vram() {
    let mut gpu = Gpu::new();

    gpu.write32(0, (0x02u32 << 24) | 0x00F81820);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0001_0010);
    gpu.write32(0, (0x02u32 << 24) | 0x00F81820);
    gpu.write32(0, (300u32 << 16) | 0x40);
    gpu.write32(0, 0x0001_0010);
    assert_eq!(gpu.vram_pixel(0, 0), 0x7C64, "antes do reset, (0,0)");
    assert_eq!(
        gpu.vram_pixel(0x40, 300),
        0x7C64,
        "antes do reset, (0x40,300)"
    );
    assert_eq!(
        gpu.vram_pixel(0x4F, 300),
        0x7C64,
        "fill na linha 300 chega ao fim"
    );
    assert_eq!(gpu.vram_pixel(0x40, 0), 0, "linha 0 nao recebeu o 2o fill");

    gpu.write32(4, 0x00 << 24);
    assert_eq!(gpu.vram_pixel(0, 0), 0x7C64, "GP1(00h) preserva (0,0)");
    assert_eq!(gpu.vram_pixel(15, 0), 0x7C64, "GP1(00h) preserva (15,0)");
    assert_eq!(
        gpu.vram_pixel(0x40, 300),
        0x7C64,
        "GP1(00h) preserva (0x40,300)"
    );
    assert_eq!(gpu.stat(), 0x1480_2000, "GP1(00h): stat = 14802000h");
}

#[test]
fn a0h_ysiz_513_mascara_para_1_linha() {
    let mut gpu = Gpu::new();

    let cmd_a0: u32 = (0xA0u32) << 24;
    gpu.write32(0, cmd_a0);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0201_0002);

    gpu.write32(0, 0x00BB_00AA);

    assert_eq!(
        gpu.vram_pixel(0, 0),
        0x00AA,
        "Ysiz=513 -> 1 linha: pixel(0,0)"
    );
    assert_eq!(
        gpu.vram_pixel(1, 0),
        0x00BB,
        "Ysiz=513 -> 1 linha: pixel(1,0)"
    );
    assert_eq!(gpu.vram_pixel(0, 1), 0, "Ysiz=513 -> 1 linha: pixel(0,1)=0");
    assert_eq!(
        (gpu.read32(4) >> 26) & 1,
        1,
        "Ysiz=513 -> 1 linha: transferencia terminou com UMA palavra, bit26=1"
    );

    gpu.write32(0, (0x02u32 << 24) | 0x00FF_FFFF);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0001_0010);
    assert_eq!(
        gpu.vram_pixel(5, 0),
        0x7FFF,
        "comando seguinte executa como comando, nao e engolido como dado"
    );
}

#[test]
fn a0h_xsiz_1024_mascara_para_0_colunas_vira_max() {
    a0h_linha_cheia(0x400);
}

#[test]
fn peek32_nao_consome_transferencia_c0h() {
    let mut gpu = Gpu::new();

    let cmd_a0: u32 = (0xA0u32) << 24;
    gpu.write32(0, cmd_a0);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0001_0002);
    gpu.write32(0, 0xBBAA_DDCC);

    let cmd_c0: u32 = (0xC0u32) << 24;
    gpu.write32(0, cmd_c0);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0001_0002);

    let peeked = gpu.peek32(0);
    assert_eq!(peeked, 0xBBAA_DDCC, "peek32 nao consome");

    let word = gpu.read32(0);
    assert_eq!(word, 0xBBAA_DDCC, "read32 apos peek32 devolve 1a palavra");

    let stat = gpu.read32(4);
    assert_eq!((stat >> 27) & 1, 0, "leitura final, bit27=0");
}

#[rustfmt::skip]
#[test]
fn a0h_com_ypos_nao_zero_endereca_linha_absoluta() {
    let mut gpu = Gpu::new();

    gpu.write32(0, 0xA0u32 << 24);
    gpu.write32(0, (100u32 << 16) | 5);
    gpu.write32(0, (2u32 << 16) | 3);
    gpu.write32(0, 0x0002_0001);
    gpu.write32(0, 0x0004_0003);
    gpu.write32(0, 0x0006_0005);

    for (i, col) in (5u16..8).enumerate() {
        assert_eq!(gpu.vram_pixel(col, 100), i as u16 + 1,
            "pixel({},100) deve ser {}, obtido {}", col, i + 1, gpu.vram_pixel(col, 100));
        assert_eq!(gpu.vram_pixel(col, 101), i as u16 + 4,
            "pixel({},101) deve ser {}, obtido {}", col, i + 4, gpu.vram_pixel(col, 101));
    }
    assert_eq!(gpu.vram_pixel(5, 0), 0, "linha 0 intacta");
    assert_eq!(gpu.vram_pixel(5, 50), 0, "linha 50 intacta (stride correto)");
}

#[rustfmt::skip]
#[test]
fn c0h_le_da_linha_absoluta_e_com_wrap_em_x() {
    let mut gpu = Gpu::new();

    gpu.write32(0, 0xA0u32 << 24);
    gpu.write32(0, (200u32 << 16) | 0x3FE);
    gpu.write32(0, 0x0001_0004);
    gpu.write32(0, 0x0002_0001);
    gpu.write32(0, 0x0004_0003);

    assert_eq!(gpu.vram_pixel(1022, 200), 0x0001, "escrita (1022,200)");
    assert_eq!(gpu.vram_pixel(1023, 200), 0x0002, "escrita (1023,200)");
    assert_eq!(gpu.vram_pixel(0, 200), 0x0003, "escrita apos wrap (0,200)");
    assert_eq!(gpu.vram_pixel(1, 200), 0x0004, "escrita apos wrap (1,200)");
    assert_eq!(gpu.vram_pixel(0, 201), 0, "wrap em X nao carrega para Y");

    gpu.write32(0, 0xC0u32 << 24);
    gpu.write32(0, (200u32 << 16) | 0x3FE);
    gpu.write32(0, 0x0001_0004);
    let w0 = gpu.read32(0);
    let w1 = gpu.read32(0);
    assert_eq!(w0, 0x0002_0001, "C0h w0 da linha 200, obtida 0x{:08X}", w0);
    assert_eq!(w1, 0x0004_0003, "C0h w1 apos wrap em X, obtida 0x{:08X}", w1);
}

#[test]
fn top3_4_vram_to_vram_consome_params_e_permite_comando_seguinte() {
    let mut gpu = Gpu::new();

    gpu.write32(0, (0x80u32) << 24);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0001_0002);
    gpu.write32(0, 0xDEAD_BEEF);

    assert_eq!(gpu.vram_pixel(0, 0), 0, "80h nao escreve na VRAM");

    gpu.write32(0, (0x02u32 << 24) | 0x00FF_FFFF);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0001_0010);

    assert_eq!(gpu.vram_pixel(0, 0), 0x7FFF, "fill apos 80h funciona");
}
