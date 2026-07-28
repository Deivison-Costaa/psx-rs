use std::cell::Cell;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq)]
enum VramCmd {
    Fill,
    CpuToVram,
    VramToCpu,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum VramState {
    Idle,
    Header {
        cmd: VramCmd,
        words: [u32; 3],
        count: u8,
    },
    CpuToVram {
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        remaining: u32,
    },
    VramToCpu {
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        remaining: u32,
    },
}

pub struct Gpu {
    pub stat: Cell<u32>,
    dma_direction: Cell<u8>,
    vram: Vec<u16>,
    vram_state: Cell<VramState>,
}

impl Default for Gpu {
    fn default() -> Self {
        Self::new()
    }
}

impl Gpu {
    pub fn new() -> Self {
        Gpu {
            stat: Cell::new(0x1480_2000),
            dma_direction: Cell::new(0),
            vram: vec![0u16; 1024 * 512],
            vram_state: Cell::new(VramState::Idle),
        }
    }

    pub fn read32(&self, offset: u32) -> u32 {
        match offset {
            0x0 => self.gpuread_word(),
            0x4 => {
                let dir = self.dma_direction.get();
                let stat = self.stat.get();
                let bit25 = match dir {
                    0 => 0,
                    1 => 1 << 25,
                    2 => ((stat >> 28) & 1) << 25,
                    3 => ((stat >> 27) & 1) << 25,
                    _ => 0,
                };
                (stat & !(1 << 25)) | bit25
            }
            _ => 0,
        }
    }

    pub fn write32(&mut self, offset: u32, val: u32) {
        match offset {
            0x0 => self.write_gp0(val),
            0x4 => self.write_gp1(val),
            _ => {}
        }
    }

    pub fn vram_pixel(&self, x: u16, y: u16) -> u16 {
        let xi = (x & 0x3FF) as usize;
        let yi = (y & 0x1FF) as usize;
        self.vram[yi * 1024 + xi]
    }

    fn write_gp0(&mut self, val: u32) {
        let state = self.vram_state.get();
        match state {
            VramState::Idle => {
                let cmd = (val >> 24) as u8;
                let top3 = (cmd >> 5) as u32;
                self.vram_state.set(match top3 {
                    5 => {
                        self.stat.set(self.stat.get() & !(1 << 26));
                        VramState::Header {
                            cmd: VramCmd::CpuToVram,
                            words: [val, 0, 0],
                            count: 1,
                        }
                    }
                    6 => {
                        self.stat.set(self.stat.get() & !(1 << 26));
                        VramState::Header {
                            cmd: VramCmd::VramToCpu,
                            words: [val, 0, 0],
                            count: 1,
                        }
                    }
                    _ => match cmd {
                        0x02 => {
                            self.stat.set(self.stat.get() & !(1 << 26));
                            VramState::Header {
                                cmd: VramCmd::Fill,
                                words: [val, 0, 0],
                                count: 1,
                            }
                        }
                        0x00 | 0x04..=0x1E | 0xE0 | 0xE7..=0xEF => VramState::Idle,
                        0xE6 => {
                            let param = val & 0xFF_FFFF;
                            let mask = (1 << 11) | (1 << 12);
                            let bits = (param & 0x3) << 11;
                            let s = self.stat.get();
                            self.stat.set((s & !mask) | bits);
                            VramState::Idle
                        }
                        0xE1 => {
                            let param = val & 0xFF_FFFF;
                            let mask = (0x7FF) | (1 << 15);
                            let bits = (param & 0x7FF) | (((param >> 11) & 1) << 15);
                            let s = self.stat.get();
                            self.stat.set((s & !mask) | bits);
                            VramState::Idle
                        }
                        _ => VramState::Idle,
                    },
                });
            }
            VramState::Header { cmd, words, count } => {
                let mut w = words;
                w[count as usize] = val;
                let c = count + 1;
                if c >= 3 {
                    self.commit_header(cmd, w);
                } else {
                    self.vram_state
                        .set(VramState::Header { cmd, words: w, count: c });
                }
            }
            VramState::CpuToVram {
                x,
                y,
                width,
                height,
                mut remaining,
            } => {
                let total = (width as u32) * (height as u32);
                let processed = total - remaining;

                let col = (processed % width as u32) as u16;
                let row = (processed / width as u32) as u16;
                let hw1 = (val & 0xFFFF) as u16;
                let px = x.wrapping_add(col) & 0x3FF;
                let py = y.wrapping_add(row) & 0x1FF;
                self.vram[py as usize * 1024 + px as usize] = hw1;

                remaining = remaining.saturating_sub(1);

                if remaining > 0 {
                    let processed2 = processed + 1;
                    let col2 = (processed2 % width as u32) as u16;
                    let row2 = (processed2 / width as u32) as u16;
                    let hw2 = ((val >> 16) & 0xFFFF) as u16;
                    let px2 = x.wrapping_add(col2) & 0x3FF;
                    let py2 = y.wrapping_add(row2) & 0x1FF;
                    self.vram[py2 as usize * 1024 + px2 as usize] = hw2;
                    remaining = remaining.saturating_sub(1);
                }

                if remaining == 0 {
                    self.stat.set(self.stat.get() | (1 << 26));
                    self.vram_state.set(VramState::Idle);
                } else {
                    self.vram_state.set(VramState::CpuToVram {
                        x,
                        y,
                        width,
                        height,
                        remaining,
                    });
                }
            }
            VramState::VramToCpu { .. } => {}
        }
    }

