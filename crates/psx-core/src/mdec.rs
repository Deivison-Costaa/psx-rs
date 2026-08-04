use std::cell::{Cell, RefCell};

// § zagzig[0..63] (reversed zigzag) (L341-343) de docs/reference/09-mdec.md, pre-calculado
// a partir de zigzag[0..63] (L320-329) por "zagzig[zigzag[i]]=i" para nao precisar de
// inicializacao em runtime.
const ZAGZIG: [u8; 64] = [
    0, 1, 8, 16, 9, 2, 3, 10, 17, 24, 32, 25, 18, 11, 4, 5, 12, 19, 26, 33, 40, 48, 41, 34, 27, 20,
    13, 6, 7, 14, 21, 28, 35, 42, 49, 56, 57, 50, 43, 36, 29, 22, 15, 23, 30, 37, 44, 51, 58, 59,
    52, 45, 38, 31, 39, 46, 53, 60, 61, 54, 47, 55, 62, 63,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum Command {
    None,
    Decode,
    QuantTable,
    ScaleTable,
}

/// Decodificador de macroblocos (MDEC): regs 1F801820h/1F801824h, tabelas de
/// quantizacao/escala e os caminhos monocromatico (4/8 bit) e colorido (24/15 bit).
/// § MDEC I/O Ports e MDEC Commands (L61-168) de docs/reference/09-mdec.md.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Mdec {
    enable_data_in: bool,
    enable_data_out: bool,
    busy: bool,
    command: Command,
    color_depth: u8,
    output_signed: bool,
    output_bit15: bool,
    quant_color: bool,
    param_bytes: Vec<u8>,
    param_bytes_needed: usize,
    #[serde(with = "crate::serde_grande")]
    iq_y: [u8; 64],
    #[serde(with = "crate::serde_grande")]
    iq_uv: [u8; 64],
    #[serde(with = "crate::serde_grande")]
    scale_table: [i32; 64],
    output: RefCell<Vec<u32>>,
    pending: Vec<u16>,
    words_left: usize,
    block_index: usize,
    #[serde(with = "crate::serde_grande")]
    cr: [i32; 64],
    #[serde(with = "crate::serde_grande")]
    cb: [i32; 64],
    current_block: u8,
    dma_pos: Cell<usize>,
}

impl Mdec {
    pub fn new() -> Self {
        Mdec {
            enable_data_in: false,
            enable_data_out: false,
            busy: false,
            command: Command::None,
            color_depth: 0,
            output_signed: false,
            output_bit15: false,
            quant_color: false,
            param_bytes: Vec::new(),
            param_bytes_needed: 0,
            iq_y: [0; 64],
            iq_uv: [0; 64],
            scale_table: [0; 64],
            output: RefCell::new(Vec::new()),
            pending: Vec::new(),
            words_left: 0,
            block_index: 0,
            cr: [0; 64],
            cb: [0; 64],
            current_block: 4,
            dma_pos: Cell::new(0),
        }
    }

    /// Palavras disponiveis na fifo de saida (usado pelo DMA1 para nao pedir
    /// mais do que o MDEC realmente decodificou).
    pub fn output_len(&self) -> usize {
        self.output.borrow().len()
    }

    pub fn read32(&self, offset: u32) -> u32 {
        match offset {
            0 => self.pop_output(),
            _ => self.status(),
        }
    }

    pub fn write32(&mut self, offset: u32, val: u32) {
        match offset {
            0 => self.write_data(val),
            _ => self.write_control(val),
        }
    }

    fn pop_output(&self) -> u32 {
        let mut out = self.output.borrow_mut();
        if out.is_empty() {
            0 // § MDEC0.Read (L72): "Garbage if there's no data available"
        } else {
            out.remove(0)
        }
    }

    /// Bytes por pixel na saida, ou `None` no caminho monocromatico (que nao tem macrobloco).
    fn bytes_por_pixel(&self) -> Option<usize> {
        match self.color_depth {
            2 => Some(3),
            3 => Some(2),
            _ => None,
        }
    }

