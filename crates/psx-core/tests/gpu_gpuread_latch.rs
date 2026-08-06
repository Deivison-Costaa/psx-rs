use psx_core::gpu::Gpu;

#[test]
fn gpuread_apos_transferencia_concluida_devolve_o_ultimo_valor_nao_zero() {
    let mut gpu = Gpu::new();

    let cmd_a0: u32 = (0xA0u32) << 24;
    gpu.write32(0, cmd_a0);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0001_0002);
    gpu.write32(0, 0xBEEF_CAFE);

    let cmd_c0: u32 = (0xC0u32) << 24;
    gpu.write32(0, cmd_c0);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0001_0002);

    let ultimo = gpu.read32(0);
    assert_eq!(
        ultimo, 0xBEEF_CAFE,
        "transferencia entrega o unico pixel par"
    );

    let depois = gpu.read32(0);
    assert_eq!(
        depois, 0xBEEF_CAFE,
        "spec § VRAM to CPU blitting / GP1(10h) (03-gpu.md L146, L939-940): GPUREAD e um \
         latch — sem transferencia em curso, uma nova leitura devolve o ultimo valor \
         (0xBEEF_CAFE), nao zero"
    );
}

#[test]
fn gpuread_sem_transferencia_nenhuma_comeca_em_zero() {
    let gpu = Gpu::new();

    assert_eq!(
        gpu.read32(0),
        0,
        "um GPU recem-criado, sem nenhuma transferencia C0h jamais feita, comeca com \
         latch zerado — nao ha valor anterior pra devolver"
    );
}

#[test]
fn peek32_do_gpuread_tambem_usa_o_latch_apos_a_transferencia() {
    let mut gpu = Gpu::new();

    let cmd_a0: u32 = (0xA0u32) << 24;
    gpu.write32(0, cmd_a0);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0001_0002);
    gpu.write32(0, 0x1234_5678);

    let cmd_c0: u32 = (0xC0u32) << 24;
    gpu.write32(0, cmd_c0);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0001_0002);
    let _ = gpu.read32(0);

    assert_eq!(
        gpu.peek32(0),
        0x1234_5678,
        "peek32 (usado pela leitura por byte, region_read_byte) tem que ver o mesmo \
         latch que gpuread_word — sem isso um byte lido depois da transferencia veria \
         um valor diferente do que um read32 no mesmo instante veria"
    );
}
