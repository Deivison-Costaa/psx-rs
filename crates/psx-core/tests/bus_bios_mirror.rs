use psx_core::bus::{Bios, Bus, BusRead, Ram};

fn bios_com_padrao() -> Bios {
    let mut data = vec![0u8; 0x80000];
    for i in 0..data.len() {
        data[i] = (i & 0xFF) as u8;
    }
    Bios::from_bytes(data).expect("BIOS sintetica de 512KB")
}

#[test]
fn mirror_nao_afeta_enderecos_acima_de_512kb() {
    let bios = bios_com_padrao();
    let ram = Ram::new();
    let bus = Bus::new(ram, bios);

    let val = bus.read32::<BusRead>(0x0008_0000);
    assert_eq!(val, 0, "enderecos >= 512KB devem ler da RAM (zeros), nao da BIOS");
}

#[test]
fn mirror_ativo_leitura_kuseg_retorna_bios() {
    let bios = bios_com_padrao();
    let ram = Ram::new();
    let bus = Bus::new(ram, bios);

    let val = bus.read32::<BusRead>(0x0000_0C80);
    let esperado = u32::from_le_bytes([0x80, 0x81, 0x82, 0x83]);
    assert_eq!(val, esperado, "KUSEG deve ler da BIOS com mirror ativo");
}

#[test]
fn mirror_ativo_leitura_kseg0_retorna_bios() {
    let bios = bios_com_padrao();
    let ram = Ram::new();
    let bus = Bus::new(ram, bios);

    let val = bus.read32::<BusRead>(0x8000_0C80);
    let esperado = u32::from_le_bytes([0x80, 0x81, 0x82, 0x83]);
    assert_eq!(val, esperado, "KSEG0 deve ler da BIOS com mirror ativo");
}

#[test]
fn mirror_ativo_leitura_kseg1_retorna_ram() {
    let bios = bios_com_padrao();
    let ram = Ram::new();
    let bus = Bus::new(ram, bios);

    let val = bus.read32::<BusRead>(0xA000_0C80);
    assert_eq!(val, 0, "KSEG1 deve ler da RAM (zeros) com mirror ativo");
}

#[test]
fn mirror_desativado_por_write_na_exp1_base() {
    let bios = bios_com_padrao();
    let ram = Ram::new();
    let mut bus = Bus::new(ram, bios);

    bus.write32::<BusRead>(0x1F80_1000, 0x0013243F);

    let val = bus.read32::<BusRead>(0x0000_0C80);
    assert_eq!(val, 0, "KUSEG deve ler da RAM apos mirror desativado");
}

#[test]
fn mirror_nao_desativado_por_outros_registradores() {
    let bios = bios_com_padrao();
    let ram = Ram::new();
    let mut bus = Bus::new(ram, bios);

    bus.write32::<BusRead>(0x1F80_1010, 0x0013243F);

    let val = bus.read32::<BusRead>(0x0000_0C80);
    let esperado = u32::from_le_bytes([0x80, 0x81, 0x82, 0x83]);
    assert_eq!(
        val, esperado,
        "mirror nao deve ser desativado por write a EXP2 (0x1F801010)"
    );
}
