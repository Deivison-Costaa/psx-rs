use crate::memcard::CARD_BYTES;

const FRAME_BYTES: usize = 128;
const BLOCO_BYTES: usize = 8192;
const BLOCOS: usize = 15;
const TITULO_BYTES: usize = 64;
const CARTAO_PADRAO: &str = "sem-serial.mcd";

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
    if imagem.len() != CARD_BYTES {
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
pub fn nome_do_cartao(serial: &str) -> String {
    let limpo: String = serial
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    if limpo.trim_matches(['-', '_']).is_empty() {
        return CARTAO_PADRAO.to_string();
    }
    format!("{limpo}.mcd")
}
