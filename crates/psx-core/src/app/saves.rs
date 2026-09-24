use crate::memcard::{CARD_BYTES, formatted_image};

const FRAME_BYTES: usize = 128;
const BLOCO_BYTES: usize = 8192;
const BLOCOS: usize = 15;
const TITULO_BYTES: usize = 64;
const CARTAO_PADRAO: &str = "sem-serial.mcd";
const MAGICO: &[u8] = b"MC";
const MAGICO_GME: &[u8] = b"123-456-STD";
const CABECALHO_GME: usize = 0xF40;
const MAGICO_VGS: &[u8] = b"VgsM";
const CABECALHO_VGS: usize = 64;
const PRIMEIRO: u8 = 0x51;
const MEIO: u8 = 0x52;
const ULTIMO: u8 = 0x53;
const APAGADO: u8 = 0xA0;
const FIM_DA_CADEIA: u16 = 0xFFFF;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErroCartao {
    Tamanho(usize),
    SemMagico,
    BlocoInvalido(u8),
    NaoEInicio(u8),
}

impl std::fmt::Display for ErroCartao {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErroCartao::Tamanho(n) => write!(
                f,
                "arquivo de {n} bytes não é um memory card (esperado 128 KiB, .gme ou .mem)"
            ),
            ErroCartao::SemMagico => {
                write!(f, "o arquivo não começa com a marca 'MC' de um memory card")
            }
            ErroCartao::BlocoInvalido(b) => write!(f, "bloco {b} fora de 1..15"),
            ErroCartao::NaoEInicio(b) => write!(f, "o bloco {b} não é o início de um save"),
        }
    }
}