    fn commit_header(&mut self, cmd: VramCmd, words: [u32; 3]) {
        match cmd {
            VramCmd::Fill => {
                self.execute_fill(words[0], words[1], words[2]);
                self.stat.set(self.stat.get() | (1 << 26));
                self.vram_state.set(VramState::Idle);
            }
            VramCmd::CpuToVram => {
                let pos = words[1];
                let size = words[2];
                let xpos = (pos & 0xFFFF) as u16 & 0x3FF;
                let ypos = ((pos >> 16) & 0xFFFF) as u16 & 0x1FF;
                let xsiz = (((size & 0xFFFF) as u16).wrapping_sub(1) & 0x3FF) + 1;
                let ysiz = ((((size >> 16) & 0xFFFF) as u16).wrapping_sub(1) & 0x1FF) + 1;
                let total = xsiz as u32 * ysiz as u32;
                self.vram_state.set(VramState::CpuToVram {
                    x: xpos,
                    y: ypos,
                    width: xsiz,
                    height: ysiz,
                    remaining: total,
                });
            }
            VramCmd::VramToCpu => {
                let pos = words[1];
                let size = words[2];
                let xpos = (pos & 0xFFFF) as u16 & 0x3FF;
                let ypos = ((pos >> 16) & 0xFFFF) as u16 & 0x1FF;
                let xsiz = (((size & 0xFFFF) as u16).wrapping_sub(1) & 0x3FF) + 1;
                let ysiz = ((((size >> 16) & 0xFFFF) as u16).wrapping_sub(1) & 0x1FF) + 1;
                let total = xsiz as u32 * ysiz as u32;
                self.stat.set(self.stat.get() | (1 << 27));
                self.vram_state.set(VramState::VramToCpu {
                    x: xpos,
                    y: ypos,
                    width: xsiz,
                    height: ysiz,
                    remaining: total,
                });
            }
        }
    }

    fn execute_fill(&mut self, color_word: u32, pos_word: u32, size_word: u32) {
        let r = color_word & 0xFF;
        let g = (color_word >> 8) & 0xFF;
        let b = (color_word >> 16) & 0xFF;
        let pixel = ((r >> 3) | ((g >> 3) << 5) | ((b >> 3) << 10)) as u16;

        let raw_x = (pos_word & 0xFFFF) as u16;
        let raw_y = ((pos_word >> 16) & 0xFFFF) as u16;
        let xpos = raw_x & 0x3F0;
        let ypos = raw_y & 0x1FF;

        let raw_w = (size_word & 0xFFFF) as u16;
        let raw_h = ((size_word >> 16) & 0xFFFF) as u16;
        let xsiz = ((raw_w as u32 & 0x3FF) + 0x0F) & !0x0F;
        let ysiz = raw_h as u32 & 0x1FF;

        if xsiz == 0 || ysiz == 0 {
            return;
        }

        for row in 0..ysiz as u16 {
            let py = ypos.wrapping_add(row) & 0x1FF;
            for col in (0..xsiz as u16).step_by(1) {
                let px = xpos.wrapping_add(col) & 0x3FF;
                self.vram[py as usize * 1024 + px as usize] = pixel;
            }
        }
    }

