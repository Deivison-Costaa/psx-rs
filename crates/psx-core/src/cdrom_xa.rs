pub const RAW_SECTOR_BYTES: usize = 2352;
pub const CDDA_FRAMES: usize = RAW_SECTOR_BYTES / 4;
/// Mono 18900 Hz: 18 grupos x 8 unidades x 28 amostras, duplicadas a 37800 Hz e 7/6 a 44100.
pub const XA_MAX_FRAMES_PER_SECTOR: usize = GRUPOS * 2 * BLOCOS * AMOSTRAS * 2 * 7 / 6;

const GRUPOS: usize = 18;
const BLOCOS: usize = 4;
const AMOSTRAS: usize = 28;
const GRUPO_BYTES: usize = 128;

const POS: [i32; 4] = [0, 60, 115, 98];
const NEG: [i32; 4] = [0, 0, -52, -55];

const ANEL: usize = 32;
const TAPS: usize = 29;

/// § 25-point Zigzag Interpolation de docs/reference/15-cdrom-format.md: Table1..Table7,
/// cada uma com os coeficientes de Index 1..29.
#[rustfmt::skip]
const ZIGZAG: [[i16; TAPS]; 7] = [
    [0x0000, 0x0000, 0x0000, 0x0000, 0x0000, -0x0002, 0x000A, -0x0022, 0x0041, -0x0054, 0x0034, 0x0009, -0x010A, 0x0400, -0x0A78, 0x234C, 0x6794, -0x1780, 0x0BCD, -0x0623, 0x0350, -0x016D, 0x006B, 0x000A, -0x0010, 0x0011, -0x0008, 0x0003, -0x0001],
    [0x0000, 0x0000, 0x0000, -0x0002, 0x0000, 0x0003, -0x0013, 0x003C, -0x004B, 0x00A2, -0x00E3, 0x0132, -0x0043, -0x0267, 0x0C9D, 0x74BB, -0x11B4, 0x09B8, -0x05BF, 0x0372, -0x01A8, 0x00A6, -0x001B, 0x0005, 0x0006, -0x0008, 0x0003, -0x0001, 0x0000],
    [0x0000, 0x0000, -0x0001, 0x0003, -0x0002, -0x0005, 0x001F, -0x004A, 0x00B3, -0x0192, 0x02B1, -0x039E, 0x04F8, -0x05A6, 0x7939, -0x05A6, 0x04F8, -0x039E, 0x02B1, -0x0192, 0x00B3, -0x004A, 0x001F, -0x0005, -0x0002, 0x0003, -0x0001, 0x0000, 0x0000],
    [0x0000, -0x0001, 0x0003, -0x0008, 0x0006, 0x0005, -0x001B, 0x00A6, -0x01A8, 0x0372, -0x05BF, 0x09B8, -0x11B4, 0x74BB, 0x0C9D, -0x0267, -0x0043, 0x0132, -0x00E3, 0x00A2, -0x004B, 0x003C, -0x0013, 0x0003, 0x0000, -0x0002, 0x0000, 0x0000, 0x0000],
    [-0x0001, 0x0003, -0x0008, 0x0011, -0x0010, 0x000A, 0x006B, -0x016D, 0x0350, -0x0623, 0x0BCD, -0x1780, 0x6794, 0x234C, -0x0A78, 0x0400, -0x010A, 0x0009, 0x0034, -0x0054, 0x0041, -0x0022, 0x000A, -0x0001, 0x0000, 0x0001, 0x0000, 0x0000, 0x0000],
    [0x0002, -0x0008, 0x0010, -0x0023, 0x002B, 0x001A, -0x00EB, 0x027B, -0x0548, 0x0AFA, -0x16FA, 0x53E0, 0x3C07, -0x1249, 0x080E, -0x0347, 0x015B, -0x0044, -0x0017, 0x0046, -0x0023, 0x0011, -0x0005, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
    [-0x0005, 0x0011, -0x0023, 0x0046, -0x0017, -0x0044, 0x015B, -0x0347, 0x080E, -0x1249, 0x3C07, 0x53E0, -0x16FA, 0x0AFA, -0x0548, 0x027B, -0x00EB, 0x001A, 0x002B, -0x0023, 0x0010, -0x0008, 0x0002, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000],
];

/// Old/older do ADPCM e o anel de 32 amostras + contador de seis passos do zigzag:
/// tudo atravessa a fronteira do setor.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct XaState {
    pub old_left: i32,
    pub older_left: i32,
    pub old_right: i32,
    pub older_right: i32,
    pub ring_left: [i16; ANEL],
    pub ring_right: [i16; ANEL],
    pub ring_pos: u8,
    pub six_step: u8,
}

impl XaState {
    fn push_37800(&mut self, (esq, dir): (i16, i16), saida: &mut Vec<(i16, i16)>) {
        let p = usize::from(self.ring_pos) % ANEL;
        self.ring_left[p] = esq;
        self.ring_right[p] = dir;
        self.ring_pos = ((p + 1) % ANEL) as u8;
        self.six_step += 1;
        if self.six_step < 6 {
            return;
        }
        self.six_step = 0;
        let pos = usize::from(self.ring_pos);
        saida.extend(ZIGZAG.iter().map(|tabela| {
            (
                zigzag(&self.ring_left, pos, tabela),
                zigzag(&self.ring_right, pos, tabela),
            )
        }));
    }
}

