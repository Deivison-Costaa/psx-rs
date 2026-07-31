use psx_core::gpu::Gpu;

fn write_gp1(gpu: &mut Gpu, cmd: u8, param: u32) {
    gpu.write32(4, ((cmd as u32) << 24) | (param & 0x00FF_FFFF));
}

#[rustfmt::skip]
#[test]
fn d1_framebuffer_for_display_retorna_none_quando_display_desabilitado() {
    let mut gpu = Gpu::new();

    write_gp1(&mut gpu, 0x03, 1);

    let result = gpu.framebuffer_for_display();
    assert!(
        result.is_none(),
        "D1: apos GP1(03h)=1 (display DESABILITADO — polaridade corrigida na iter 0112, \
         03-gpu.md L781: param 0=On, 1=Off), framebuffer_for_display deve retornar None"
    );
}

#[rustfmt::skip]
#[test]
fn d2_framebuffer_for_display_retorna_some_quando_display_habilitado() {
    let mut gpu = Gpu::new();

    write_gp1(&mut gpu, 0x03, 0);

    let result = gpu.framebuffer_for_display();
    assert!(
        result.is_some(),
        "D2: apos GP1(03h)=0 (display HABILITADO — polaridade corrigida na iter 0112), \
         framebuffer_for_display deve retornar Some"
    );
}

#[rustfmt::skip]
#[test]
fn d3_framebuffer_para_egui_formato_rgba8() {
    let mut gpu = Gpu::new();

    write_gp1(&mut gpu, 0x03, 0);
    write_gp1(&mut gpu, 0x05, 4 | (2 << 10));
    write_gp1(&mut gpu, 0x07, 1 << 10);

    let vram = gpu.vram_raw_mut();
    vram[(2 & 0x1FF) * 1024 + (4 & 0x3FF)] = 0x7C00;
    vram[(2 & 0x1FF) * 1024 + (5 & 0x3FF)] = 0x03E0;
    vram[(2 & 0x1FF) * 1024 + (6 & 0x3FF)] = 0x001F;

    let fb = gpu.framebuffer_for_display().expect("display deve estar habilitado");

    assert!(fb.width > 0, "D3: largura deve ser > 0, obtido {}", fb.width);
    assert!(fb.height > 0, "D3: altura deve ser > 0, obtido {}", fb.height);
    assert_eq!(
        fb.data.len(),
        (fb.width as usize) * (fb.height as usize) * 4,
        "D3: dados devem ter width*height*4 bytes"
    );

    let px = |x: usize, y: usize| {
        let offset = (y * fb.width as usize + x) * 4;
        (fb.data[offset], fb.data[offset + 1], fb.data[offset + 2], fb.data[offset + 3])
    };

    assert_eq!(
        px(0, 0),
        (0x00, 0x00, 0xF8, 255),
        "D3: pixel (0,0) 0x7C00=azul deve ser (0,0,0xF8,255), obtido {:?}",
        px(0, 0),
    );
    assert_eq!(
        px(1, 0),
        (0x00, 0xF8, 0x00, 255),
        "D3: pixel (1,0) 0x03E0=verde deve ser (0,0xF8,0,255), obtido {:?}",
        px(1, 0),
    );
    assert_eq!(
        px(2, 0),
        (0xF8, 0x00, 0x00, 255),
        "D3: pixel (2,0) 0x001F=vermelho deve ser (0xF8,0,0,255), obtido {:?}",
        px(2, 0),
    );
}