impl std::error::Error for ErroCartao {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Save {
    pub bloco: u8,
    pub nome: String,
    pub titulo: String,
    pub blocos: u8,
    pub tamanho: u32,
}

fn u32_le(bytes: &[u8], em: usize) -> Option<u32> {
    let f = bytes.get(em..em + 4)?;
    Some(u32::from_le_bytes([f[0], f[1], f[2], f[3]]))
}

fn nome_do_arquivo(entrada: &[u8]) -> String {
    entrada
        .get(0x0A..0x1F)
        .unwrap_or(&[])
        .iter()
        .take_while(|b| **b != 0)
        .map(|b| *b as char)
        .collect()
}

/// § Shift-JIS Character Set (16bit) (L3006): o Title Frame costuma ser 16 bits, mas a BIOS
/// tambem aceita 20h..7Fh cru — os dois caminhos aparecem em cartao de jogo real.
fn titulo_shift_jis(bytes: &[u8]) -> String {
    let mut fora = String::new();
    let mut i = 0;
    while i + 1 < bytes.len() {
        let (alto, baixo) = (bytes[i], bytes[i + 1]);
        if alto == 0 {
            break;
        }
        if alto == 0x81 || alto == 0x82 {
            fora.push(par_shift_jis(alto, baixo));
            i += 2;
            continue;
        }
        if (0x20..0x80).contains(&alto) {
            fora.push(alto as char);
            i += 1;
            continue;
        }
        fora.push('?');
        i += 2;
    }
    fora.trim_end().to_string()
}

fn par_shift_jis(alto: u8, baixo: u8) -> char {
    match (alto, baixo) {
        (0x81, 0x40) => ' ',
        (0x82, 0x4F..=0x58) => (b'0' + (baixo - 0x4F)) as char,
        (0x82, 0x60..=0x79) => (b'A' + (baixo - 0x60)) as char,
        (0x82, 0x81..=0x9A) => (b'a' + (baixo - 0x81)) as char,
        _ => '?',
    }
}

fn titulo_do_bloco(imagem: &[u8], bloco: usize) -> String {
    let base = bloco * BLOCO_BYTES;
    let Some(frame) = imagem.get(base..base + FRAME_BYTES) else {
        return String::new();
    };
    if &frame[0..2] != b"SC" {
        return String::new();
    }
    titulo_shift_jis(&frame[0x04..0x04 + TITULO_BYTES])
}

/// Diretorio: bloco 0, frames 1..15. Estado 51h e o primeiro (ou unico) bloco de um
/// arquivo; 52h/53h sao continuacoes do MESMO arquivo e nao viram entrada propria.
pub fn lista(imagem: &[u8]) -> Vec<Save> {
    if !e_cartao(imagem) {
        return Vec::new();
    }
    let mut fora = Vec::new();
    for bloco in 1..=BLOCOS {
        let base = bloco * FRAME_BYTES;
        let Some(entrada) = imagem.get(base..base + FRAME_BYTES) else {
            continue;
        };
        if u32_le(entrada, 0) != Some(0x0000_0051) {
            continue;
        }
        let tamanho = u32_le(entrada, 4).unwrap_or(0);
        fora.push(Save {
            bloco: bloco as u8,
            nome: nome_do_arquivo(entrada),
            titulo: titulo_do_bloco(imagem, bloco),
            blocos: (tamanho / BLOCO_BYTES as u32).max(1) as u8,
            tamanho,
        });
    }
    fora
}

/// O serial vem de dentro do disco, que e entrada externa: so sobrevive `[A-Za-z0-9-_]`,
/// para nao virar caminho relativo na pasta de cartoes.
pub fn serial_limpo(serial: &str) -> String {
    serial
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect()
}

pub fn nome_do_cartao(serial: &str) -> String {
    let limpo = serial_limpo(serial);
    if limpo.trim_matches(['-', '_']).is_empty() {
        return CARTAO_PADRAO.to_string();
    }
    format!("{limpo}.mcd")
}

pub fn e_cartao(imagem: &[u8]) -> bool {
    imagem.len() == CARD_BYTES && imagem.starts_with(MAGICO)
}

pub fn formatado() -> Vec<u8> {
    formatted_image()
}

/// Cru (.mcd/.mcr/.mc), DexDrive (.gme) ou VGS (.mem): os dois ultimos so tem cabecalho
/// de tamanho fixo antes da imagem crua.
pub fn importa(bytes: &[u8]) -> Result<Vec<u8>, ErroCartao> {
    let corpo = if bytes.len() == CABECALHO_GME + CARD_BYTES && bytes.starts_with(MAGICO_GME) {
        &bytes[CABECALHO_GME..]
    } else if bytes.len() == CABECALHO_VGS + CARD_BYTES && bytes.starts_with(MAGICO_VGS) {
        &bytes[CABECALHO_VGS..]
    } else if bytes.len() == CARD_BYTES {
        bytes
    } else {
        return Err(ErroCartao::Tamanho(bytes.len()));
    };
    if !corpo.starts_with(MAGICO) {
        return Err(ErroCartao::SemMagico);
    }
    Ok(corpo.to_vec())
}

fn estado(imagem: &[u8], bloco: usize) -> u8 {
    imagem.get(bloco * FRAME_BYTES).copied().unwrap_or(0)
}

pub fn blocos_livres(imagem: &[u8]) -> u8 {
    (1..=BLOCOS)
        .filter(|b| !matches!(estado(imagem, *b), PRIMEIRO | MEIO | ULTIMO))
        .count() as u8
}

fn marca_apagado(imagem: &mut [u8], bloco: usize) {
    let base = bloco * FRAME_BYTES;
    let atual = imagem[base];
    imagem[base] = APAGADO | (atual & 0x0F);
    let soma = imagem[base..base + FRAME_BYTES - 1]
        .iter()
        .fold(0u8, |a, b| a ^ b);
    imagem[base + FRAME_BYTES - 1] = soma;
}

/// Como a BIOS apaga: 51h/52h/53h viram A1h/A2h/A3h seguindo a cadeia de "proximo bloco",
/// e o conteudo fica (da para desapagar). Devolve uma imagem NOVA.
pub fn apaga(imagem: &[u8], bloco: u8) -> Result<Vec<u8>, ErroCartao> {
    if !e_cartao(imagem) {
        return Err(ErroCartao::SemMagico);
    }
    let inicio = usize::from(bloco);
    if !(1..=BLOCOS).contains(&inicio) {
        return Err(ErroCartao::BlocoInvalido(bloco));
    }
    if estado(imagem, inicio) != PRIMEIRO {
        return Err(ErroCartao::NaoEInicio(bloco));
    }
    let mut nova = imagem.to_vec();
    let mut atual = inicio;
    for _ in 0..BLOCOS {
        marca_apagado(&mut nova, atual);
        let base = atual * FRAME_BYTES;
        let proximo = u16::from_le_bytes([nova[base + 8], nova[base + 9]]);
        if proximo == FIM_DA_CADEIA {
            break;
        }
        let seguinte = usize::from(proximo) + 1;
        if !(1..=BLOCOS).contains(&seguinte) || !matches!(estado(&nova, seguinte), MEIO | ULTIMO) {
            break;
        }
        atual = seguinte;
    }
    Ok(nova)
}