fn zigzag(anel: &[i16; ANEL], pos: usize, tabela: &[i16; TAPS]) -> i16 {
    let soma: i32 = tabela
        .iter()
        .enumerate()
        .map(|(k, &c)| (i32::from(anel[(pos + ANEL - 1 - k) % ANEL]) * i32::from(c)) >> 15)
        .sum();
    soma.clamp(-0x8000, 0x7FFF) as i16
}

/// § decode_28_nibbles (L963) de docs/reference/15-cdrom-format.md; `src` e o grupo de 128.
pub fn decode_28_nibbles(
    src: &[u8],
    blk: usize,
    nibble: usize,
    mut old: i32,
    mut older: i32,
) -> ([i16; AMOSTRAS], i32, i32) {
    let par = src.get(4 + blk * 2 + nibble).copied().unwrap_or(0);
    let shift = 12u32.saturating_sub(u32::from(par & 0x0F));
    let filtro = usize::from((par & 0x30) >> 4).min(POS.len() - 1);
    let (f0, f1) = (POS[filtro], NEG[filtro]);

    let mut out = [0i16; AMOSTRAS];
    for (j, slot) in out.iter_mut().enumerate() {
        let byte = src.get(16 + blk + j * 4).copied().unwrap_or(0);
        let bruto = (byte >> (nibble * 4)) & 0x0F;
        let t = i32::from(bruto) - if bruto >= 8 { 16 } else { 0 };
        let s = ((t << shift) + ((old * f0 + older * f1 + 32) >> 6)).clamp(-0x8000, 0x7FFF);
        *slot = s as i16;
        older = old;
        old = s;
    }
    (out, old, older)
}

/// § decode_sector (L944) de docs/reference/15-cdrom-format.md; mono sai nos dois canais.
pub fn decode_sector(src: &[u8], stereo: bool, state: &mut XaState) -> Vec<(i16, i16)> {
    let inicio = 12 + 4 + 8;
    let mut saida = Vec::with_capacity(GRUPOS * BLOCOS * AMOSTRAS * if stereo { 1 } else { 2 });
    for grupo in 0..GRUPOS {
        let base = inicio + grupo * GRUPO_BYTES;
        if base + GRUPO_BYTES > src.len() {
            break;
        }
        let dados = &src[base..base + GRUPO_BYTES];
        for blk in 0..BLOCOS {
            if stereo {
                let (esq, ol, oler) =
                    decode_28_nibbles(dados, blk, 0, state.old_left, state.older_left);
                state.old_left = ol;
                state.older_left = oler;
                let (dir, or, orer) =
                    decode_28_nibbles(dados, blk, 1, state.old_right, state.older_right);
                state.old_right = or;
                state.older_right = orer;
                saida.extend(esq.iter().zip(dir.iter()).map(|(a, b)| (*a, *b)));
            } else {
                for nibble in 0..2 {
                    let (mono, ol, oler) =
                        decode_28_nibbles(dados, blk, nibble, state.old_left, state.older_left);
                    state.old_left = ol;
                    state.older_left = oler;
                    saida.extend(mono.iter().map(|a| (*a, *a)));
                }
            }
        }
    }
    saida
}

pub fn cdda_frames(raw: &[u8]) -> Vec<(i16, i16)> {
    raw.chunks_exact(4)
        .map(|q| {
            (
                i16::from_le_bytes([q[0], q[1]]),
                i16::from_le_bytes([q[2], q[3]]),
            )
        })
        .collect()
}

pub const OUTPUT_HZ: u32 = 44100;

/// 37800 Hz entra uma vez no anel e 18900 Hz entra duas; a cada seis entradas saem sete
/// quadros de 44100 Hz, um por tabela do zigzag.
pub fn resample_to_44100(
    frames: &[(i16, i16)],
    source_hz: u32,
    state: &mut XaState,
) -> Vec<(i16, i16)> {
    let repeticoes = match source_hz {
        37800 => 1,
        18900 => 2,
        _ => return frames.to_vec(),
    };
    let mut saida = Vec::with_capacity(frames.len() * repeticoes * 7 / 6 + 7);
    for &quadro in frames {
        for _ in 0..repeticoes {
            state.push_37800(quadro, &mut saida);
        }
    }
    saida
}

pub fn xa_sample_rate(coding: u8) -> u32 {
    if coding & 0x04 == 0 { 37800 } else { 18900 }
}

pub fn xa_is_stereo(coding: u8) -> bool {
    coding & 0x03 == 1
}

// § Data/ADPCM Sector Filtering/Delivery (06-cdrom.md L760-782): "reject if submode isn't
// audio+realtime (bit2 and bit6 must be both set)" — bit2=Audio, bit6=Real Time (RT),
// 15-cdrom-format.md L778-787. So o bit2 (sem o bit6) deixava passar setores de video
// (que a mesma nota diz serem marcados como Data, bit3, nao Video) que so por acaso
// tinham o bit de audio ligado.
pub fn is_xa_audio_sector(raw: &[u8]) -> bool {
    raw.len() >= 0x18 && raw[0x0F] == 0x02 && raw[0x12] & 0x44 == 0x44
}
