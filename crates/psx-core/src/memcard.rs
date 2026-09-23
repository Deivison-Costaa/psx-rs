pub const FRAME_BYTES: usize = 128;
pub const FRAMES: usize = 1024;
pub const CARD_BYTES: usize = FRAME_BYTES * FRAMES;
pub const ADDRESS: u8 = 0x81;

const FLAG_INICIAL: u8 = 0x08;
const FLAG_DIRETORIO_NAO_LIDA: u8 = 0x08;

const ULTIMO_DIRETORIO: usize = 15;
const ULTIMO_SETOR_RUIM: usize = 35;
const QUADROS_FORMATADOS: usize = ULTIMO_SETOR_RUIM + 1;
const LIVRE_FORMATADO: u8 = 0xA0;
const NENHUM: [u8; 4] = [0xFF; 4];

fn xor(bytes: &[u8]) -> u8 {
    bytes.iter().fold(0, |acc, b| acc ^ b)
}

// § Memory Card Data Format (L2666-2750) de docs/reference/10-controllers-memcards.md, no
// leiaute exato que o shell da SCPH1001 grava ao formatar: quadros 36..63 ficam zerados.
fn quadro_formatado(indice: usize) -> [u8; FRAME_BYTES] {
    let mut quadro = [0u8; FRAME_BYTES];
    match indice {
        0 => quadro[0..2].copy_from_slice(b"MC"),
        1..=ULTIMO_DIRETORIO => quadro[0] = LIVRE_FORMATADO,
        _ => quadro[0..4].copy_from_slice(&NENHUM),
    }
    if indice > 0 {
        quadro[8..10].copy_from_slice(&NENHUM[..2]);
    }
    quadro[FRAME_BYTES - 1] = xor(&quadro[..FRAME_BYTES - 1]);
    quadro
}

/// Imagem de 128 KiB como sai do formatador da BIOS: cabecalho "MC", 15 entradas de
/// diretorio livres, lista de setores ruins vazia e o resto zerado.
pub fn formatted_image() -> Vec<u8> {
    (0..QUADROS_FORMATADOS)
        .flat_map(quadro_formatado)
        .chain(std::iter::repeat_n(
            0u8,
            CARD_BYTES - QUADROS_FORMATADOS * FRAME_BYTES,
        ))
        .collect()
}

