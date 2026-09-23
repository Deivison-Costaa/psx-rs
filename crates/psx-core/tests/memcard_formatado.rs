use psx_core::app::saves;
use psx_core::bus::{Bios, Bus, Ram};
use psx_core::cpu::Cpu;
use psx_core::memcard::{self, CARD_BYTES, FRAME_BYTES, MemoryCard};

const CROSS: u16 = 1 << 14;
const APERTA_NO_MENU: usize = 155_000_000;
const SOLTA: usize = APERTA_NO_MENU + 3_000_000;
const ATE_O_GERENCIADOR: usize = 180_000_000;

fn bios() -> Option<Bios> {
    ["bios/SCPH1001.BIN", "../../bios/SCPH1001.BIN"]
        .iter()
        .find_map(|c| std::fs::read(c).ok())
        .and_then(|b| Bios::from_bytes(b).ok())
}

fn xor(bytes: &[u8]) -> u8 {
    bytes.iter().fold(0, |acc, b| acc ^ b)
}

fn quadro(imagem: &[u8], indice: usize) -> &[u8] {
    &imagem[indice * FRAME_BYTES..(indice + 1) * FRAME_BYTES]
}

#[test]
fn imagem_formatada_segue_o_leiaute_do_bloco_0() {
    let imagem = memcard::formatted_image();
    assert_eq!(imagem.len(), CARD_BYTES);
    assert_eq!(&quadro(&imagem, 0)[..2], b"MC");
    assert_eq!(quadro(&imagem, 0)[127], 0x0E, "checksum usual do cabecalho");
    for indice in 1..=15 {
        let entrada = quadro(&imagem, indice);
        assert_eq!(&entrada[..4], &[0xA0, 0, 0, 0], "quadro {indice}: livre");
        assert_eq!(
            &entrada[8..10],
            &[0xFF, 0xFF],
            "quadro {indice}: sem proximo"
        );
    }
    for indice in 16..=35 {
        assert_eq!(&quadro(&imagem, indice)[..4], &[0xFF; 4], "quadro {indice}");
    }
    for indice in 0..=35 {
        assert_eq!(
            xor(quadro(&imagem, indice)),
            0,
            "checksum do quadro {indice}"
        );
    }
    assert!(memcard::is_formatted(&imagem));
    assert!(!memcard::is_formatted(&vec![0u8; CARD_BYTES]));
    assert!(saves::lista(&imagem).is_empty());
    assert_eq!(MemoryCard::formatted().data(), &imagem[..]);
}

/// Liga a BIOS sem disco e entra no MEMORY CARD do shell. Diante de um cartao que nao
/// reconhece, o shell o formata sozinho; diante de um reconhecido, so le.
fn entra_no_gerenciador(bios: Bios, cartao: &[u8]) -> (Vec<u8>, bool) {
    let mut bus = Bus::new(Ram::new(), bios);
    let mut cpu = Cpu::new();
    bus.sio_mut().connect_digital_pad(true);
    bus.sio_mut()
        .load_memory_card(cartao)
        .expect("imagem de 128 KiB");
    for passo in 0..ATE_O_GERENCIADOR {
        match passo {
            APERTA_NO_MENU => bus.sio().set_buttons(!CROSS),
            SOLTA => bus.sio().set_buttons(0xFFFF),
            _ => {}
        }
        cpu.step(&mut bus);
    }
    let gravou = bus.sio().memory_card_dirty();
    (bus.sio().memory_card_image(), gravou)
}

#[test]
fn shell_da_bios_formata_um_cartao_zerado_igual_ao_nosso_formatador() {
    let Some(bios) = bios() else {
        eprintln!("SKIP: sem bios/SCPH1001.BIN");
        return;
    };
    let (imagem, gravou) = entra_no_gerenciador(bios, &vec![0u8; CARD_BYTES]);
    assert!(gravou, "cartao zerado nao e reconhecido: o shell formata");
    assert_eq!(imagem, memcard::formatted_image());
}

/// Um save de 1 bloco no diretorio 1: se o shell reformatasse, a entrada voltaria a A0h.
fn com_um_save(mut imagem: Vec<u8>) -> Vec<u8> {
    let mut entrada = [0u8; FRAME_BYTES];
    entrada[0] = 0x51;
    entrada[4..8].copy_from_slice(&0x2000u32.to_le_bytes());
    entrada[8..10].copy_from_slice(&[0xFF, 0xFF]);
    entrada[0x0A..0x0A + 17].copy_from_slice(b"BASLUS-00000TESTE");
    entrada[127] = xor(&entrada[..127]);
    imagem[FRAME_BYTES..2 * FRAME_BYTES].copy_from_slice(&entrada);
    let titulo = 8192;
    imagem[titulo..titulo + 4].copy_from_slice(&[b'S', b'C', 0x11, 1]);
    imagem[titulo + 4..titulo + 9].copy_from_slice(b"TESTE");
    imagem
}

#[test]
fn shell_da_bios_reconhece_o_cartao_formatado_e_mantem_o_save() {
    let Some(bios) = bios() else {
        eprintln!("SKIP: sem bios/SCPH1001.BIN");
        return;
    };
    let cartao = com_um_save(memcard::formatted_image());
    let (imagem, _) = entra_no_gerenciador(bios, &cartao);
    let mudados: Vec<usize> = (0..CARD_BYTES / FRAME_BYTES)
        .filter(|&q| quadro(&imagem, q) != quadro(&cartao, q))
        .collect();
    assert!(
        mudados.iter().all(|&q| q == 63),
        "so o quadro de teste de escrita (63) pode mudar; mudaram {mudados:?}"
    );
    let saves = saves::lista(&imagem);
    assert_eq!(saves.len(), 1);
    assert_eq!(saves[0].nome, "BASLUS-00000TESTE");
}
