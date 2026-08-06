use psx_core::gpu::{Framebuffer, Gpu};

fn desenha_rect_1x1(gpu: &mut Gpu, x: u16, y: u16, cor: u32) {
    let cmd: u32 = (0x68u32 << 24) | (cor & 0x00FF_FFFF);
    gpu.write32(0, cmd);
    gpu.write32(0, ((y as u32) << 16) | (x as u32));
}

fn pixel_rgb(fb: &Framebuffer, x: u16, y: u16) -> (u8, u8, u8) {
    let idx = (y as usize * fb.width as usize + x as usize) * 4;
    (fb.data[idx], fb.data[idx + 1], fb.data[idx + 2])
}

const VERMELHO: u32 = 0x0000FF;
const AZUL: u32 = 0xFF0000;

#[test]
fn framebuffer_so_muda_no_proximo_vblank_nao_a_cada_escrita_em_vram() {
    let mut gpu = Gpu::new();
    gpu.write32(4, 0x0300_0000);

    desenha_rect_1x1(&mut gpu, 10, 10, VERMELHO);
    gpu.enter_vblank();
    let fb1 = gpu.framebuffer_for_display().expect("display habilitado por padrao");
    assert_eq!(
        pixel_rgb(&fb1, 10, 10),
        (0xF8, 0, 0),
        "apos o 1o vblank, o framebuffer de exibicao reflete o quadro ja completo (vermelho)"
    );

    desenha_rect_1x1(&mut gpu, 10, 10, AZUL);
    let fb_meio_do_quadro = gpu.framebuffer_for_display().expect("display habilitado");
    assert_eq!(
        pixel_rgb(&fb_meio_do_quadro, 10, 10),
        (0xF8, 0, 0),
        "spec § GPU Display Control Commands / Video Timings (03-gpu.md): o hardware so \
         atualiza a imagem exibida no proximo vblank — escrever em VRAM no meio do quadro \
         (antes do vblank seguinte) nao pode vazar pro framebuffer que a app desktop le, \
         senao um jogo sem double-buffer mostraria poligonos pela metade (achado 10.42)"
    );

    gpu.enter_vblank();
    let fb2 = gpu.framebuffer_for_display().expect("display habilitado");
    assert_eq!(
        pixel_rgb(&fb2, 10, 10),
        (0, 0, 0xF8),
        "apos o 2o vblank, o framebuffer de exibicao ja reflete o quadro azul completo"
    );
}
