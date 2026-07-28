use std::cell::Cell;

const VRAM_W: usize = 1024;
const VRAM_H: usize = 512;

fn vram_pos(x: u16, y: u16) -> usize {
    (x as usize % VRAM_W) + (y as usize % VRAM_H) * VRAM_W
}

#[derive(Debug)]
pub struct Gpu {
    pub stat: u32,
    dma_direction: u8,
    vram: Box<[u16]>,

    active_cmd: u8,
    cmd_params: [u32; 4],
    cmd_param_count: u8,
    cmd_data_phase: bool,

    copy_start_x: u16,
    copy_start_y: u16,
    copy_width: u16,
    copy_height: u16,
    copy_hw_total: u32,
    copy_hw_written: u32,

    readout_buf: Vec<u16>,
    readout_idx: Cell<usize>,
}

impl Default for Gpu {
    fn default() -> Self {
        Self::new()
    }
}

impl Gpu {
    pub fn new() -> Self {
        Gpu {
            stat: 0x1480_2000,
            dma_direction: 0,
            vram: vec![0u16; VRAM_W * VRAM_H].into_boxed_slice(),
            active_cmd: 0,
            cmd_params: [0u32; 4],
            cmd_param_count: 0,
            cmd_data_phase: false,
            copy_start_x: 0,
            copy_start_y: 0,
            copy_width: 0,
            copy_height: 0,
            copy_hw_total: 0,
            copy_hw_written: 0,
            readout_buf: Vec::new(),
            readout_idx: Cell::new(0),
        }
    }

    pub fn vram_u16(&self, x: u16, y: u16) -> u16 {
        self.vram[vram_pos(x, y)]
    }

    pub fn read32(&self, offset: u32) -> u32 {
        match offset {
            0x0 => self.read_gpuread(),
            0x4 => {
                let bit25 = match self.dma_direction {
                    0 => 0,
                    1 => 1 << 25,
                    2 => ((self.stat >> 28) & 1) << 25,
                    3 => ((self.stat >> 27) & 1) << 25,
                    _ => 0,
                };
                (self.stat & !(1 << 25)) | bit25
            }
            _ => 0,
        }
    }

    fn read_gpuread(&self) -> u32 {
        let idx = self.readout_idx.get();
        if idx < self.readout_buf.len() {
            let lo = self.readout_buf[idx] as u32;
            let hi = if idx + 1 < self.readout_buf.len() {
                self.readout_buf[idx + 1] as u32
            } else {
                0
            };
            self.readout_idx.set(idx + 2);
            lo | (hi << 16)
        } else {
            0
        }
    }

    pub fn write32(&mut self, offset: u32, val: u32) {
        match offset {
            0x0 => self.write_gp0(val),
            0x4 => self.write_gp1(val),
            _ => {}
        }
    }

    fn write_gp0(&mut self, val: u32) {
        if self.cmd_data_phase {
            self.receive_data(val);
            return;
        }
        if self.active_cmd != 0 {
            self.cmd_params[self.cmd_param_count as usize] = val;
            self.cmd_param_count += 1;
            self.advance_command();
            return;
        }

        let cmd = (val >> 24) as u8;
        match cmd {
            0x02 => {
                self.active_cmd = 1;
                self.cmd_params[0] = val;
                self.cmd_param_count = 1;
            }
            0xA0 => {
                self.active_cmd = 2;
                self.cmd_param_count = 0;
            }
            0xC0 => {
                self.active_cmd = 3;
                self.cmd_param_count = 0;
            }
            0xE6 => {
                let param = val & 0xFF_FFFF;
                let mask = (1 << 11) | (1 << 12);
                let bits = (param & 0x3) << 11;
                self.stat = (self.stat & !mask) | bits;
            }
            0xE1 => {
                let param = val & 0xFF_FFFF;
                let mask = (0x7FF) | (1 << 15);
                let bits = (param & 0x7FF) | (((param >> 11) & 1) << 15);
                self.stat = (self.stat & !mask) | bits;
            }
            0x00 | 0x04..=0x1E | 0xE0 | 0xE7..=0xEF => {}
            _ => {}
        }
    }

    fn advance_command(&mut self) {
        match self.active_cmd {
            1 => {
                if self.cmd_param_count >= 3 {
                    self.execute_fill();
                    self.reset_command();
                }
            }
            2 => {
                self.advance_cpu_to_vram();
            }
            3 => self.advance_vram_to_cpu(),
            _ => {}
        }
    }

    fn reset_command(&mut self) {
        self.active_cmd = 0;
        self.cmd_param_count = 0;
        self.cmd_data_phase = false;
        self.copy_hw_written = 0;
        self.copy_hw_total = 0;
    }

    fn execute_fill(&mut self) {
        let color_word = self.cmd_params[0];
        let r = (color_word & 0xFF) as u8;
        let g = ((color_word >> 8) & 0xFF) as u8;
        let b = ((color_word >> 16) & 0xFF) as u8;
        let pixel = ((r >> 3) as u16 & 0x1F)
            | (((g >> 3) as u16 & 0x1F) << 5)
            | (((b >> 3) as u16 & 0x1F) << 10);

        let pos = self.cmd_params[1];
        let xpos_raw = (pos & 0xFFFF) as u16;
        let ypos_raw = ((pos >> 16) & 0xFFFF) as u16;
        let xpos = xpos_raw & 0x3F0;
        let ypos = ypos_raw & 0x1FF;

        let size = self.cmd_params[2];
        let xsiz_raw = (size & 0xFFFF) as u16;
        let ysiz_raw = ((size >> 16) & 0xFFFF) as u16;
        let xsiz_masked = (xsiz_raw as u32 & 0x3FF) as u16;
        let ysiz = ysiz_raw & 0x1FF;
        let xsiz = ((xsiz_masked as u32 + 0xF) & !0xF) as u16;

        if xsiz == 0 || ysiz == 0 {
            return;
        }

        for dy in 0..ysiz {
            for dx in 0..xsiz {
                let px = xpos.wrapping_add(dx) % (VRAM_W as u16);
                let py = ypos.wrapping_add(dy) % (VRAM_H as u16);
                self.vram[vram_pos(px, py)] = pixel;
            }
        }
    }

