use psx_core::app::saves::{self, ErroCartao};
use psx_core::memcard::{formatted_image, is_formatted};

const FRAME: usize = 128;
const CARTAO: usize = 128 * 1024;
const CABECALHO_GME: usize = 0xF40;
const CABECALHO_VGS: usize = 64;

fn xor(bytes: &[u8]) -> u8 {
    bytes.iter().fold(0, |a, b| a ^ b)
}

fn entrada(imagem: &mut [u8], frame: usize, estado: u8, tamanho: u32, proximo: u16, nome: &[u8]) {
    let base = frame * FRAME;
    imagem[base..base + FRAME].fill(0);
    imagem[base] = estado;
    imagem[base + 4..base + 8].copy_from_slice(&tamanho.to_le_bytes());
    imagem[base + 8..base + 10].copy_from_slice(&proximo.to_le_bytes());
    imagem[base + 0x0A..base + 0x0A + nome.len()].copy_from_slice(nome);
    imagem[base + FRAME - 1] = xor(&imagem[base..base + FRAME - 1]);
}

/// Bloco 1: save de 1 bloco. Blocos 2-3-5: save de 3 blocos encadeado fora de ordem.
fn cartao() -> Vec<u8> {
    let mut img = formatted_image();
    entrada(&mut img, 1, 0x51, 0x2000, 0xFFFF, b"BASCUS-94900CRASH");
    entrada(&mut img, 2, 0x51, 0x6000, 2, b"BASLUS-00001LONGO");
    entrada(&mut img, 3, 0x52, 0, 4, b"");
    entrada(&mut img, 5, 0x53, 0, 0xFFFF, b"");
    img
}

fn estado(img: &[u8], frame: usize) -> u8 {
    img[frame * FRAME]
}

fn checksum_ok(img: &[u8], frame: usize) -> bool {
    xor(&img[frame * FRAME..(frame + 1) * FRAME]) == 0
}

#[test]
fn imagem_sem_magico_mc_nao_lista_saves_fantasmas() {
    let mut lixo = cartao();
    lixo[0] = b'X';
    assert!(
        saves::lista(&lixo).is_empty(),
        "0198.5: imagem sem 'MC' no quadro 0 nao e cartao, e o diretorio dela e lixo"
    );
    assert_eq!(saves::lista(&cartao()).len(), 2);
}

#[test]
fn apagar_marca_a_cadeia_inteira_como_apagada() {
    let novo = saves::apaga(&cartao(), 2).expect("apagar o save longo");
    assert_eq!(estado(&novo, 2), 0xA1);
    assert_eq!(estado(&novo, 3), 0xA2);
    assert_eq!(estado(&novo, 5), 0xA3);
    assert_eq!(estado(&novo, 1), 0x51, "o outro save fica intacto");
    for frame in [2, 3, 5] {
        assert!(checksum_ok(&novo, frame), "checksum do quadro {frame}");
    }
    let restantes = saves::lista(&novo);
    assert_eq!(restantes.len(), 1);
    assert_eq!(restantes[0].nome, "BASCUS-94900CRASH");
}

#[test]
fn apagar_nao_mexe_na_imagem_original() {
    let original = cartao();
    let _ = saves::apaga(&original, 1).expect("apagar");
    assert_eq!(estado(&original, 1), 0x51);
}

#[test]
fn apagar_bloco_que_nao_e_inicio_de_arquivo_e_recusado() {
    assert_eq!(saves::apaga(&cartao(), 3), Err(ErroCartao::NaoEInicio(3)));
    assert_eq!(saves::apaga(&cartao(), 4), Err(ErroCartao::NaoEInicio(4)));
    assert_eq!(
        saves::apaga(&cartao(), 0),
        Err(ErroCartao::BlocoInvalido(0))
    );
    assert_eq!(
        saves::apaga(&cartao(), 16),
        Err(ErroCartao::BlocoInvalido(16))
    );
}

#[test]
fn cadeia_em_laco_nao_trava() {
    let mut img = formatted_image();
    entrada(&mut img, 1, 0x51, 0x4000, 1, b"LACO");
    entrada(&mut img, 2, 0x52, 0, 1, b"");
    let novo = saves::apaga(&img, 1).expect("apagar mesmo com laco");
    assert_eq!(estado(&novo, 1), 0xA1);
    assert_eq!(estado(&novo, 2), 0xA2);
}

#[test]
fn blocos_livres_conta_o_que_nao_esta_em_uso() {
    assert_eq!(saves::blocos_livres(&formatted_image()), 15);
    assert_eq!(saves::blocos_livres(&cartao()), 11);
    let apagado = saves::apaga(&cartao(), 2).expect("apagar");
    assert_eq!(saves::blocos_livres(&apagado), 14);
}

#[test]
fn importa_imagem_crua_de_128_kib() {
    assert_eq!(saves::importa(&cartao()), Ok(cartao()));
}

#[test]
fn importa_recusa_tamanho_errado_e_falta_de_magico() {
    assert_eq!(saves::importa(&[0u8; 100]), Err(ErroCartao::Tamanho(100)));
    assert_eq!(
        saves::importa(&vec![0u8; CARTAO]),
        Err(ErroCartao::SemMagico)
    );
}

#[test]
fn importa_gme_do_dexdrive_tirando_o_cabecalho() {
    let mut gme = vec![0u8; CABECALHO_GME];
    gme[..12].copy_from_slice(b"123-456-STD\0");
    gme.extend_from_slice(&cartao());
    assert_eq!(saves::importa(&gme), Ok(cartao()));
}

#[test]
fn importa_mem_do_vgs_tirando_o_cabecalho() {
    let mut vgs = vec![0u8; CABECALHO_VGS];
    vgs[..4].copy_from_slice(b"VgsM");
    vgs.extend_from_slice(&cartao());
    assert_eq!(saves::importa(&vgs), Ok(cartao()));
}

#[test]
fn formatar_devolve_cartao_valido_e_vazio() {
    let novo = saves::formatado();
    assert!(is_formatted(&novo));
    assert!(saves::lista(&novo).is_empty());
}

#[test]
fn erro_vira_texto_em_portugues() {
    assert!(ErroCartao::SemMagico.to_string().contains("MC"));
    assert!(ErroCartao::Tamanho(5).to_string().contains('5'));
}
