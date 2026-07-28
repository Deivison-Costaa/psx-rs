use psx_core::bus::{Bios, Bus, BusRead, Ram};

fn cria_bus() -> Bus {
    let ram = Ram::new();
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS de teste");
    Bus::new(ram, bios)
}

fn gp0() -> u32 {
    0xBF80_1810
}
fn gp1() -> u32 {
    0xBF80_1814
}

fn coord(y: u32, x: u32) -> u32 {
    (y << 16) | x
}

// --- VRAM (sem estado pendente) ---

#[test]
fn vram_inicia_zerada_apos_reset() {
    let mut bus = cria_bus();
    bus.write32::<BusRead>(gp1(), 0x00 << 24);
    assert_eq!(
        bus.gpu_vram_u16(0, 0),
        0x0000,
        "A1: VRAM[0,0] deve ser 0x0000"
    );
    assert_eq!(
        bus.gpu_vram_u16(1023, 511),
        0x0000,
        "A1: VRAM[1023,511] deve ser 0x0000"
    );
}

// --- Fill GP0(02h) ---

#[test]
fn fill_1x1_escreve_pixel_convertido() {
    let mut bus = cria_bus();
    let r: u32 = 0xFF;
    let g: u32 = 0x80;
    let b: u32 = 0x40;
    bus.write32::<BusRead>(gp0(), (0x02 << 24) | (b << 16) | (g << 8) | r);
    bus.write32::<BusRead>(gp0(), coord(0, 0));
    bus.write32::<BusRead>(gp0(), coord(1, 0x10));
    let pixel = bus.gpu_vram_u16(0, 0);
    assert_eq!(pixel & 0x1F, (r >> 3) as u16, "A2: vermelho truncado");
    assert_eq!((pixel >> 5) & 0x1F, (g >> 3) as u16, "A2: verde truncado");
    assert_eq!((pixel >> 10) & 0x1F, (b >> 3) as u16, "A2: azul truncado");
    assert_eq!((pixel >> 15) & 1, 0, "A2: mask bit=0");
}

#[test]
fn fill_retangulo_preenche_area() {
    let mut bus = cria_bus();
    bus.write32::<BusRead>(gp0(), (0x02 << 24) | 0xFF_FF_FF);
    bus.write32::<BusRead>(gp0(), coord(0, 0x10));
    bus.write32::<BusRead>(gp0(), coord(2, 0x30));
    for y in 0..2 {
        for x in 0..3 {
            assert_ne!(
                bus.gpu_vram_u16(16 + x, y),
                0x0000,
                "A3: pixel ({},{}) deve ter sido preenchido",
                16 + x,
                y
            );
        }
    }
}

#[test]
fn fill_color_24bit_para_15bit_mascara_zero() {
    let mut bus = cria_bus();
    bus.write32::<BusRead>(gp0(), (0x02 << 24) | 0x12_34_56);
    bus.write32::<BusRead>(gp0(), coord(0, 0));
    bus.write32::<BusRead>(gp0(), coord(1, 0x10));
    let p = bus.gpu_vram_u16(0, 0);
    assert_eq!(p & 0x1F, (0x56 >> 3) & 0x1F);
    assert_eq!((p >> 5) & 0x1F, (0x34 >> 3) & 0x1F);
    assert_eq!((p >> 10) & 0x1F, (0x12 >> 3) & 0x1F);
    assert_eq!(p >> 15, 0);
}

#[test]
fn fill_xpos_mascarado_para_multiplo_de_0x10() {
    let mut bus = cria_bus();
    bus.write32::<BusRead>(gp0(), (0x02 << 24) | 0xAA_AA_AA);
    bus.write32::<BusRead>(gp0(), coord(0, 0x17));
    bus.write32::<BusRead>(gp0(), coord(1, 0x10));
    assert_ne!(
        bus.gpu_vram_u16(0x10, 0),
        0,
        "A5: Xpos=0x17 mascara para 0x10, pixel em 0x10 preenchido"
    );
    assert_eq!(
        bus.gpu_vram_u16(0x0F, 0),
        0,
        "A5: pixel em 0x0F fora da area (preenchimento comeca em 0x10)"
    );
}

#[test]
fn fill_ypos_mascarado_para_9_bits() {
    let mut bus = cria_bus();
    bus.write32::<BusRead>(gp0(), (0x02 << 24) | 0xFF_FF_FF);
    bus.write32::<BusRead>(gp0(), coord(0x201, 0));
    bus.write32::<BusRead>(gp0(), coord(1, 0x10));
    assert_ne!(
        bus.gpu_vram_u16(0, 1),
        0x0000,
        "A6: Ypos=0x201 mascara para 0x001"
    );
}

#[test]
fn fill_xsiz_zero_nao_preenche() {
    let mut bus = cria_bus();
    bus.write32::<BusRead>(gp0(), (0x02 << 24) | 0xFF_FF_FF);
    bus.write32::<BusRead>(gp0(), coord(0, 0));
    bus.write32::<BusRead>(gp0(), coord(1, 0x0000));
    assert_eq!(
        bus.gpu_vram_u16(0, 0),
        0x0000,
        "A7: Xsiz=0 → sem preenchimento"
    );
}