    fn gpuread_word(&self) -> u32 {
        let state = self.vram_state.get();
        match state {
            VramState::VramToCpu {
                x,
                y,
                width,
                height,
                mut remaining,
            } => {
                if remaining == 0 {
                    let mut stat = self.stat.get();
                    stat &= !(1 << 27);
                    stat |= 1 << 26;
                    self.stat.set(stat);
                    self.vram_state.set(VramState::Idle);
                    return 0;
                }

                let total = (width as u32) * (height as u32);
                let processed = total - remaining;

                let col = (processed % width as u32) as u16;
                let row = (processed / width as u32) as u16;
                let px = x.wrapping_add(col) & 0x3FF;
                let py = y.wrapping_add(row) & 0x1FF;
                let hw1 = self.vram[py as usize * 1024 + px as usize];

                remaining = remaining.saturating_sub(1);

                let hw2 = if remaining > 0 {
                    let processed2 = processed + 1;
                    let col2 = (processed2 % width as u32) as u16;
                    let row2 = (processed2 / width as u32) as u16;
                    let px2 = x.wrapping_add(col2) & 0x3FF;
                    let py2 = y.wrapping_add(row2) & 0x1FF;
                    remaining = remaining.saturating_sub(1);
                    self.vram[py2 as usize * 1024 + px2 as usize]
                } else {
                    remaining = 0;
                    0
                };

                if remaining == 0 {
                    let mut stat = self.stat.get();
                    stat &= !(1 << 27);
                    stat |= 1 << 26;
                    self.stat.set(stat);
                    self.vram_state.set(VramState::Idle);
                } else {
                    self.vram_state.set(VramState::VramToCpu {
                        x,
                        y,
                        width,
                        height,
                        remaining,
                    });
                }

                (hw1 as u32) | ((hw2 as u32) << 16)
            }
            _ => 0,
        }
    }

    fn write_gp1(&mut self, val: u32) {
        let cmd = (val >> 24) as u8;
        match cmd {
            0x00 => {
                self.stat.set(0x1480_2000);
                self.dma_direction.set(0);
                self.vram.fill(0);
                self.vram_state.set(VramState::Idle);
            }
            0x01 => {}
            0x02 => {
                let s = self.stat.get();
                self.stat.set(s & !(1 << 24));
            }
            0x03 => {
                let bit = val & 1;
                let s = self.stat.get();
                if bit == 0 {
                    self.stat.set(s & !(1 << 23));
                } else {
                    self.stat.set(s | (1 << 23));
                }
            }
            0x04 => {
                self.dma_direction.set((val & 0x3) as u8);
                let s = self.stat.get();
                self.stat.set((s & !(3 << 29)) | ((val & 0x3) << 29));
            }
            0x08 => {
                let param = val & 0xFF;
                let bits: u32 =
                    (param & 0x80) << 7 | (param & 0x40) << 10 | (param & 0x3F) << 17;
                let mask: u32 = (1 << 14) | (0x7F << 16);
                let s = self.stat.get();
                self.stat.set((s & !mask) | bits);
            }
            _ => {}
        }
    }
}

impl fmt::Debug for Gpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Gpu")
            .field("stat", &self.stat.get())
            .field("dma_direction", &self.dma_direction.get())
            .field("vram_state", &self.vram_state.get())
            .field("vram", &format_args!("[{} halfwords]", self.vram.len()))
            .finish()
    }
}

impl fmt::Debug for VramState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VramState::Idle => write!(f, "Idle"),
            VramState::Header { cmd, count, .. } => {
                let name = match cmd {
                    VramCmd::Fill => "Fill",
                    VramCmd::CpuToVram => "CpuToVram",
                    VramCmd::VramToCpu => "VramToCpu",
                };
                write!(f, "Header({}, count={})", name, count)
            }
            VramState::CpuToVram {
                x,
                y,
                width,
                height,
                remaining,
            } => {
                write!(
                    f,
                    "CpuToVram(pos=({},{}), size={}x{}, rem={})",
                    x, y, width, height, remaining
                )
            }
            VramState::VramToCpu {
                x,
                y,
                width,
                height,
                remaining,
            } => {
                write!(
                    f,
                    "VramToCpu(pos=({},{}), size={}x{}, rem={})",
                    x, y, width, height, remaining
                )
            }
        }
    }
}

impl fmt::Debug for VramCmd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VramCmd::Fill => write!(f, "Fill"),
            VramCmd::CpuToVram => write!(f, "CpuToVram"),
            VramCmd::VramToCpu => write!(f, "VramToCpu"),
        }
    }
}
