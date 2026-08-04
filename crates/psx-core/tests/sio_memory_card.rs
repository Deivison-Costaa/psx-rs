use psx_core::memcard::{CARD_BYTES, FRAME_BYTES, MemoryCard};
use psx_core::sio::Sio;

const JOY_DATA: u32 = 0x1F80_1040;
const JOY_CTRL: u32 = 0x1F80_104A;

/// Troca uma sequencia inteira e devolve as respostas, uma por byte enviado.
fn troca(cartao: &mut MemoryCard, enviados: &[u8]) -> Vec<u8> {
    cartao.begin();
    enviados.iter().map(|b| cartao.exchange(*b).0).collect()
}

fn le_setor(cartao: &mut MemoryCard, setor: u16) -> Vec<u8> {
    let mut envio = vec![0x81, 0x52, 0x00, 0x00, (setor >> 8) as u8, setor as u8];
    envio.extend(std::iter::repeat_n(0u8, 4 + FRAME_BYTES + 2));
    troca(cartao, &envio)
}

fn escreve_setor(cartao: &mut MemoryCard, setor: u16, dados: &[u8], checksum: u8) -> Vec<u8> {
    let mut envio = vec![0x81, 0x57, 0x00, 0x00, (setor >> 8) as u8, setor as u8];
    envio.extend_from_slice(dados);
    envio.push(checksum);
    envio.extend([0u8; 3]);
    troca(cartao, &envio)
}

fn checksum(setor: u16, dados: &[u8]) -> u8 {
    dados
        .iter()
        .fold((setor >> 8) as u8 ^ setor as u8, |acc, b| acc ^ b)
}

#[test]
fn imagem_crua_de_128_kib_e_aceita_e_devolvida_igual() {
    let mut bruto = vec![0u8; CARD_BYTES];
    bruto[0] = b'M';
    bruto[1] = b'C';
    bruto[CARD_BYTES - 1] = 0x5A;
    let cartao = MemoryCard::from_bytes(&bruto).expect("imagem de 128 KiB");
    assert_eq!(cartao.data().len(), CARD_BYTES);
    assert_eq!(&cartao.data()[..2], b"MC");
    assert_eq!(cartao.data()[CARD_BYTES - 1], 0x5A);
    assert!(
        MemoryCard::from_bytes(&[0u8; 16]).is_err(),
        "tamanho errado"
    );
}

#[test]
fn cartao_vazio_tem_1024_quadros_de_128_bytes() {
    let cartao = MemoryCard::new();
    assert_eq!(cartao.data().len(), 1024 * FRAME_BYTES);
    assert_eq!(
        cartao.flag(),
        0x08,
        "§ FLAG Byte (L2842): valor inicial 08h"
    );
}

#[test]
fn leitura_segue_a_sequencia_de_bytes_da_spec() {
    let mut bruto = vec![0u8; CARD_BYTES];
    for (i, b) in bruto[0x80..0x100].iter_mut().enumerate() {
        *b = (i as u8).wrapping_mul(3).wrapping_add(7);
    }
    let mut cartao = MemoryCard::from_bytes(&bruto).unwrap();
    let r = le_setor(&mut cartao, 1);

    assert_eq!(r[1], 0x08, "resposta ao comando e o FLAG");
    assert_eq!(&r[2..4], &[0x5A, 0x5D], "ID1 e ID2");
    assert_eq!(r[4], 0x00, "resposta ao MSB do endereco");
    assert_eq!(r[5], 0x00, "resposta ao LSB e o byte anterior (MSB = 0)");
    assert_eq!(&r[6..8], &[0x5C, 0x5D], "acknowledge do comando");
    assert_eq!(&r[8..10], &[0x00, 0x01], "endereco confirmado");
    let dados: Vec<u8> = r[10..10 + FRAME_BYTES].to_vec();
    assert_eq!(dados, bruto[0x80..0x100].to_vec());
    assert_eq!(
        r[10 + FRAME_BYTES],
        checksum(1, &dados),
        "CHK = MSB xor LSB xor todos os bytes de dados"
    );
    assert_eq!(r[11 + FRAME_BYTES], 0x47, "byte de fim 47h = Good");
}

#[test]
fn ack_cai_no_ultimo_byte_da_leitura() {
    let mut cartao = MemoryCard::new();
    cartao.begin();
    let mut envio = vec![0x81u8, 0x52, 0x00, 0x00, 0x00, 0x00];
    envio.extend(std::iter::repeat_n(0u8, 4 + FRAME_BYTES + 2));
    let acks: Vec<bool> = envio.iter().map(|b| cartao.exchange(*b).1).collect();
    let n = acks.len();
    assert!(
        acks[..n - 1].iter().all(|a| *a),
        "todo byte menos o ultimo pede /ACK"
    );
    assert!(!acks[n - 1], "o ultimo byte nao pede /ACK");
}

