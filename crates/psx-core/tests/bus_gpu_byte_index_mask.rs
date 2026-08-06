use psx_core::bus::{Bios, Bus, BusRead, Ram};

#[test]
fn gpu_read16_dobra_indice_de_byte_no_limite_da_palavra() {
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS de teste");
    let mut bus = Bus::new(Ram::new(), bios);

    let gp1_addr = 0xBF80_1814;
    bus.write32::<BusRead>(gp1_addr, 0x00 << 24);
    assert_eq!(
        bus.read32::<BusRead>(gp1_addr),
        0x1480_2000,
        "GPUSTAT de teste deve ter valor conhecido"
    );

    assert_eq!(
        bus.read16::<BusRead>(0x1F80_1817),
        0x0014,
        "leitura em GP1+3 deve dobrar para o byte 0 no segundo acesso"
    );
}