    fn advance_cpu_to_vram(&mut self) {
        if self.cmd_param_count < 2 {
            return;
        }
        let dest = self.cmd_params[0];
        let size = self.cmd_params[1];
        let xpos = (dest & 0x3FF) as u16;
        let ypos = ((dest >> 16) & 0x1FF) as u16;
        let xsiz_raw = (size & 0xFFFF) as u16;
        let ysiz_raw = ((size >> 16) & 0xFFFF) as u16;
        let xsiz = ((xsiz_raw as u32).wrapping_sub(1) & 0x3FF) as u16 + 1;
        let ysiz = ((ysiz_raw as u32).wrapping_sub(1) & 0x1FF) as u16 + 1;

        self.copy_start_x = xpos;
        self.copy_start_y = ypos;
        self.copy_width = xsiz;
        self.copy_height = ysiz;
        self.copy_hw_total = xsiz as u32 * ysiz as u32;
        self.copy_hw_written = 0;
        self.cmd_data_phase = true;
    }

    fn advance_vram_to_cpu(&mut self) {
        if self.cmd_param_count < 2 {
            return;
        }
        let src = self.cmd_params[0];
        let size = self.cmd_params[1];
        let xpos = (src & 0x3FF) as u16;
        let ypos = ((src >> 16) & 0x1FF) as u16;
        let xsiz_raw = (size & 0xFFFF) as u16;
        let ysiz_raw = ((size >> 16) & 0xFFFF) as u16;
        let xsiz = ((xsiz_raw as u32).wrapping_sub(1) & 0x3FF) as u16 + 1;
        let ysiz = ((ysiz_raw as u32).wrapping_sub(1) & 0x1FF) as u16 + 1;

        let hw_total = xsiz as usize * ysiz as usize;
        self.readout_buf.clear();
        for dy in 0..ysiz {
            for dx in 0..xsiz {
                let px = xpos.wrapping_add(dx) % (VRAM_W as u16);
                let py = ypos.wrapping_add(dy) % (VRAM_H as u16);
                self.readout_buf.push(self.vram[vram_pos(px, py)]);
            }
        }
        if hw_total % 2 != 0 {
            self.readout_buf.push(0);
        }
        self.readout_idx.set(0);
        self.stat |= 1 << 27;
        self.reset_command();
    }

    fn receive_data(&mut self, val: u32) {
        let lo = val as u16;
        let hi = (val >> 16) as u16;
        let remaining = self.copy_hw_total - self.copy_hw_written;

        if remaining == 0 {
            self.cmd_data_phase = false;
            self.active_cmd = 0;
            return;
        }

        let px = self
            .copy_start_x
            .wrapping_add((self.copy_hw_written % self.copy_width as u32) as u16)
            % (VRAM_W as u16);
        let py = self
            .copy_start_y
            .wrapping_add((self.copy_hw_written / self.copy_width as u32) as u16)
            % (VRAM_H as u16);
        self.vram[vram_pos(px, py)] = lo;
        self.copy_hw_written += 1;

        if self.copy_hw_written >= self.copy_hw_total {
            self.cmd_data_phase = false;
            self.active_cmd = 0;
            if remaining > 1 {
                return;
            }
            return;
        }

        let px2 = self
            .copy_start_x
            .wrapping_add((self.copy_hw_written % self.copy_width as u32) as u16)
            % (VRAM_W as u16);
        let py2 = self
            .copy_start_y
            .wrapping_add((self.copy_hw_written / self.copy_width as u32) as u16)
            % (VRAM_H as u16);
        self.vram[vram_pos(px2, py2)] = hi;
        self.copy_hw_written += 1;

        if self.copy_hw_written >= self.copy_hw_total {
            self.cmd_data_phase = false;
            self.active_cmd = 0;
        }
    }

    fn write_gp1(&mut self, val: u32) {
        let cmd = (val >> 24) as u8;
        match cmd {
            0x00 => {
                self.stat = 0x1480_2000;
                self.dma_direction = 0;
                self.reset_command();
                self.readout_buf.clear();
                self.readout_idx.set(0);
            }
            0x01 => {}
            0x02 => {
                self.stat &= !(1 << 24);
            }
            0x03 => {
                let bit = val & 1;
                if bit == 0 {
                    self.stat &= !(1 << 23);
                } else {
                    self.stat |= 1 << 23;
                }
            }
            0x04 => {
                self.dma_direction = (val & 0x3) as u8;
                self.stat = (self.stat & !(3 << 29)) | ((val & 0x3) << 29);
            }
            0x08 => {
                let param = val & 0xFF;
                let bits: u32 = (param & 0x80) << 7 | (param & 0x40) << 10 | (param & 0x3F) << 17;
                let mask: u32 = (1 << 14) | (0x7F << 16);
                self.stat = (self.stat & !mask) | bits;
            }
            _ => {}
        }
    }
}