pub fn is_formatted(imagem: &[u8]) -> bool {
    imagem.len() == CARD_BYTES && &imagem[0..2] == b"MC" && xor(&imagem[..FRAME_BYTES]) == 0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryCardError {
    TamanhoInvalido(usize),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum Modo {
    #[default]
    Ocioso,
    Read,
    Write,
    GetId,
    Abortado,
}

/// Cartao de memoria de 128 KiB no endereco 81h do SIO0.
/// § Memory Card Read/Write Commands (L2564) de docs/reference/10-controllers-memcards.md.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryCard {
    data: Vec<u8>,
    flag: u8,
    dirty: bool,
    modo: Modo,
    passo: usize,
    endereco: u16,
    anterior: u8,
    buffer: Vec<u8>,
    resultado: u8,
}

impl Default for MemoryCard {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryCard {
    pub fn new() -> Self {
        MemoryCard {
            data: vec![0u8; CARD_BYTES],
            flag: FLAG_INICIAL,
            dirty: false,
            modo: Modo::Ocioso,
            passo: 0,
            endereco: 0,
            anterior: 0,
            buffer: Vec::with_capacity(FRAME_BYTES),
            resultado: 0xFF,
        }
    }

    pub fn formatted() -> Self {
        MemoryCard {
            data: formatted_image(),
            ..Self::new()
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, MemoryCardError> {
        if bytes.len() != CARD_BYTES {
            return Err(MemoryCardError::TamanhoInvalido(bytes.len()));
        }
        let mut cartao = Self::new();
        cartao.data.copy_from_slice(bytes);
        Ok(cartao)
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn flag(&self) -> u8 {
        self.flag
    }

    pub fn take_dirty(&mut self) -> bool {
        std::mem::take(&mut self.dirty)
    }

    /// /CS baixou: comeca uma transferencia nova.
    pub fn begin(&mut self) {
        self.modo = Modo::Ocioso;
        self.passo = 0;
        self.endereco = 0;
        self.anterior = 0;
        self.buffer.clear();
        self.resultado = 0xFF;
    }

    fn setor_valido(&self) -> bool {
        (self.endereco as usize) < FRAMES
    }

    fn quadro(&self) -> &[u8] {
        let base = (self.endereco as usize) * FRAME_BYTES;
        &self.data[base..base + FRAME_BYTES]
    }

    fn checksum_do_quadro(&self) -> u8 {
        let inicial = (self.endereco >> 8) as u8 ^ self.endereco as u8;
        self.quadro().iter().fold(inicial, |acc, b| acc ^ b)
    }

    /// Um byte de SPI: devolve a resposta e se o cartao vai puxar /ACK depois dela.
    pub fn exchange(&mut self, entrada: u8) -> (u8, bool) {
        let passo = self.passo;
        self.passo += 1;
        let (resposta, ack) = match (self.modo, passo) {
            (_, 0) => (0xFF, entrada == ADDRESS),
            (_, 1) => {
                self.modo = match entrada {
                    0x52 => Modo::Read,
                    0x57 => Modo::Write,
                    0x53 => Modo::GetId,
                    _ => Modo::Abortado,
                };
                (self.flag, self.modo != Modo::Abortado)
            }
            (Modo::Read, _) => self.passo_de_leitura(passo, entrada),
            (Modo::Write, _) => self.passo_de_escrita(passo, entrada),
            (Modo::GetId, _) => Self::passo_de_get_id(passo),
            _ => (0xFF, false),
        };
        self.anterior = entrada;
        (resposta, ack)
    }

    // § Reading Data from Memory Card (L2565) de docs/reference/10-controllers-memcards.md.
    fn passo_de_leitura(&mut self, passo: usize, entrada: u8) -> (u8, bool) {
        const DADOS: usize = 10;
        match passo {
            2 => (0x5A, true),
            3 => (0x5D, true),
            4 => {
                self.endereco = u16::from(entrada) << 8;
                (0x00, true)
            }
            5 => {
                self.endereco |= u16::from(entrada);
                (self.anterior, true)
            }
            6 => (0x5C, true),
            7 => (0x5D, true),
            8 if !self.setor_valido() => {
                // Cartao Sony: confirma FFFFh e aborta sem mandar dados nem byte de fim.
                self.modo = Modo::Abortado;
                (0xFF, false)
            }
            8 => ((self.endereco >> 8) as u8, true),
            9 => (self.endereco as u8, true),
            p if (DADOS..DADOS + FRAME_BYTES).contains(&p) => (self.quadro()[p - DADOS], true),
            p if p == DADOS + FRAME_BYTES => (self.checksum_do_quadro(), true),
            p if p == DADOS + FRAME_BYTES + 1 => (0x47, false),
            _ => (0xFF, false),
        }
    }

    // § Writing Data to Memory Card (L2589) de docs/reference/10-controllers-memcards.md.
    fn passo_de_escrita(&mut self, passo: usize, entrada: u8) -> (u8, bool) {
        const DADOS: usize = 6;
        match passo {
            2 => (0x5A, true),
            3 => (0x5D, true),
            4 => {
                self.endereco = u16::from(entrada) << 8;
                (0x00, true)
            }
            5 => {
                self.endereco |= u16::from(entrada);
                (self.anterior, true)
            }
            p if (DADOS..DADOS + FRAME_BYTES).contains(&p) => {
                self.buffer.push(entrada);
                (self.anterior, true)
            }
            p if p == DADOS + FRAME_BYTES => {
                self.grava(entrada);
                (self.anterior, true)
            }
            p if p == DADOS + FRAME_BYTES + 1 => (0x5C, true),
            p if p == DADOS + FRAME_BYTES + 2 => (0x5D, true),
            p if p == DADOS + FRAME_BYTES + 3 => (self.fim_da_escrita(), false),
            _ => (0xFF, false),
        }
    }

    fn checksum_recebido(&self) -> u8 {
        let inicial = (self.endereco >> 8) as u8 ^ self.endereco as u8;
        self.buffer.iter().fold(inicial, |acc, b| acc ^ b)
    }

    fn grava(&mut self, checksum: u8) {
        if !self.setor_valido() {
            self.resultado = 0xFF;
            return;
        }
        if checksum != self.checksum_recebido() {
            self.resultado = 0x4E;
            return;
        }
        let base = (self.endereco as usize) * FRAME_BYTES;
        self.data[base..base + FRAME_BYTES].copy_from_slice(&self.buffer);
        self.flag &= !FLAG_DIRETORIO_NAO_LIDA;
        self.dirty = true;
        self.resultado = 0x47;
    }

    fn fim_da_escrita(&self) -> u8 {
        self.resultado
    }

    // § Get Memory Card ID Command (L2605) de docs/reference/10-controllers-memcards.md.
    fn passo_de_get_id(passo: usize) -> (u8, bool) {
        match passo {
            2 => (0x5A, true),
            3 => (0x5D, true),
            4 => (0x5C, true),
            5 => (0x5D, true),
            6 => (0x04, true),
            7 => (0x00, true),
            8 => (0x00, true),
            9 => (0x80, false),
            _ => (0xFF, false),
        }
    }
}
