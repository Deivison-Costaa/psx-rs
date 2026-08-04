pub const SETOR_LICENCA: u32 = 4;
pub const SETOR_PVD: u32 = 16;
pub const ARQUIVO_DE_BOOT: &str = "SYSTEM.CNF;1";

const BYTES_POR_SETOR: usize = 2048;
const SETOR_BRUTO: usize = 2352;
const MAX_SETORES: u32 = 16;
const ASSINATURA: &[u8] = b"CD001";
const EDITORA: &[u8] = b"Sony Computer Entertainment";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Regiao {
    America,
    Europa,
    Japao,
    #[default]
    Desconhecida,
}

impl Regiao {
    pub fn nome(&self) -> &'static str {
        match self {
            Regiao::America => "NTSC-U",
            Regiao::Europa => "PAL",
            Regiao::Japao => "NTSC-J",
            Regiao::Desconhecida => "?",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Extensao {
    pub lba: u32,
    pub tamanho: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Identidade {
    pub serial: Option<String>,
    pub regiao: Regiao,
    pub rotulo: Option<String>,
    pub boot: Option<String>,
}

fn u32_le(bytes: &[u8], em: usize) -> Option<u32> {
    let fatia = bytes.get(em..em + 4)?;
    Some(u32::from_le_bytes([fatia[0], fatia[1], fatia[2], fatia[3]]))
}

fn contem(agulha: &[u8], palheiro: &[u8]) -> bool {
    palheiro.windows(agulha.len()).any(|j| j == agulha)
}

pub fn regiao_da_licenca(setor: &[u8]) -> Regiao {
    let Some(linha) = setor.get(0x20..0x46) else {
        return Regiao::Desconhecida;
    };
    if !contem(EDITORA, linha) {
        return Regiao::Desconhecida;
    }
    if contem(b"Amer", linha) {
        Regiao::America
    } else if contem(b"Euro", linha) {
        Regiao::Europa
    } else if contem(b"Inc.", linha) {
        Regiao::Japao
    } else {
        Regiao::Desconhecida
    }
}

fn e_um_pvd(pvd: &[u8]) -> bool {
    pvd.first() == Some(&0x01) && pvd.get(0x01..0x06) == Some(ASSINATURA)
}

pub fn rotulo_do_volume(pvd: &[u8]) -> Option<String> {
    if !e_um_pvd(pvd) {
        return None;
    }
    let bruto = pvd.get(0x28..0x48)?;
    let texto: String = bruto
        .iter()
        .take_while(|b| **b != 0)
        .map(|b| *b as char)
        .collect();
    let limpo = texto.trim();
    if limpo.is_empty() {
        None
    } else {
        Some(limpo.to_string())
    }
}

pub fn raiz_do_pvd(pvd: &[u8]) -> Option<Extensao> {
    if !e_um_pvd(pvd) {
        return None;
    }
    let registro = pvd.get(0x9C..0x9C + 34)?;
    Some(Extensao {
        lba: u32_le(registro, 0x02)?,
        tamanho: u32_le(registro, 0x0A)?,
    })
}

// LEN_DR ja inclui o padding e os 14 bytes de System Use do CD-XA; somar 33+LEN_FI pula para
// o meio do proximo registro. § Format of a Directory Record (L1262) de 15-cdrom-format.md.
pub fn procura_no_diretorio(dir: &[u8], nome: &str) -> Option<Extensao> {
    let mut em = 0usize;
    while em < dir.len() {
        let len_dr = *dir.get(em)? as usize;
        if len_dr == 0 {
            return None;
        }
        let registro = dir.get(em..em + len_dr)?;
        let len_fi = *registro.get(0x20)? as usize;
        let cru = registro.get(0x21..0x21 + len_fi)?;
        let atual: String = cru.iter().map(|b| *b as char).collect();
        if atual.eq_ignore_ascii_case(nome) {
            return Some(Extensao {
                lba: u32_le(registro, 0x02)?,
                tamanho: u32_le(registro, 0x0A)?,
            });
        }
        em += len_dr;
    }
    None
}

pub fn caminho_de_boot(texto: &str) -> Option<String> {
    for linha in texto.lines() {
        let Some((chave, valor)) = linha.split_once('=') else {
            continue;
        };
        if !chave.trim().eq_ignore_ascii_case("BOOT") {
            continue;
        }
        let limpo = valor.trim().trim_matches('\0').trim();
        if limpo.is_empty() {
            return None;
        }
        return Some(limpo.to_string());
    }
    None
}

pub fn serial_do_boot(texto: &str) -> Option<String> {
    let caminho = caminho_de_boot(texto)?;
    let base = caminho.rsplit(['\\', '/', ':']).next()?;
    let sem_versao = base.split(';').next()?;
    let serial: String = sem_versao
        .chars()
        .filter(|c| *c != '.')
        .map(|c| {
            if c == '_' {
                '-'
            } else {
                c.to_ascii_uppercase()
            }
        })
        .collect();
    if serial.is_empty() {
        None
    } else {
        Some(serial)
    }
}

pub fn dados_do_setor(bruto: &[u8]) -> Option<&[u8]> {
    if bruto.len() >= SETOR_BRUTO {
        let inicio = if bruto[0x0F] == 0x02 { 0x18 } else { 0x10 };
        return bruto.get(inicio..inicio + BYTES_POR_SETOR);
    }
    bruto.get(..BYTES_POR_SETOR)
}

fn setores_de(tamanho: u32) -> u32 {
    tamanho
        .max(1)
        .div_ceil(BYTES_POR_SETOR as u32)
        .min(MAX_SETORES)
}

pub fn identifica<F>(mut le_setor: F) -> Identidade
where
    F: FnMut(u32) -> Option<Vec<u8>>,
{
    let mut id = Identidade::default();
    if let Some(licenca) = le_setor(SETOR_LICENCA) {
        id.regiao = regiao_da_licenca(&licenca);
    }
    let Some(pvd) = le_setor(SETOR_PVD) else {
        return id;
    };
    id.rotulo = rotulo_do_volume(&pvd);
    let Some(raiz) = raiz_do_pvd(&pvd) else {
        return id;
    };

    let mut alvo = None;
    for i in 0..setores_de(raiz.tamanho) {
        let Some(dir) = le_setor(raiz.lba + i) else {
            break;
        };
        if let Some(achado) = procura_no_diretorio(&dir, ARQUIVO_DE_BOOT) {
            alvo = Some(achado);
            break;
        }
    }
    let Some(alvo) = alvo else {
        return id;
    };

    let mut texto = String::new();
    for i in 0..setores_de(alvo.tamanho) {
        let Some(setor) = le_setor(alvo.lba + i) else {
            break;
        };
        texto.push_str(&String::from_utf8_lossy(&setor));
    }
    id.boot = caminho_de_boot(&texto);
    id.serial = serial_do_boot(&texto);
    id
}