    /// Palavras de um macrobloco 16x16 na profundidade atual (128 em 15bpp, 192 em 24bpp).
    pub fn palavras_por_macrobloco(&self) -> Option<usize> {
        self.bytes_por_pixel().map(|b| 256 * b / 4)
    }

    /// Leitura pelo DMA1. § MDEC Data/Response Register (L74-78) de docs/reference/09-mdec.md:
    /// o registrador entrega quatro bitmaps 8x8 em fila e "usually, the data is received via
    /// DMA1, which is doing the re-ordering automatically" — isto e, o DMA1 costura os quatro
    /// blocos no macrobloco 16x16 de § Colored Macroblocks (L376-388).
    pub fn read32_dma(&self) -> u32 {
        let Some(bpp) = self.bytes_por_pixel() else {
            return self.pop_output();
        };
        let out = self.output.borrow();
        let pos = self.dma_pos.get();
        let mut palavra = [0u8; 4];
        for (k, destino) in palavra.iter_mut().enumerate() {
            let i = pos * 4 + k;
            let (pixel, canal) = (i / bpp, i % bpp);
            let (x, y) = (pixel % 16, pixel / 16);
            let quad = usize::from(x >= 8) + 2 * usize::from(y >= 8);
            let fonte = (quad * 64 + (y % 8) * 8 + (x % 8)) * bpp + canal;
            *destino = out
                .get(fonte / 4)
                .map(|w| (w >> (8 * (fonte % 4))) as u8)
                .unwrap_or(0);
        }
        drop(out);
        let total = 256 * bpp / 4;
        if pos + 1 >= total {
            let mut out = self.output.borrow_mut();
            let n = total.min(out.len());
            out.drain(..n);
            self.dma_pos.set(0);
        } else {
            self.dma_pos.set(pos + 1);
        }
        u32::from_le_bytes(palavra)
    }

    // § 1F801824h.Write - MDEC1 Control/Reset (L101-112) de docs/reference/09-mdec.md.
    fn write_control(&mut self, val: u32) {
        if val & (1 << 31) != 0 {
            self.reset();
        }
        self.enable_data_in = val & (1 << 30) != 0;
        self.enable_data_out = val & (1 << 29) != 0;
    }

    fn reset(&mut self) {
        self.busy = false;
        self.command = Command::None;
        self.param_bytes.clear();
        self.param_bytes_needed = 0;
        self.output.borrow_mut().clear();
        self.pending.clear();
        self.words_left = 0;
        self.block_index = 0;
        self.current_block = 4;
        self.dma_pos.set(0);
    }

    // § 1F801824h.Read - MDEC1 Status (L80-99) de docs/reference/09-mdec.md.
    fn status(&self) -> u32 {
        let mut s: u32 = 0;
        if self.output.borrow().is_empty() {
            s |= 1 << 31;
        }
        if self.busy {
            s |= 1 << 29;
        }
        if self.enable_data_in {
            s |= 1 << 28;
        }
        if self.enable_data_out && !self.output.borrow().is_empty() {
            s |= 1 << 27;
        }
        s |= (self.color_depth as u32 & 0x3) << 25;
        if self.output_signed {
            s |= 1 << 24;
        }
        if self.output_bit15 {
            s |= 1 << 23;
        }
        s |= (self.current_block as u32 & 0x7) << 16;
        let remaining_words = if self.command == Command::Decode {
            self.words_left
        } else {
            self.param_bytes_needed
                .saturating_sub(self.param_bytes.len())
                / 4
        };
        if self.busy && remaining_words > 0 {
            s |= (remaining_words as u32 - 1) & 0xFFFF;
        } else {
            s |= 0xFFFF;
        }
        s
    }

    fn write_data(&mut self, val: u32) {
        if !self.busy {
            self.dispatch_command(val);
        } else if self.command == Command::Decode {
            self.feed_decode(val);
        } else {
            self.param_bytes.extend_from_slice(&val.to_le_bytes());
            if self.param_bytes.len() >= self.param_bytes_needed {
                self.finish_command();
            }
        }
    }

