use psx_core::gpu::Gpu;

fn gp1_reset(gpu: &mut Gpu) {
    gpu.write32(4, 0x00_000000);
}

fn write_gp1(gpu: &mut Gpu, cmd: u8, param: u32) {
    gpu.write32(4, ((cmd as u32) << 24) | (param & 0x00FF_FFFF));
}

fn bit31(gpu: &Gpu) -> bool {
    (gpu.read32(4) >> 31) & 1 != 0
}

fn bit20(gpu: &Gpu) -> bool {
    (gpu.read32(4) >> 20) & 1 != 0
}

#[rustfmt::skip]
#[test]
fn t1_padrao_video_mode_ntsc() {
    let gpu = Gpu::new();

    assert!(
        !gpu.video_mode(),
        "T1: video_mode deve ser false (NTSC) apos reset"
    );
    assert!(
        !bit20(&gpu),
        "T1: GPUSTAT bit20 deve ser 0 (NTSC) apos reset"
    );
}

#[rustfmt::skip]
#[test]
fn t2_gp1_08h_bit3_pal() {
    let mut gpu = Gpu::new();

    write_gp1(&mut gpu, 0x08, 0x08);

    assert!(
        gpu.video_mode(),
        "T2: video_mode deve ser true (PAL) apos GP1(08h) bit3=1"
    );
    assert!(
        bit20(&gpu),
        "T2: GPUSTAT bit20 deve ser 1 apos GP1(08h) bit3=1"
    );
}

#[rustfmt::skip]
#[test]
fn t3_gp1_08h_bit3_ntsc() {
    let mut gpu = Gpu::new();

    write_gp1(&mut gpu, 0x08, 0x08);
    write_gp1(&mut gpu, 0x08, 0x00);

    assert!(
        !gpu.video_mode(),
        "T3: video_mode deve voltar para false (NTSC) apos GP1(08h) bit3=0"
    );
    assert!(
        !bit20(&gpu),
        "T3: GPUSTAT bit20 deve ser 0 apos GP1(08h) bit3=0"
    );
}

#[rustfmt::skip]
#[test]
fn t4_frame_cycles_ntsc_valor_esperado() {
    let gpu = Gpu::new();

    assert_eq!(
        gpu.frame_cycles(), 566_187,
        "T4: frame_cycles deve ser 566_187 (NTSC ~59.826Hz a 33.8688MHz)"
    );
}

#[rustfmt::skip]
#[test]
fn t5_frame_cycles_pal_valor_esperado() {
    let mut gpu = Gpu::new();
    write_gp1(&mut gpu, 0x08, 0x08);

    assert_eq!(
        gpu.frame_cycles(), 680_659,
        "T5: frame_cycles deve ser 680_659 (PAL ~49.761Hz a 33.8688MHz)"
    );
}

#[rustfmt::skip]
#[test]
fn t6_bit31_reflete_odd_line_quando_nao_esta_em_vblank() {
    let mut gpu = Gpu::new();

    assert!(
        !bit31(&gpu),
        "T6: bit31 deve ser 0 inicialmente"
    );

    gpu.set_odd_line(true);
    assert!(
        bit31(&gpu),
        "T6: bit31 deve ser 1 apos set_odd_line(true)"
    );

    gpu.set_odd_line(false);
    assert!(
        !bit31(&gpu),
        "T6: bit31 deve ser 0 apos set_odd_line(false)"
    );
}

#[rustfmt::skip]
#[test]
fn t7_enter_vblank_zera_bit31_mesmo_com_odd_line_true() {
    let mut gpu = Gpu::new();
    gpu.set_odd_line(true);

    assert!(
        bit31(&gpu),
        "T7: bit31 deve ser 1 antes de vblank"
    );

    gpu.enter_vblank();

    assert!(
        !bit31(&gpu),
        "T7: bit31 deve ser 0 durante vblank, mesmo com odd_line=true"
    );
    assert!(
        gpu.in_vblank(),
        "T7: in_vblank deve ser true"
    );
}

#[rustfmt::skip]
#[test]
fn t8_exit_vblank_restaura_bit31() {
    let mut gpu = Gpu::new();
    gpu.set_odd_line(true);
    gpu.enter_vblank();

    assert!(
        !bit31(&gpu),
        "T8: bit31 deve ser 0 durante vblank (precondicao)"
    );

    gpu.exit_vblank();

    assert!(
        bit31(&gpu),
        "T8: bit31 deve ser 1 apos sair de vblank (restaurou odd_line)"
    );
    assert!(
        !gpu.in_vblank(),
        "T8: in_vblank deve ser false"
    );
}

#[rustfmt::skip]
#[test]
fn t9_reset_gp1_00h_volta_para_ntsc() {
    let mut gpu = Gpu::new();
    write_gp1(&mut gpu, 0x08, 0x08);
    write_gp1(&mut gpu, 0x08, 0x0F);

    assert!(
        gpu.video_mode(),
        "T9: video_mode deve ser PAL antes do reset"
    );

    gp1_reset(&mut gpu);

    assert!(
        !gpu.video_mode(),
        "T9: video_mode deve ser false (NTSC) apos reset"
    );
    assert_eq!(
        gpu.frame_cycles(), 566_187,
        "T9: frame_cycles deve ser NTSC apos reset"
    );
}