#[test]
fn fill_ysiz_zero_nao_preenche() {
    let mut bus = cria_bus();
    bus.write32::<BusRead>(gp0(), (0x02 << 24) | 0xFF_FF_FF);
    bus.write32::<BusRead>(gp0(), coord(0, 0));
    bus.write32::<BusRead>(gp0(), coord(0, 0x10));
    assert_eq!(
        bus.gpu_vram_u16(0, 0),
        0x0000,
        "A8: Ysiz=0 → sem preenchimento"
    );
}

#[test]
fn fill_xsiz_arredondado_para_cima() {
    let mut bus = cria_bus();
    bus.write32::<BusRead>(gp0(), (0x02 << 24) | 0xFF_FF_FF);
    bus.write32::<BusRead>(gp0(), coord(0, 0));
    bus.write32::<BusRead>(gp0(), coord(1, 0x01));
    assert_ne!(
        bus.gpu_vram_u16(0, 0),
        0x0000,
        "A9: Xsiz=0x01 arredonda para 0x10"
    );
    assert_ne!(
        bus.gpu_vram_u16(15, 0),
        0x0000,
        "A9: Xsiz=0x01 arredonda para 0x10 (x=15 incluso)"
    );
}

#[test]
fn fill_xsiz_3f1_vira_400() {
    let mut bus = cria_bus();
    bus.write32::<BusRead>(gp0(), (0x02 << 24) | 0xFF_FF_FF);
    bus.write32::<BusRead>(gp0(), coord(0, 0));
    bus.write32::<BusRead>(gp0(), coord(1, 0x3F1));
    assert_ne!(
        bus.gpu_vram_u16(0, 0),
        0x0000,
        "A10: Xsiz=0x3F1 → 0x400 (preenche)"
    );
}

// --- CPU→VRAM GP0(A0h) ---

#[test]
fn cpu_para_vram_escreve_dados() {
    let mut bus = cria_bus();
    bus.write32::<BusRead>(gp0(), 0xA000_0000);
    bus.write32::<BusRead>(gp0(), coord(0, 0));
    bus.write32::<BusRead>(gp0(), coord(1, 1));
    bus.write32::<BusRead>(gp0(), 0xABCD);
    assert_eq!(bus.gpu_vram_u16(0, 0), 0xABCD, "B1: pixel[0,0]=0xABCD");
}

#[test]
fn cpu_para_vram_duas_halfwords_por_word() {
    let mut bus = cria_bus();
    bus.write32::<BusRead>(gp0(), 0xA000_0000);
    bus.write32::<BusRead>(gp0(), coord(0, 0));
    bus.write32::<BusRead>(gp0(), coord(1, 2));
    bus.write32::<BusRead>(gp0(), 0xDEAD_BEEF);
    assert_eq!(
        bus.gpu_vram_u16(0, 0),
        0xBEEF,
        "B2: halfword baixa → primeiro pixel"
    );
    assert_eq!(
        bus.gpu_vram_u16(1, 0),
        0xDEAD,
        "B2: halfword alta → segundo pixel"
    );
}

#[test]
fn cpu_para_vram_xsiz_contado_em_halfwords() {
    let mut bus = cria_bus();
    bus.write32::<BusRead>(gp0(), 0xA000_0000);
    bus.write32::<BusRead>(gp0(), coord(0, 0));
    bus.write32::<BusRead>(gp0(), coord(2, 4));
    for i in 0..8u16 {
        let val = (i as u32 + 1) | ((i as u32 + 2) << 16);
        bus.write32::<BusRead>(gp0(), val);
    }
    let mut ok = 0;
    for y in 0..2 {
        for x in 0..4 {
            if bus.gpu_vram_u16(x, y) != 0 {
                ok += 1;
            }
        }
    }
    assert_eq!(ok, 8, "B3: 4×2=8 halfwords preenchidos");
}

#[test]
fn cpu_para_vram_qtd_impar_halfwords() {
    let mut bus = cria_bus();
    bus.write32::<BusRead>(gp0(), 0xA000_0000);
    bus.write32::<BusRead>(gp0(), coord(0, 0));
    bus.write32::<BusRead>(gp0(), coord(1, 3));
    bus.write32::<BusRead>(gp0(), 0x1122_3344);
    bus.write32::<BusRead>(gp0(), 0x0000_5566);
    assert_eq!(bus.gpu_vram_u16(0, 0), 0x3344, "B4/1");
    assert_eq!(bus.gpu_vram_u16(1, 0), 0x1122, "B4/2");
    assert_eq!(bus.gpu_vram_u16(2, 0), 0x5566, "B4/3");
}

#[test]
fn cpu_para_vram_xpos_ypos_mascarados() {
    let mut bus = cria_bus();
    bus.write32::<BusRead>(gp0(), 0xA000_0000);
    bus.write32::<BusRead>(gp0(), coord(0x201, 0x401));
    bus.write32::<BusRead>(gp0(), coord(1, 1));
    bus.write32::<BusRead>(gp0(), 0xCAFE);
    assert_eq!(
        bus.gpu_vram_u16(1, 1),
        0xCAFE,
        "B5: Xpos=0x401 mascara X para 0x001, Ypos=0x201 mascara Y para 0x001"
    );
}

