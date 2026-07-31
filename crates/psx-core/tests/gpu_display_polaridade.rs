use psx_core::gpu::Gpu;

fn write_gp1(gpu: &mut Gpu, cmd: u8, param: u32) {
    gpu.write32(4, ((cmd as u32) << 24) | (param & 0x00FF_FFFF));
}

#[test]
fn reset_deixa_display_desabilitado_e_sem_framebuffer() {
    let mut gpu = Gpu::new();
    write_gp1(&mut gpu, 0x00, 0);
    assert_eq!(
        gpu.stat() & (1 << 23),
        1 << 23,
        "GP1(00h): reset deixa GPUSTAT.23=1 (1=Disabled na spec)"
    );
    assert!(
        gpu.framebuffer_for_display().is_none(),
        "display desabilitado apos reset: sem framebuffer"
    );
}

#[test]
fn gp1_03_param_zero_liga_o_display() {
    let mut gpu = Gpu::new();
    write_gp1(&mut gpu, 0x03, 0);
    assert_eq!(
        gpu.stat() & (1 << 23),
        0,
        "GP1(03h)=0 e display ON: GPUSTAT.23 vai a 0 (0=Enabled)"
    );
    assert!(
        gpu.framebuffer_for_display().is_some(),
        "display ligado: framebuffer_for_display deve devolver Some"
    );
}

#[test]
fn gp1_03_param_um_desliga_o_display() {
    let mut gpu = Gpu::new();
    write_gp1(&mut gpu, 0x03, 0);
    write_gp1(&mut gpu, 0x03, 1);
    assert_eq!(
        gpu.stat() & (1 << 23),
        1 << 23,
        "GP1(03h)=1 e display OFF: GPUSTAT.23 vai a 1 (1=Disabled)"
    );
    assert!(
        gpu.framebuffer_for_display().is_none(),
        "display desligado: framebuffer_for_display deve devolver None"
    );
}

#[test]
fn religar_depois_de_desligar_volta_a_mostrar() {
    let mut gpu = Gpu::new();
    write_gp1(&mut gpu, 0x03, 1);
    write_gp1(&mut gpu, 0x03, 0);
    assert!(
        gpu.framebuffer_for_display().is_some(),
        "OFF depois ON: framebuffer volta"
    );
}
