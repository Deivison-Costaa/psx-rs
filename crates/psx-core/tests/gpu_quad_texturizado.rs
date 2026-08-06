use psx_core::gpu::Gpu;

fn escreve_vram(gpu: &mut Gpu, x: u16, y: u16, val: u16) {
    gpu.vram_raw_mut()[y as usize * 1024 + x as usize] = val;
}

fn clut_em(y: u16) -> u16 {
    (y & 0x1FF) << 6
}

fn triangulo_texturizado(
    gpu: &mut Gpu,
    clut_attr: u16,
    texpage: u32,
    verts: &[((i16, i16), u8, u8); 3],
) {
    gpu.write32(0, 0x2500_0000);
    for (idx, &((sx, sy), u, v)) in verts.iter().enumerate() {
        gpu.write32(0, ((sy as u32 & 0xFFFF) << 16) | (sx as u32 & 0xFFFF));
        let mut uv = ((v as u32) << 8) | (u as u32);
        if idx == 0 {
            uv |= (clut_attr as u32) << 16;
        } else if idx == 1 {
            uv |= (texpage & 0xFF_FFFF) << 16;
        }
        gpu.write32(0, uv);
    }
}

#[test]
fn textura_4bpp_le_a_linha_certa_para_v_maior_que_zero() {
    let mut gpu = Gpu::new();

    escreve_vram(&mut gpu, 0, 0, 0x000A);
    escreve_vram(&mut gpu, 0, 1, 0x4321);
    escreve_vram(&mut gpu, 1, 1, 0x5555);
    escreve_vram(&mut gpu, 0, 2, 0x0006);
    escreve_vram(&mut gpu, 1, 100, 0x1111);
    escreve_vram(&mut gpu, 2, 100, 0x2222);
    escreve_vram(&mut gpu, 3, 100, 0x3333);
    escreve_vram(&mut gpu, 4, 100, 0x4444);
    escreve_vram(&mut gpu, 5, 100, 0x5555);
    escreve_vram(&mut gpu, 6, 100, 0x6666);
    escreve_vram(&mut gpu, 10, 100, 0xAAAA);

    triangulo_texturizado(
        &mut gpu,
        clut_em(100),
        0,
        &[((10, 10), 0, 0), ((16, 10), 6, 0), ((10, 16), 0, 6)],
    );

    assert_eq!(
        gpu.vram_pixel(10, 10),
        0xAAAA,
        "v=0 le a linha 0 da textura — este e o unico caso que os testes antigos cobriam"
    );
    assert_eq!(
        gpu.vram_pixel(10, 11),
        0x1111,
        "v=1 tem de ler a LINHA 1 da textura. `docs/reference/03-gpu.md` L246-252: a coordenada \
         horizontal endereca a VRAM em unidades de 4 bits, e a linha e escolhida separadamente por \
         v. Somar v*256 ao deslocamento horizontal conta a linha DUAS vezes: cada linha anda 64 \
         halfwords para a direita e le lixo. Era o defeito que fazia o logo da BIOS sair como \
         barra chapada (item 2.2c)."
    );
    assert_eq!(
        gpu.vram_pixel(10, 12),
        0x6666,
        "v=2 le a linha 2 da textura"
    );

    assert_eq!(
        [
            gpu.vram_pixel(11, 11),
            gpu.vram_pixel(12, 11),
            gpu.vram_pixel(13, 11),
        ],
        [0x2222, 0x3333, 0x4444],
        "dentro da MESMA linha v=1, os quatro texels de um halfword vem dos quatro nibbles: o \
         deslocamento horizontal e u/4 e o nibble e u%4. Ler u/2 faria u=2 e u=3 cairem no \
         halfword seguinte (que aqui vale 0x5555) em vez de continuarem no primeiro"
    );
}

#[test]
fn textura_8bpp_le_a_linha_certa_para_v_maior_que_zero() {
    let mut gpu = Gpu::new();

    escreve_vram(&mut gpu, 0, 0, 0x0001);
    escreve_vram(&mut gpu, 0, 1, 0x0201);
    escreve_vram(&mut gpu, 1, 1, 0x0403);
    escreve_vram(&mut gpu, 1, 100, 0x4444);
    escreve_vram(&mut gpu, 2, 100, 0x5555);
    escreve_vram(&mut gpu, 3, 100, 0x6666);
    escreve_vram(&mut gpu, 4, 100, 0x7777);

    triangulo_texturizado(
        &mut gpu,
        clut_em(100),
        1 << 7,
        &[((20, 20), 0, 0), ((26, 20), 6, 0), ((20, 26), 0, 6)],
    );

    assert_eq!(gpu.vram_pixel(20, 20), 0x4444, "8bpp, v=0, linha 0");
    assert_eq!(
        gpu.vram_pixel(20, 21),
        0x4444,
        "8bpp com v=1 tem o mesmo defeito de endereco do 4bpp: o deslocamento horizontal e u/2, \
         nao (v*256 + u)/2"
    );
    assert_eq!(
        [
            gpu.vram_pixel(21, 21),
            gpu.vram_pixel(22, 21),
            gpu.vram_pixel(23, 21),
        ],
        [0x5555, 0x6666, 0x7777],
        "em 8bpp cabem DOIS texels por halfword: u/2 escolhe o halfword e u%2 escolhe o byte. \
         Dividir por 4 faria u=2 e u=3 lerem o primeiro halfword de novo"
    );
}

#[test]
fn textura_4bpp_respeita_a_base_horizontal_da_texpage_em_v_maior_que_zero() {
    let mut gpu = Gpu::new();

    escreve_vram(&mut gpu, 64, 0, 0x0001);
    escreve_vram(&mut gpu, 64, 1, 0x0002);
    escreve_vram(&mut gpu, 1, 100, 0x6666);
    escreve_vram(&mut gpu, 2, 100, 0x7777);

    triangulo_texturizado(
        &mut gpu,
        clut_em(100),
        1,
        &[((30, 30), 0, 0), ((36, 30), 6, 0), ((30, 36), 0, 6)],
    );

    assert_eq!(
        gpu.vram_pixel(30, 30),
        0x6666,
        "`docs/reference/03-gpu.md` L494: a base X da texpage anda em passos de 64 halfwords"
    );
    assert_eq!(
        gpu.vram_pixel(30, 31),
        0x7777,
        "a base da texpage e o deslocamento da linha nao podem se misturar: com o defeito, v=1 \
         somava 64 halfwords e caia exatamente sobre a texpage 1, mascarando o erro em quem usa \
         texpage 0"
    );
}

#[test]
fn textura_15bpp_ja_lia_a_linha_certa() {
    let mut gpu = Gpu::new();

    escreve_vram(&mut gpu, 0, 0, 0x7C00);
    escreve_vram(&mut gpu, 0, 1, 0x03E0);

    triangulo_texturizado(
        &mut gpu,
        0,
        2 << 7,
        &[((40, 40), 0, 0), ((46, 40), 6, 0), ((40, 46), 0, 6)],
    );

    assert_eq!(gpu.vram_pixel(40, 40), 0x7C00, "controle: 15bpp em v=0");
    assert_eq!(
        gpu.vram_pixel(40, 41),
        0x03E0,
        "controle: o caminho de 15bpp nunca teve o termo v*256 e ja acertava a linha. Serve para \
         provar que o defeito e do endereco de 4/8bpp, nao da interpolacao de v"
    );
}
