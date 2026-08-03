pub const RAW_SECTOR_BYTES: usize = 2352;
pub const CDDA_FRAMES: usize = RAW_SECTOR_BYTES / 4;

const GRUPOS: usize = 18;
const BLOCOS: usize = 4;
const AMOSTRAS: usize = 28;
const GRUPO_BYTES: usize = 128;

const POS: [i32; 4] = [0, 60, 115, 98];
const NEG: [i32; 4] = [0, 0, -52, -55];

/// § Old/Older Values (L1119) de docs/reference/15-cdrom-format.md: atravessam o setor.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct XaState {
    pub old_left: i32,
    pub older_left: i32,
    pub old_right: i32,
    pub older_right: i32,
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

/// § 25-point Zigzag Interpolation (L991) de docs/reference/15-cdrom-format.md; aqui e
/// vizinho mais proximo: a taxa fica certa, o filtro nao.
pub fn resample_to_44100(frames: &[(i16, i16)], source_hz: u32) -> Vec<(i16, i16)> {
    if source_hz == 0 || source_hz == OUTPUT_HZ || frames.is_empty() {
        return frames.to_vec();
    }
    let total = frames.len() as u64 * u64::from(OUTPUT_HZ) / u64::from(source_hz);
    (0..total)
        .map(|i| frames[(i * u64::from(source_hz) / u64::from(OUTPUT_HZ)) as usize])
        .collect()
}

pub fn xa_sample_rate(coding: u8) -> u32 {
    if coding & 0x04 == 0 { 37800 } else { 18900 }
}

pub fn xa_is_stereo(coding: u8) -> bool {
    coding & 0x03 == 1
}

pub fn is_xa_audio_sector(raw: &[u8]) -> bool {
    raw.len() >= 0x18 && raw[0x0F] == 0x02 && raw[0x12] & 0x04 != 0
}
