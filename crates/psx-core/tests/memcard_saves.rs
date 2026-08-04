use psx_core::app::saves::{self, Save};

const FRAME: usize = 128;
const BLOCO: usize = 8192;
const CARTAO: usize = 128 * 1024;

const LIVRE: u32 = 0x0000_00A0;
const APAGADO: u32 = 0x0000_00A1;
const PRIMEIRO: u32 = 0x0000_0051;
const ULTIMO: u32 = 0x0000_0053;

fn entrada(imagem: &mut [u8], frame: usize, estado: u32, tamanho: u32, proximo: u16, nome: &[u8]) {
    let base = frame * FRAME;
    imagem[base..base + 4].copy_from_slice(&estado.to_le_bytes());
    imagem[base + 4..base + 8].copy_from_slice(&tamanho.to_le_bytes());
    imagem[base + 8..base + 10].copy_from_slice(&proximo.to_le_bytes());
    imagem[base + 0x0A..base + 0x0A + nome.len()].copy_from_slice(nome);
}

fn titulo_16bits(texto: &str) -> Vec<u8> {
    let mut fora = Vec::new();
    for c in texto.chars() {
        let par = match c {
            ' ' => [0x81, 0x40],
            '0'..='9' => [0x82, 0x4F + (c as u8 - b'0')],
            'A'..='Z' => [0x82, 0x60 + (c as u8 - b'A')],
            'a'..='z' => [0x82, 0x81 + (c as u8 - b'a')],
            _ => [0x81, 0x43],
        };
        fora.extend_from_slice(&par);
    }
    fora
}

fn frame_de_titulo(imagem: &mut [u8], bloco: usize, corpo: &[u8]) {
    let base = bloco * BLOCO;
    imagem[base] = b'S';
    imagem[base + 1] = b'C';
    imagem[base + 2] = 0x11;
    imagem[base + 3] = 0x01;
    imagem[base + 4..base + 4 + corpo.len()].copy_from_slice(corpo);
}

fn cartao() -> Vec<u8> {
    let mut imagem = vec![0u8; CARTAO];
    imagem[0] = b'M';
    imagem[1] = b'C';
    for frame in 1..=15 {
        entrada(&mut imagem, frame, LIVRE, 0, 0xFFFF, b"");
    }

    entrada(
        &mut imagem,
        1,
        PRIMEIRO,
        0x2000,
        0xFFFF,
        b"BASLUS-00005RAY01",
    );
    frame_de_titulo(&mut imagem, 1, &titulo_16bits("RAYMAN 2 fase"));

    entrada(&mut imagem, 2, PRIMEIRO, 0x4000, 2, b"BASCUS-94900CRASH");
    frame_de_titulo(&mut imagem, 2, b"CRASH BANDICOOT\0");
    entrada(&mut imagem, 3, ULTIMO, 0, 0xFFFF, b"");

    entrada(
        &mut imagem,
        4,
        APAGADO,
        0x2000,
        0xFFFF,
        b"BASLES-00001MORTO",
    );
    imagem
}

#[test]
fn lista_so_os_arquivos_vivos_do_diretorio() {
    let saves = saves::lista(&cartao());
    assert_eq!(saves.len(), 2, "achou {saves:?}");
    assert_eq!(saves[0].nome, "BASLUS-00005RAY01");
    assert_eq!(saves[1].nome, "BASCUS-94900CRASH");
}

#[test]
fn bloco_de_continuacao_nao_vira_arquivo_proprio() {
    let saves = saves::lista(&cartao());
    assert!(
        !saves.iter().any(|s| s.bloco == 3),
        "o bloco 3 tem estado 53h: e a segunda metade do CRASH, nao um arquivo novo"
    );
}

#[test]
fn arquivo_apagado_nao_aparece() {
    let saves = saves::lista(&cartao());
    assert!(!saves.iter().any(|s| s.nome.contains("MORTO")));
}

#[test]
fn tamanho_em_blocos_sai_do_filesize() {
    let saves = saves::lista(&cartao());
    assert_eq!(saves[0].blocos, 1);
    assert_eq!(saves[1].blocos, 2);
    assert_eq!(saves[1].tamanho, 0x4000);
}

#[test]
fn titulo_em_shift_jis_de_16_bits_vira_texto() {
    let saves = saves::lista(&cartao());
    assert_eq!(
        saves[0].titulo, "RAYMAN 2 fase",
        "as quatro faixas do Shift-JIS de 16 bits: espaco, digito, maiuscula e minuscula"
    );
}

#[test]
fn titulo_em_ascii_de_8_bits_tambem_e_aceito() {
    let saves = saves::lista(&cartao());
    assert_eq!(
        saves[1].titulo, "CRASH BANDICOOT",
        "a BIOS aceita 20h..7Fh cru no Title Frame, nao so o par de 16 bits"
    );
}

#[test]
fn bloco_sem_assinatura_sc_fica_sem_titulo() {
    let mut imagem = cartao();
    imagem[BLOCO] = b'X';
    let saves = saves::lista(&imagem);
    assert_eq!(saves[0].titulo, "");
}

#[test]
fn cartao_de_tamanho_errado_nao_lista_nada() {
    assert_eq!(saves::lista(&[]), Vec::<Save>::new());
    let mut cortado = cartao();
    cortado.truncate(CARTAO / 2);
    assert_eq!(
        saves::lista(&cortado),
        Vec::<Save>::new(),
        "o diretorio inteiro cabe nos primeiros 2 KiB: sem conferir o tamanho, meio cartao \
         listaria arquivos cujos blocos de dados nao existem no arquivo"
    );
}

#[test]
fn cartao_zerado_nao_lista_nada() {
    assert_eq!(saves::lista(&vec![0u8; CARTAO]), Vec::<Save>::new());
}

#[test]
fn nome_do_cartao_sai_do_serial() {
    assert_eq!(saves::nome_do_cartao("SLUS-00005"), "SLUS-00005.mcd");
    assert_eq!(saves::nome_do_cartao("SCUS-94900"), "SCUS-94900.mcd");
}

#[test]
fn serial_com_caminho_nao_escapa_da_pasta() {
    assert_eq!(
        saves::nome_do_cartao("../../etc/passwd"),
        "etcpasswd.mcd",
        "o serial vem de dentro do disco: barra, ponto e dois-pontos nao podem virar caminho"
    );
    assert!(!saves::nome_do_cartao("..\\..\\windows").contains('\\'));
}

#[test]
fn serial_vazio_cai_num_cartao_padrao() {
    assert_eq!(saves::nome_do_cartao(""), "sem-serial.mcd");
    assert_eq!(saves::nome_do_cartao("///"), "sem-serial.mcd");
}
