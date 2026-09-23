use psx_core::gpu::Gpu;

fn info(gpu: &mut Gpu, indice: u32) -> u32 {
    gpu.write32(4, 0x1000_0000 | indice);
    gpu.read32(0)
}

#[test]
fn indice_7_devolve_a_versao_2() {
    let mut gpu = Gpu::new();
    assert_eq!(
        info(&mut gpu, 7),
        2,
        "`docs/reference/03-gpu.md` GP1(10h): indice 07h = versao da GPU; o core emula a v2 \
         (flip de E1 bits 12/13, modulacao em 8 bits), que responde 00000002h"
    );
}

#[test]
fn registros_de_desenho_sao_lidos_de_volta() {
    let mut gpu = Gpu::new();
    gpu.write32(0, 0xE200_0000 | (3 << 15) | (5 << 10) | (7 << 5) | 9);
    gpu.write32(0, 0xE300_0000 | (20 << 10) | 10);
    gpu.write32(0, 0xE400_0000 | (239 << 10) | 319);
    gpu.write32(0, 0xE500_0000 | (0x7FF << 11) | 0x400);

    assert_eq!(
        info(&mut gpu, 2),
        (3 << 15) | (5 << 10) | (7 << 5) | 9,
        "indice 02h = GP0(E2h)"
    );
    assert_eq!(info(&mut gpu, 3), (20 << 10) | 10, "indice 03h = GP0(E3h)");
    assert_eq!(
        info(&mut gpu, 4),
        (239 << 10) | 319,
        "indice 04h = GP0(E4h)"
    );
    assert_eq!(
        info(&mut gpu, 5),
        (0x7FF << 11) | 0x400,
        "indice 05h = GP0(E5h), 22 bits com os sinais de X e Y"
    );
    assert_eq!(info(&mut gpu, 8), 0, "indice 08h = 00000000h na v2");
}

#[test]
fn indices_sem_registro_mantem_o_latch() {
    let mut gpu = Gpu::new();
    assert_eq!(info(&mut gpu, 7), 2);
    for indice in [0, 1, 6, 9, 0xF] {
        assert_eq!(
            info(&mut gpu, indice),
            2,
            "indice {indice:X}h: 'Returns Nothing (old value in GPUREAD remains unchanged)'"
        );
    }
}

#[test]
fn indices_e_comandos_espelhados() {
    let mut gpu = Gpu::new();
    assert_eq!(info(&mut gpu, 0x17), 2, "10h-FFFFFFh espelham 00h..0Fh");
    gpu.write32(4, 0x1F00_0008);
    assert_eq!(gpu.read32(0), 0, "GP1(11h..1Fh) espelham GP1(10h)");
    gpu.write32(4, 0x5000_0007);
    assert_eq!(gpu.read32(0), 2, "GP1(40h..FFh) espelham GP1(00h..3Fh)");
}