    // § MDEC Commands (L128-167) de docs/reference/09-mdec.md.
    fn dispatch_command(&mut self, word: u32) {
        let cmd = (word >> 29) & 0x7;
        self.param_bytes.clear();
        match cmd {
            1 => {
                self.color_depth = ((word >> 27) & 0x3) as u8;
                self.output_signed = word & (1 << 26) != 0;
                self.output_bit15 = word & (1 << 25) != 0;
                let words = (word & 0xFFFF) as usize;
                self.param_bytes_needed = words * 4;
                self.words_left = words;
                self.pending.clear();
                self.block_index = 0;
                self.current_block = 4;
                if words == 0 {
                    self.command = Command::None;
                    self.busy = false;
                } else {
                    self.command = Command::Decode;
                    self.busy = true;
                }
            }
            2 => {
                self.quant_color = word & 1 != 0;
                self.param_bytes_needed = if self.quant_color { 128 } else { 64 };
                self.command = Command::QuantTable;
                self.busy = true;
            }
            3 => {
                self.param_bytes_needed = 128;
                self.command = Command::ScaleTable;
                self.busy = true;
            }
            _ => {
                self.command = Command::None;
                self.busy = false;
            }
        }
    }

    fn finish_command(&mut self) {
        match self.command {
            Command::Decode => {}
            Command::QuantTable => {
                self.iq_y.copy_from_slice(&self.param_bytes[0..64]);
                if self.quant_color {
                    self.iq_uv.copy_from_slice(&self.param_bytes[64..128]);
                }
            }
            Command::ScaleTable => {
                for i in 0..64 {
                    let lo = self.param_bytes[i * 2];
                    let hi = self.param_bytes[i * 2 + 1];
                    self.scale_table[i] = i16::from_le_bytes([lo, hi]) as i32;
                }
            }
            Command::None => {}
        }
        self.command = Command::None;
        self.busy = false;
        self.param_bytes.clear();
    }

    // O MDEC decodifica enquanto os dados chegam, nao no fim do comando: o gabarito de
    // mdec/step-by-step-log ja mostra a fifo de saida cheia com 32 das 256 palavras
    // entregues. Cada bloco 8x8 sai assim que suas meias-palavras fecham.
    fn feed_decode(&mut self, word: u32) {
        self.pending.push(word as u16);
        self.pending.push((word >> 16) as u16);
        self.words_left = self.words_left.saturating_sub(1);
        while self.try_one_block(false) {}
        if self.words_left == 0 {
            // § Run-Length compressed Blocks (L464-466): o EOB e dispensavel se o bloco ja
            // esta definido ate blk[63]. So o caminho monocromatico chega ao fim do comando
            // com bloco em aberto (mdec/4bit e mdec/8bit, iter 0174); em cor, macrobloco
            // incompleto nao sai — e o que o console faz nas 256 palavras do gabarito.
            if self.color_depth < 2 {
                self.try_one_block(true);
            }
            self.busy = false;
            self.command = Command::None;
            self.pending.clear();
        }
    }

    fn quant_do_bloco(&self) -> &[u8; 64] {
        if self.color_depth >= 2 && self.block_index < 2 {
            &self.iq_uv
        } else {
            &self.iq_y
        }
    }

    fn try_one_block(&mut self, truncar: bool) -> bool {
        let Some((block, usados)) =
            Self::rl_decode_block(&self.pending, self.quant_do_bloco(), truncar)
        else {
            return false;
        };
        self.pending.drain(..usados);
        let spatial = Self::idct_core(&block, &self.scale_table);
        if self.color_depth >= 2 {
            self.push_color_block(&spatial);
        } else {
            self.current_block = 4;
            let mut out = self.output.borrow_mut();
            // § MDEC(1) bit26 "Data Output Signed" (L133): 0=unsigned, 1=signed.
            // y_to_mono espera o inverso (L293: "if unsigned then Y=Y xor 80h").
            Self::push_mono_block(&spatial, self.color_depth, !self.output_signed, &mut out);
        }
        true
    }

