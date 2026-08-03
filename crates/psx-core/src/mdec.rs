use std::cell::RefCell;

// § zagzig[0..63] (reversed zigzag) (L341-343) de docs/reference/09-mdec.md, pre-calculado
// a partir de zigzag[0..63] (L320-329) por "zagzig[zigzag[i]]=i" para nao precisar de
// inicializacao em runtime.
const ZAGZIG: [u8; 64] = [
    0, 1, 8, 16, 9, 2, 3, 10, 17, 24, 32, 25, 18, 11, 4, 5, 12, 19, 26, 33, 40, 48, 41, 34, 27, 20,
    13, 6, 7, 14, 21, 28, 35, 42, 49, 56, 57, 50, 43, 36, 29, 22, 15, 23, 30, 37, 44, 51, 58, 59,
    52, 45, 38, 31, 39, 46, 53, 60, 61, 54, 47, 55, 62, 63,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Command {
    None,
    Decode,
    QuantTable,
    ScaleTable,
}

/// Decodificador de macroblocos (MDEC): regs 1F801820h/1F801824h, tabelas de
/// quantizacao/escala e o caminho de decodificacao monocromatico (4/8 bit).
/// § MDEC I/O Ports e MDEC Commands (L61-168) de docs/reference/09-mdec.md.
#[derive(Debug)]
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
    iq_y: [u8; 64],
    scale_table: [i32; 64],
    output: RefCell<Vec<u32>>,
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
            scale_table: [0; 64],
            output: RefCell::new(Vec::new()),
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
        s |= 4 << 16; // Current Block: so o caminho monocromatico (Y) roda nesta iteracao.
        let remaining_words = self
            .param_bytes_needed
            .saturating_sub(self.param_bytes.len())
            / 4;
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
            Command::Decode => self.run_decode(),
            Command::QuantTable => {
                // A tabela de cor (Cb/Cr, os 64 bytes seguintes quando quant_color)
                // e recebida e descartada: so o caminho monocromatico (iq_y) roda
                // nesta iteracao (ver run_decode).
                self.iq_y.copy_from_slice(&self.param_bytes[0..64]);
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

    // § MDEC(1) monocromatico: decode_monochrome_macroblock (L182-185) repetido
    // enquanto sobrarem meias-palavras no comando (L138-139: "usually all
    // macroblocks... sent at once"). Cor (24/15bpp, yuv_to_rgb) fica para uma
    // proxima iteracao: nenhuma suite deste lote exercita esse caminho (R5).
    fn run_decode(&mut self) {
        if self.color_depth != 0 && self.color_depth != 1 {
            return;
        }
        let halfwords: Vec<u16> = self
            .param_bytes
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        let mut pos = 0usize;
        let mut out = self.output.borrow_mut();
        loop {
            let start = pos;
            let block = match Self::rl_decode_block(&halfwords, &mut pos, &self.iq_y) {
                Some(b) => b,
                None => break,
            };
            if pos == start {
                break;
            }
            let spatial = Self::idct_core(&block, &self.scale_table);
            // § MDEC(1) bit26 "Data Output Signed" (L133): 0=unsigned, 1=signed.
            // y_to_mono espera o inverso (L293: "if unsigned then Y=Y xor 80h").
            Self::push_mono_block(&spatial, self.color_depth, !self.output_signed, &mut out);
            if pos >= halfwords.len() {
                break;
            }
        }
    }

    fn next_halfword(data: &[u16], pos: &mut usize) -> Option<u16> {
        let w = *data.get(*pos)?;
        *pos += 1;
        Some(w)
    }

    fn signed10(n: u16) -> i32 {
        let v = (n & 0x3FF) as i32;
        if v & 0x200 != 0 { v - 0x400 } else { v }
    }

    // § rl_decode_block(blk,src,qt) (L187-206) de docs/reference/09-mdec.md. Quando
    // os dados do comando se esgotam antes do fim natural do bloco (k>63), o bloco
    // termina com o que ja foi gravado — spec L464-466: EOB e dispensavel se o
    // bloco ja estiver definido ate blk[63], caso deste lote (0174).
    fn rl_decode_block(data: &[u16], pos: &mut usize, qt: &[u8; 64]) -> Option<[i32; 64]> {
        let mut blk = [0i32; 64];
        let mut n = Self::next_halfword(data, pos)?;
        while n == 0xFE00 {
            n = Self::next_halfword(data, pos)?;
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
            n = match Self::next_halfword(data, pos) {
                Some(w) => w,
                None => return Some(blk),
            };
            let skip = ((n >> 10) & 0x3F) as usize;
            k += skip + 1;
            if k > 63 {
                break;
            }
            let qk = qt[k] as i32;
            val = (Self::signed10(n) * qk * q_scale + 4) / 8;
        }
        Some(blk)
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
                    dst[x + y * 8] = ((sum + 0xFFF) / 0x2000) as i32;
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
                .map(|c| (c[0] >> 4) | ((c[1] >> 4) << 4))
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
