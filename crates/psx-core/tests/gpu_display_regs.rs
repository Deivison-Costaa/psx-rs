use psx_core::gpu::Gpu;

fn gp1_reset(gpu: &mut Gpu) {
    gpu.write32(4, 0x00_000000);
}

fn write_gp1(gpu: &mut Gpu, cmd: u8, param: u32) {
    gpu.write32(4, ((cmd as u32) << 24) | (param & 0x00FF_FFFF));
}

fn mount_gp1(cmd: u8, param: u32) -> u32 {
    ((cmd as u32) << 24) | (param & 0x00FF_FFFF)
}

#[rustfmt::skip]
#[test]
fn t1_gp1_00h_reset_configura_display_regs_com_valores_padrao() {
    let mut gpu = Gpu::new();

    write_gp1(&mut gpu, 0x05, 0x12345);
    write_gp1(&mut gpu, 0x06, 0xABC_DEF);
    write_gp1(&mut gpu, 0x07, 0x987_654);

    gp1_reset(&mut gpu);

    assert_eq!(
        gpu.display_start_x(), 0,
        "T1: apos reset, display_start_x deve ser 0, obtido {}",
        gpu.display_start_x(),
    );
    assert_eq!(
        gpu.display_start_y(), 0,
        "T1: apos reset, display_start_y deve ser 0, obtido {}",
        gpu.display_start_y(),
    );
    assert_eq!(
        gpu.display_range_x1(), 0x200,
        "T1: apos reset, display_range_x1 deve ser 0x200, obtido 0x{:X}",
        gpu.display_range_x1(),
    );
    assert_eq!(
        gpu.display_range_x2(), 0xC00,
        "T1: apos reset, display_range_x2 deve ser 0xC00, obtido 0x{:X}",
        gpu.display_range_x2(),
    );
    assert_eq!(
        gpu.display_range_y1(), 0x10,
        "T1: apos reset, display_range_y1 deve ser 0x10, obtido 0x{:X}",
        gpu.display_range_y1(),
    );
    assert_eq!(
        gpu.display_range_y2(), 0x100,
        "T1: apos reset, display_range_y2 deve ser 0x100, obtido 0x{:X}",
        gpu.display_range_y2(),
    );
}

#[rustfmt::skip]
#[test]
fn t2_gp1_05h_define_endereco_de_inicio_do_display() {
    let mut gpu = Gpu::new();

    let x: u32 = 0x2AA;
    let y: u32 = 0x1CC;
    let param = x | (y << 10);
    write_gp1(&mut gpu, 0x05, param);

    assert_eq!(
        gpu.display_start_x(), x as u16,
        "T2: display_start_x deve ser 0x{:X}, obtido 0x{:X}",
        x, gpu.display_start_x(),
    );
    assert_eq!(
        gpu.display_start_y(), y as u16,
        "T2: display_start_y deve ser 0x{:X}, obtido 0x{:X}",
        y, gpu.display_start_y(),
    );
}

#[rustfmt::skip]
#[test]
fn t3_gp1_06h_define_faixa_horizontal_do_display() {
    let mut gpu = Gpu::new();

    let x1: u32 = 0x258;
    let x2: u32 = 0xC38;
    let param = x1 | (x2 << 12);
    write_gp1(&mut gpu, 0x06, param);

    assert_eq!(
        gpu.display_range_x1(), x1 as u16,
        "T3: display_range_x1 deve ser 0x{:X}, obtido 0x{:X}",
        x1, gpu.display_range_x1(),
    );
    assert_eq!(
        gpu.display_range_x2(), x2 as u16,
        "T3: display_range_x2 deve ser 0x{:X}, obtido 0x{:X}",
        x2, gpu.display_range_x2(),
    );
}

#[rustfmt::skip]
#[test]
fn t4_gp1_07h_define_faixa_vertical_do_display() {
    let mut gpu = Gpu::new();

    let y1: u32 = 0x1E;
    let y2: u32 = 0x12E;
    let param = y1 | (y2 << 10);
    write_gp1(&mut gpu, 0x07, param);

    assert_eq!(
        gpu.display_range_y1(), y1 as u16,
        "T4: display_range_y1 deve ser 0x{:X}, obtido 0x{:X}",
        y1, gpu.display_range_y1(),
    );
    assert_eq!(
        gpu.display_range_y2(), y2 as u16,
        "T4: display_range_y2 deve ser 0x{:X}, obtido 0x{:X}",
        y2, gpu.display_range_y2(),
    );
}

#[rustfmt::skip]
#[test]
fn t5_gp1_05h_mascara_x_a_10_bits_e_y_a_9_bits() {
    let mut gpu = Gpu::new();

    let x_com_lixo: u32 = 0x7FF;
    let y_com_lixo: u32 = 0x3FF;
    let param = x_com_lixo | (y_com_lixo << 10);
    write_gp1(&mut gpu, 0x05, param);

    assert_eq!(
        gpu.display_start_x(), 0x3FF,
        "T5: X deve ser mascarado a 10 bits (0x3FF), obtido 0x{:X}",
        gpu.display_start_x(),
    );
    assert_eq!(
        gpu.display_start_y(), 0x1FF,
        "T5: Y deve ser mascarado a 9 bits (0x1FF), obtido 0x{:X}",
        gpu.display_start_y(),
    );
}

#[rustfmt::skip]
#[test]
fn t6_gp1_06h_mascara_x1_e_x2_a_12_bits() {
    let mut gpu = Gpu::new();

    let x1_com_lixo: u32 = 0x1FFF;
    let x2_com_lixo: u32 = 0x1FFF;
    let param = x1_com_lixo | (x2_com_lixo << 12);
    write_gp1(&mut gpu, 0x06, param);

    assert_eq!(
        gpu.display_range_x1(), 0xFFF,
        "T6: X1 deve ser mascarado a 12 bits (0xFFF), obtido 0x{:X}",
        gpu.display_range_x1(),
    );
    assert_eq!(
        gpu.display_range_x2(), 0xFFF,
        "T6: X2 deve ser mascarado a 12 bits (0xFFF), obtido 0x{:X}",
        gpu.display_range_x2(),
    );
}

#[rustfmt::skip]
#[test]
fn t7_gp1_07h_mascara_y1_e_y2_a_10_bits() {
    let mut gpu = Gpu::new();

    let y1_com_lixo: u32 = 0x7FF;
    let y2_com_lixo: u32 = 0x7FF;
    let param = y1_com_lixo | (y2_com_lixo << 10);
    write_gp1(&mut gpu, 0x07, param);

    assert_eq!(
        gpu.display_range_y1(), 0x3FF,
        "T7: Y1 deve ser mascarado a 10 bits (0x3FF), obtido 0x{:X}",
        gpu.display_range_y1(),
    );
    assert_eq!(
        gpu.display_range_y2(), 0x3FF,
        "T7: Y2 deve ser mascarado a 10 bits (0x3FF), obtido 0x{:X}",
        gpu.display_range_y2(),
    );
}