#[test]
fn cpu_para_vram_xsiz_ysiz_zero_vira_max() {
    let mut bus = cria_bus();
    bus.write32::<BusRead>(gp0(), 0xA000_0000);
    bus.write32::<BusRead>(gp0(), coord(0, 1023));
    bus.write32::<BusRead>(gp0(), coord(0, 0));
    bus.write32::<BusRead>(gp0(), 0xBABE);
    assert_ne!(
        bus.gpu_vram_u16(1023, 0),
        0x0000,
        "B6: Xsiz=0 → max (1 halfword em x=1023)"
    );
}

// --- VRAM→CPU GP0(C0h) ---

#[test]
fn vram_para_cpu_le_dados_do_gpuread() {
    let mut bus = cria_bus();
    bus.write32::<BusRead>(gp0(), (0x02 << 24) | 0xFF_FF_FF);
    bus.write32::<BusRead>(gp0(), coord(10, 4));
    bus.write32::<BusRead>(gp0(), coord(1, 0x10));
    bus.write32::<BusRead>(gp0(), 0xC000_0000);
    bus.write32::<BusRead>(gp0(), coord(10, 4));
    bus.write32::<BusRead>(gp0(), coord(1, 1));
    let data = bus.read32::<BusRead>(gp0());
    assert_ne!(data, 0, "C1: GPUREAD deve retornar dado não-nulo");
}

#[test]
fn vram_para_cpu_le_halfword_esperada() {
    let mut bus = cria_bus();
    bus.write32::<BusRead>(gp0(), 0xA000_0000);
    bus.write32::<BusRead>(gp0(), coord(5, 3));
    bus.write32::<BusRead>(gp0(), coord(1, 1));
    bus.write32::<BusRead>(gp0(), 0xBEEF);
    bus.write32::<BusRead>(gp0(), 0xC000_0000);
    bus.write32::<BusRead>(gp0(), coord(5, 3));
    bus.write32::<BusRead>(gp0(), coord(1, 1));
    let data = bus.read32::<BusRead>(gp0());
    assert_eq!((data & 0xFFFF) as u16, 0xBEEF, "C2: halfword baixa = BEEF");
}

#[test]
fn vram_para_cpu_duas_halfwords_por_word() {
    let mut bus = cria_bus();
    bus.write32::<BusRead>(gp0(), 0xA000_0000);
    bus.write32::<BusRead>(gp0(), coord(0, 0));
    bus.write32::<BusRead>(gp0(), coord(1, 2));
    bus.write32::<BusRead>(gp0(), 0xDEAD_BEEF);
    bus.write32::<BusRead>(gp0(), 0xC000_0000);
    bus.write32::<BusRead>(gp0(), coord(0, 0));
    bus.write32::<BusRead>(gp0(), coord(1, 2));
    let data = bus.read32::<BusRead>(gp0());
    assert_eq!(data, 0xDEAD_BEEF, "C3: duas halfwords lidas em uma word");
}

#[test]
fn vram_para_cpu_qtd_impar_halfwords() {
    let mut bus = cria_bus();
    bus.write32::<BusRead>(gp0(), 0xA000_0000);
    bus.write32::<BusRead>(gp0(), coord(0, 0));
    bus.write32::<BusRead>(gp0(), coord(1, 3));
    bus.write32::<BusRead>(gp0(), 0xAAAA);
    bus.write32::<BusRead>(gp0(), 0xBBBB);
    bus.write32::<BusRead>(gp0(), 0xC000_0000);
    bus.write32::<BusRead>(gp0(), coord(0, 0));
    bus.write32::<BusRead>(gp0(), coord(1, 3));
    let w1 = bus.read32::<BusRead>(gp0());
    let w2 = bus.read32::<BusRead>(gp0());
    assert_eq!(w1 & 0xFFFF, 0xAAAA_u32, "C4/1");
    assert_eq!(w2 & 0xFFFF, 0xBBBB_u32, "C4/2");
    assert_eq!(w2 >> 16, 0, "C4/3: padding zero na halfword extra");
}

#[test]
fn vram_para_cpu_gpuread_retorna_zero_apos_termino() {
    let mut bus = cria_bus();
    bus.write32::<BusRead>(gp0(), (0x02 << 24) | 0xFF_FF_FF);
    bus.write32::<BusRead>(gp0(), coord(0, 0));
    bus.write32::<BusRead>(gp0(), coord(1, 0x10));
    bus.write32::<BusRead>(gp0(), 0xC000_0000);
    bus.write32::<BusRead>(gp0(), coord(0, 0));
    bus.write32::<BusRead>(gp0(), coord(1, 1));
    let _ = bus.read32::<BusRead>(gp0());
    let after = bus.read32::<BusRead>(gp0());
    assert_eq!(after, 0, "C5: GPUREAD=0 após fim da transferência");
}