#[test]
fn escrita_grava_o_setor_e_devolve_47h() {
    let mut cartao = MemoryCard::new();
    let dados: Vec<u8> = (0..FRAME_BYTES).map(|i| (i as u8) ^ 0xA5).collect();
    let r = escreve_setor(&mut cartao, 3, &dados, checksum(3, &dados));
    assert_eq!(&r[2..4], &[0x5A, 0x5D]);
    assert_eq!(&r[r.len() - 3..], &[0x5C, 0x5D, 0x47], "5Ch 5Dh 47h no fim");
    assert_eq!(&cartao.data()[3 * FRAME_BYTES..4 * FRAME_BYTES], &dados[..]);
    assert!(
        cartao.take_dirty(),
        "a imagem precisa ser regravada em disco"
    );
    assert!(!cartao.take_dirty(), "e a marca e consumida uma vez so");
}

#[test]
fn escrita_com_checksum_errado_devolve_4eh_e_nao_grava() {
    let mut cartao = MemoryCard::new();
    let dados = vec![0x5Au8; FRAME_BYTES];
    let r = escreve_setor(&mut cartao, 3, &dados, checksum(3, &dados) ^ 0xFF);
    assert_eq!(*r.last().unwrap(), 0x4E, "4Eh = BadChecksum");
    assert!(
        cartao.data()[3 * FRAME_BYTES..4 * FRAME_BYTES]
            .iter()
            .all(|b| *b == 0)
    );
    assert!(!cartao.take_dirty());
}

#[test]
fn setor_acima_de_3ffh_devolve_ffffh_e_aborta() {
    let mut cartao = MemoryCard::new();
    let r = le_setor(&mut cartao, 0x0400);
    assert_eq!(&r[8..10], &[0xFF, 0xFF], "endereco confirmado invalido");
    assert!(
        r[10..].iter().all(|b| *b == 0xFF),
        "cartao Sony aborta sem mandar dados, checksum nem byte de fim"
    );
}

#[test]
fn escrita_em_setor_invalido_devolve_ffh() {
    let mut cartao = MemoryCard::new();
    let dados = vec![0u8; FRAME_BYTES];
    let r = escreve_setor(&mut cartao, 0x3FFF, &dados, checksum(0x3FFF, &dados));
    assert_eq!(*r.last().unwrap(), 0xFF, "FFh = BadSector");
    assert!(!cartao.take_dirty());
}

#[test]
fn get_id_devolve_a_sequencia_fixa_da_spec() {
    let mut cartao = MemoryCard::new();
    let r = troca(&mut cartao, &[0x81, 0x53, 0, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&r[1..4], &[0x08, 0x5A, 0x5D]);
    assert_eq!(&r[4..10], &[0x5C, 0x5D, 0x04, 0x00, 0x00, 0x80]);
}

#[test]
fn bit3_do_flag_cai_na_escrita_e_nao_na_leitura() {
    let mut cartao = MemoryCard::new();
    let dados = vec![0u8; FRAME_BYTES];
    le_setor(&mut cartao, 0);
    assert_eq!(
        cartao.flag() & 0x08,
        0x08,
        "§ FLAG Byte (L2842): o bit3 NAO cai na leitura"
    );
    escreve_setor(&mut cartao, 0x3F, &dados, checksum(0x3F, &dados));
    assert_eq!(cartao.flag() & 0x08, 0, "cai na escrita");
}

#[test]
fn comando_invalido_aborta_logo_depois_do_byte_de_comando() {
    let mut cartao = MemoryCard::new();
    cartao.begin();
    assert_eq!(cartao.exchange(0x81), (0xFF, true));
    let (resposta, ack) = cartao.exchange(0x42);
    assert_eq!(resposta, 0x08, "o FLAG ainda sai");
    assert!(!ack, "e a transferencia acaba ali");
}

#[test]
fn sio_encaminha_o_endereco_81h_para_o_cartao_e_o_01h_para_o_pad() {
    let sio = Sio::new();
    sio.connect_digital_pad(true);
    sio.connect_memory_card(true);
    sio.write_byte(JOY_CTRL, 0x03);

    // A resposta ao byte de comando separa os dois: o pad devolve 41h (ID do digital),
    // o cartao devolve o FLAG (08h).
    for (endereco, comando, esperado) in [(0x01u8, 0x42u8, 0x41u8), (0x81, 0x53, 0x08)] {
        sio.write_byte(JOY_CTRL, 0x00);
        sio.write_byte(JOY_CTRL, 0x03);
        sio.write_byte(JOY_DATA, endereco);
        assert_eq!(
            sio.read_byte(JOY_DATA),
            0xFF,
            "byte de endereco nao tem resposta"
        );
        sio.write_byte(JOY_DATA, comando);
        assert_eq!(
            sio.read_byte(JOY_DATA),
            esperado,
            "endereco {endereco:02X} tem de cair no dispositivo certo"
        );
    }
}
