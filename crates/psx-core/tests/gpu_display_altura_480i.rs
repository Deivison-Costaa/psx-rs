use psx_core::gpu::Gpu;

fn write_gp1(gpu: &mut Gpu, cmd: u8, param: u32) {
    gpu.write32(4, ((cmd as u32) << 24) | (param & 0x00FF_FFFF));
}

fn gpu_display_ligado_range_240() -> Gpu {
    let mut gpu = Gpu::new();
    write_gp1(&mut gpu, 0x03, 0);
    write_gp1(&mut gpu, 0x05, 0);
    write_gp1(&mut gpu, 0x07, 0x10 | (0x100 << 10));
    gpu
}

#[test]
fn altura_240p_continua_y2_menos_y1() {
    let mut gpu = gpu_display_ligado_range_240();
    write_gp1(&mut gpu, 0x08, 0x03);
    let fb = gpu.framebuffer_for_display().expect("display ligado");
    assert_eq!(fb.height, 240, "240p: altura e Y2-Y1 = 0x100-0x10 = 240");
}

#[test]
fn altura_480i_dobra_o_range_vertical() {
    let mut gpu = gpu_display_ligado_range_240();
    write_gp1(&mut gpu, 0x08, 0x27);
    let fb = gpu.framebuffer_for_display().expect("display ligado");
    assert_eq!(
        fb.height, 480,
        "480i (vres=1 e interlace=1): altura e (Y2-Y1)*2 = 480"
    );
}

#[test]
fn vres_480_sem_interlace_nao_dobra() {
    let mut gpu = gpu_display_ligado_range_240();
    write_gp1(&mut gpu, 0x08, 0x07);
    let fb = gpu.framebuffer_for_display().expect("display ligado");
    assert_eq!(
        fb.height, 240,
        "vres=1 so vale com bit5=1 (interlace); sem ele a altura fica em Y2-Y1"
    );
}

#[test]
fn interlace_sem_vres_480_nao_dobra() {
    let mut gpu = gpu_display_ligado_range_240();
    write_gp1(&mut gpu, 0x08, 0x23);
    let fb = gpu.framebuffer_for_display().expect("display ligado");
    assert_eq!(
        fb.height, 240,
        "interlace ligado com vres=240: altura segue Y2-Y1"
    );
}

#[test]
fn framebuffer_480i_le_linhas_consecutivas_da_vram() {
    let mut gpu = gpu_display_ligado_range_240();
    write_gp1(&mut gpu, 0x08, 0x27);
    {
        let vram = gpu.vram_raw_mut();
        vram[0] = 0x001F;
        vram[1024] = 0x03E0;
        vram[479 * 1024] = 0x7C00;
    }
    let fb = gpu.framebuffer_for_display().expect("display ligado");
    let px = |x: usize, y: usize| {
        let o = (y * fb.width as usize + x) * 4;
        (fb.data[o], fb.data[o + 1], fb.data[o + 2])
    };
    assert_eq!(
        px(0, 0),
        (0xF8, 0, 0),
        "linha 0 do fb vem da linha 0 da VRAM"
    );
    assert_eq!(
        px(0, 1),
        (0, 0xF8, 0),
        "linha 1 do fb vem da linha 1 da VRAM"
    );
    assert_eq!(
        px(0, 479),
        (0, 0, 0xF8),
        "linha 479 do fb vem da linha 479 da VRAM"
    );
}
