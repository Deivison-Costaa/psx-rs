use psx_core::bus::{Bios, BiosError};

const BIOS_OK_SIZE: usize = 0x80000;

fn make_ok_bios() -> Vec<u8> {
    let mut data = vec![0u8; BIOS_OK_SIZE];
    data[0x0000] = 0x3C;
    data[0x0001] = 0x1F;
    data[0x0004] = 0xAC;
    data[0x0005] = 0x81;
    data[0x0008] = 0x34;
    data[0x0009] = 0x21;
    data[0x000C] = 0x00;
    data[0x000D] = 0x80;

    data[0x0100] = 0xDE;
    data[0x0101] = 0xAD;
    data[0x0102] = 0xBE;
    data[0x0103] = 0xEF;

    data[0x7FFFC] = 0x11;
    data[0x7FFFD] = 0x22;
    data[0x7FFFE] = 0x33;
    data[0x7FFFF] = 0x44;
    data
}

#[test]
fn bios_from_bytes_ok() {
    let data = make_ok_bios();
    let bios = Bios::from_bytes(data.clone()).expect("512 KiB exatos devem funcionar");
    assert_eq!(bios.size(), BIOS_OK_SIZE);
}

#[test]
fn bios_from_bytes_muito_curto() {
    let curto = vec![0u8; 0x40000];
    let err = Bios::from_bytes(curto).unwrap_err();
    assert!(matches!(
        err,
        BiosError::WrongSize {
            got: 0x40000,
            expected: BIOS_OK_SIZE
        }
    ));
}

#[test]
fn bios_from_bytes_vazio() {
    let vazio = vec![];
    let err = Bios::from_bytes(vazio).unwrap_err();
    assert!(matches!(err, BiosError::WrongSize { .. }));
}

#[test]
fn bios_from_bytes_muito_longo() {
    let longo = vec![0u8; 0x100000];
    let err = Bios::from_bytes(longo).unwrap_err();
    assert!(matches!(
        err,
        BiosError::WrongSize {
            got: 0x100000,
            expected: BIOS_OK_SIZE
        }
    ));
}

#[test]
fn bios_read32_little_endian() {
    let data = make_ok_bios();
    let bios = Bios::from_bytes(data).unwrap();

    let word = bios.read32(0x0100);
    assert_eq!(word, 0xEFBEADDE);
}

#[test]
fn bios_read32_ultimo_word() {
    let data = make_ok_bios();
    let bios = Bios::from_bytes(data).unwrap();
    let word = bios.read32(0x7FFFC);
    assert_eq!(word, 0x44332211);
}

#[test]
fn bios_read32_primeiro_word() {
    let data = make_ok_bios();
    let data_clone = data.clone();
    let bios = Bios::from_bytes(data).unwrap();
    let word = bios.read32(0x0000);
    let expected = u32::from_le_bytes([
        data_clone[0x0000],
        data_clone[0x0001],
        data_clone[0x0002],
        data_clone[0x0003],
    ]);
    assert_eq!(word, expected);
}

#[test]
fn bios_read32_offset_dentro_limite() {
    let data = make_ok_bios();
    let bios = Bios::from_bytes(data).unwrap();
    let _ = bios.read32(BIOS_OK_SIZE - 4);
}