    // § decode_colored_macroblock (L172-180): Cr, Cb e quatro Y; cada Y vira um quadrante
    // 8x8 do macrobloco 16x16 via yuv_to_rgb.
    fn push_color_block(&mut self, spatial: &[i32; 64]) {
        match self.block_index {
            0 => {
                self.cr = *spatial;
                self.current_block = 4;
            }
            1 => {
                self.cb = *spatial;
                self.current_block = 5;
            }
            _ => {
                let quadrante = self.block_index - 2;
                self.current_block = quadrante as u8;
                self.yuv_to_rgb(
                    spatial,
                    (quadrante as i32 & 1) * 8,
                    (quadrante as i32 >> 1) * 8,
                );
            }
        }
        self.block_index = (self.block_index + 1) % 6;
    }

    // § yuv_to_rgb(xx,yy) (L269-283) de docs/reference/09-mdec.md. A spec (L284-285) diz que a
    // resolucao de ponto fixo exata do hardware e desconhecida; os coeficientes abaixo foram
    // fixados contra o gabarito palavra a palavra de mdec/step-by-step-log (R1).
    fn yuv_to_rgb(&self, y_blk: &[i32; 64], xx: i32, yy: i32) {
        let unsigned = !self.output_signed;
        let mut out = self.output.borrow_mut();
        let mut bytes24: Vec<u8> = Vec::new();
        for y in 0..8i32 {
            for x in 0..8i32 {
                let c = (((x + xx) / 2) + ((y + yy) / 2) * 8) as usize;
                let (cr, cb) = (self.cr[c], self.cb[c]);
                let luma = y_blk[(x + y * 8) as usize];
                let r = Self::satura8(luma + (1.402 * cr as f64).floor() as i32);
                let gg = Self::satura8(
                    luma + ((-0.3437 * cb as f64) + (-0.7143 * cr as f64)).floor() as i32,
                );
                let b = Self::satura8(luma + (1.772 * cb as f64).floor() as i32);
                let (r, gg, b) = if unsigned {
                    ((r ^ 0x80), (gg ^ 0x80), (b ^ 0x80))
                } else {
                    (r, gg, b)
                };
                if self.color_depth == 2 {
                    bytes24.extend_from_slice(&[r, gg, b]);
                } else {
                    let mut px = Self::para5(r) | (Self::para5(gg) << 5) | (Self::para5(b) << 10);
                    if self.output_bit15 {
                        px |= 1 << 15;
                    }
                    if (x + y * 8) % 2 == 0 {
                        out.push(px);
                    } else {
                        let ultimo = out.len() - 1;
                        out[ultimo] |= px << 16;
                    }
                }
            }
        }
        for chunk in bytes24.chunks(4) {
            let mut w = [0u8; 4];
            w[..chunk.len()].copy_from_slice(chunk);
            out.push(u32::from_le_bytes(w));
        }
    }

    fn satura8(v: i32) -> u8 {
        v.clamp(-128, 127) as i8 as u8
    }

    // A reducao de 8 para 5 bits do modo 15bpp arredonda para o mais proximo, nao trunca: com
    // `>>3` puro, 774 de 3072 canais do gabarito de mdec/step-by-step-log saem exatamente um
    // passo abaixo do console, e nenhum acima.
    fn para5(v: u8) -> u32 {
        (((v as u32) + 4) >> 3).min(31)
    }

    // Mesmo arredondamento na reducao para 4 bits: com `>>4` puro, 16 dos 32 bytes do
    // gabarito de mdec/4bit saem um passo abaixo; com arredondamento, 2.
    fn para4(v: u8) -> u8 {
        ((v as u16 + 8) >> 4).min(15) as u8
    }

    fn signed10(n: u16) -> i32 {
        let v = (n & 0x3FF) as i32;
        if v & 0x200 != 0 { v - 0x400 } else { v }
    }

    // § rl_decode_block(blk,src,qt) (L187-206) de docs/reference/09-mdec.md. Devolve o bloco
    // e quantas meias-palavras consumiu, ou `None` quando os dados acabam no meio: nesse caso
    // nada e consumido e o bloco espera o resto do fluxo. Com `truncar`, fecha com o que ja
    // foi gravado (fim de comando no caminho monocromatico).
    fn rl_decode_block(data: &[u16], qt: &[u8; 64], truncar: bool) -> Option<([i32; 64], usize)> {
        let mut blk = [0i32; 64];
        let mut pos = 0usize;
        let mut n;
        loop {
            n = *data.get(pos)?;
            pos += 1;
            if n != 0xFE00 {
                break;
            }
        }
        let q_scale = ((n >> 10) & 0x3F) as i32;
        let mut val = Self::signed10(n) * qt[0] as i32;
        let mut k: usize = 0;
        loop {
            if q_scale == 0 {
                val = Self::signed10(n) * 2;
            }
            val = val.clamp(-0x400, 0x3FF);
            if q_scale != 0 {
                blk[ZAGZIG[k] as usize] = val;
            } else {
                blk[k] = val;
            }
            if k == 63 {
                break;
            }
            n = match data.get(pos) {
                Some(&w) => {
                    pos += 1;
                    w
                }
                None => return if truncar { Some((blk, pos)) } else { None },
            };
            let skip = ((n >> 10) & 0x3F) as usize;
            k += skip + 1;
            if k > 63 {
                break;
            }
            let qk = qt[k] as i32;
            val = (Self::signed10(n) * qk * q_scale + 4) >> 3;
        }
        Some((blk, pos))
    }

    // § real_idct_core(blk) (L241-267) de docs/reference/09-mdec.md. A propria spec
    // (L262-264) admite que o arredondamento exato do hardware nao e conhecido
    // ("the results aren't perfect") — registrado no doc da iteracao 0174.
    fn idct_core(blk: &[i32; 64], scale: &[i32; 64]) -> [i32; 64] {
        let mut src = *blk;
        for _pass in 0..2 {
            let mut dst = [0i32; 64];
            for x in 0..8 {
                for y in 0..8 {
                    let mut sum: i64 = 0;
                    for z in 0..8 {
                        sum += src[y + z * 8] as i64 * (scale[x + z * 8] as i64 / 8);
                    }
                    dst[x + y * 8] = ((sum + 0xFFF) >> 13) as i32;
                }
            }
            src = dst;
        }
        src
    }

    // § y_to_mono (L287-296) de docs/reference/09-mdec.md.
    fn y_to_mono(y: i32, unsigned: bool) -> u8 {
        let mut v = y & 0x1FF;
        if v & 0x100 != 0 {
            v -= 0x200;
        }
        v = v.clamp(-128, 127);
        if unsigned {
            v ^= 0x80;
        }
        (v & 0xFF) as u8
    }

    // Empacotamento de 4 bits: a spec (§ Monochrome Macroblocks L395-408) so diz
    // que a saida e um bitmap 8x8, sem detalhar a ordem dos nibbles — usado o
    // gabarito de hardware como oraculo (R1): mdec/4bit vs mdec/8bit psx.log
    // mostram nibble = y_to_mono>>4, pixel par no nibble baixo, impar no alto.
    fn push_mono_block(spatial: &[i32; 64], depth: u8, unsigned_out: bool, out: &mut Vec<u32>) {
        let pixels: Vec<u8> = spatial
            .iter()
            .map(|&v| Self::y_to_mono(v, unsigned_out))
            .collect();
        let bytes: Vec<u8> = if depth == 0 {
            pixels
                .chunks(2)
                .map(|c| Self::para4(c[0]) | (Self::para4(c[1]) << 4))
                .collect()
        } else {
            pixels
        };
        for chunk in bytes.chunks(4) {
            let mut word_bytes = [0u8; 4];
            word_bytes[..chunk.len()].copy_from_slice(chunk);
            out.push(u32::from_le_bytes(word_bytes));
        }
    }
}

impl Default for Mdec {
    fn default() -> Self {
        Self::new()
    }
}
