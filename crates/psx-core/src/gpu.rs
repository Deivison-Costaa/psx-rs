use std::cell::Cell;
use std::fmt;

fn color24_to_16(color24: u32) -> u16 {
    let r = (color24 & 0xFF) as u8;
    let g = ((color24 >> 8) & 0xFF) as u8;
    let b = ((color24 >> 16) & 0xFF) as u8;
    ((r >> 3) as u16) | (((g >> 3) as u16) << 5) | (((b >> 3) as u16) << 10)
}

fn lerp_i32(a: i32, b: i32, t: i32, t_max: i32) -> i32 {
    if t_max == 0 {
        return a;
    }
    a + (b - a) * t / t_max
}

fn lerp_color(a: u16, b: u16, t: i32, t_max: i32) -> u16 {
    if t_max == 0 {
        return a;
    }
    let ar = (a & 0x1F) as i32;
    let ag = ((a >> 5) & 0x1F) as i32;
    let ab = ((a >> 10) & 0x1F) as i32;
    let br = (b & 0x1F) as i32;
    let bg = ((b >> 5) & 0x1F) as i32;
    let bb = ((b >> 10) & 0x1F) as i32;
    let r = (ar + (br - ar) * t / t_max).clamp(0, 31) as u16;
    let g = (ag + (bg - ag) * t / t_max).clamp(0, 31) as u16;
    let b = (ab + (bb - ab) * t / t_max).clamp(0, 31) as u16;
    r | (g << 5) | (b << 10)
}

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
    SkipParams {
        remaining: u8,
    },
    PolygonRender {
        gouraud: bool,
        quad: bool,
        color0: u32,
        vertices: [(i16, i16); 4],
        colors: [u32; 4],
        vertex_count: u8,
        total_vertices: u8,
        awaiting_color: bool,
    },
}

pub struct Gpu {
    stat: Cell<u32>,
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

    pub fn peek32(&self, offset: u32) -> u32 {
        match offset {
            0x0 => self.peek_gpuread(),
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

    pub fn stat(&self) -> u32 {
        self.stat.get()
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
                    4 => {
                        self.stat.set(self.stat.get() & !(1 << 26));
                        VramState::SkipParams { remaining: 3 }
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
                        0x20..=0x3F => {
                            self.stat.set(self.stat.get() & !(1 << 26));
                            let gouraud = (cmd & 0x10) != 0;
                            let quad = (cmd & 0x08) != 0;
                            let total = if quad { 4 } else { 3 };
                            VramState::PolygonRender {
                                gouraud,
                                quad,
                                color0: val & 0x00FF_FFFF,
                                vertices: [(0, 0); 4],
                                colors: [0; 4],
                                vertex_count: 0,
                                total_vertices: total,
                                awaiting_color: false,
                            }
                        }
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
            VramState::SkipParams { remaining } => {
                if remaining <= 1 {
                    self.stat.set(self.stat.get() | (1 << 26));
                    self.vram_state.set(VramState::Idle);
                } else {
                    self.vram_state.set(VramState::SkipParams {
                        remaining: remaining - 1,
                    });
                }
            }
            VramState::Header { cmd, words, count } => {
                let mut w = words;
                w[count as usize] = val;
                let c = count + 1;
                if c >= 3 {
                    self.commit_header(cmd, w);
                } else {
                    self.vram_state.set(VramState::Header {
                        cmd,
                        words: w,
                        count: c,
                    });
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
            VramState::PolygonRender {
                gouraud,
                quad,
                color0,
                mut vertices,
                mut colors,
                mut vertex_count,
                total_vertices,
                mut awaiting_color,
            } => {
                if awaiting_color {
                    colors[vertex_count as usize] = val & 0x00FF_FFFF;
                    awaiting_color = false;
                } else {
                    let x = ((val & 0xFFFF) << 5) as i16 >> 5;
                    let y = (((val >> 16) & 0xFFFF) << 5) as i16 >> 5;
                    if vertex_count == 0 {
                        colors[0] = color0;
                    }
                    vertices[vertex_count as usize] = (x, y);
                    vertex_count += 1;

                    if vertex_count >= total_vertices {
                        if !gouraud {
                            for c in colors.iter_mut().take(total_vertices as usize).skip(1) {
                                *c = color0;
                            }
                        }
                        self.render_polygon(
                            gouraud,
                            quad,
                            &mut vertices,
                            &mut colors,
                            total_vertices,
                        );
                        self.stat.set(self.stat.get() | (1 << 26));
                        self.vram_state.set(VramState::Idle);
                        return;
                    }
                    if gouraud {
                        awaiting_color = true;
                    }
                }
                self.vram_state.set(VramState::PolygonRender {
                    gouraud,
                    quad,
                    color0,
                    vertices,
                    colors,
                    vertex_count,
                    total_vertices,
                    awaiting_color,
                });
            }
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
            for col in 0..xsiz as u16 {
                let px = xpos.wrapping_add(col) & 0x3FF;
                self.vram[py as usize * 1024 + px as usize] = pixel;
            }
        }
    }

    fn render_polygon(
        &mut self,
        gouraud: bool,
        quad: bool,
        vertices: &mut [(i16, i16); 4],
        colors: &mut [u32; 4],
        _total: u8,
    ) {
        let n = if quad { 4 } else { 3 };
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = (vertices[i].0 as i32 - vertices[j].0 as i32).abs();
                let dy = (vertices[i].1 as i32 - vertices[j].1 as i32).abs();
                if dx > 1023 || dy > 511 {
                    return;
                }
            }
        }
        if quad {
            self.render_triangle(
                gouraud,
                [vertices[0], vertices[1], vertices[2]],
                [colors[0], colors[1], colors[2]],
            );
            self.render_triangle(
                gouraud,
                [vertices[1], vertices[2], vertices[3]],
                [colors[1], colors[2], colors[3]],
            );
        } else {
            self.render_triangle(
                gouraud,
                [vertices[0], vertices[1], vertices[2]],
                [colors[0], colors[1], colors[2]],
            );
        }
    }

    fn render_triangle(&mut self, gouraud: bool, verts: [(i16, i16); 3], colors: [u32; 3]) {
        let (v0, v1, v2) = (verts[0], verts[1], verts[2]);
        let (c0, c1, c2) = (colors[0], colors[1], colors[2]);
        let mut verts = [
            (v0.0 as i32, v0.1 as i32, color24_to_16(c0)),
            (v1.0 as i32, v1.1 as i32, color24_to_16(c1)),
            (v2.0 as i32, v2.1 as i32, color24_to_16(c2)),
        ];
        verts.sort_by_key(|v| v.1);

        let (xm, ym, pcm) = verts[1];
        let (xb, yb, pcb) = verts[2];
        let (xt, yt, pct) = verts[0];

        let dy_mt = ym - yt;
        let dy_bt = yb - yt;
        let dy_bm = yb - ym;

        if dy_bt <= 0 {
            return;
        }

        for y in yt.max(0)..=yb.min(511) {
            let x_edge_tb = lerp_i32(xt, xb, y - yt, dy_bt);
            let (x_edge_short, color_short, _dy_short) = if y < ym {
                (
                    lerp_i32(xt, xm, y - yt, dy_mt),
                    lerp_color(pct, pcm, y - yt, dy_mt),
                    dy_mt,
                )
            } else {
                (
                    lerp_i32(xm, xb, y - ym, dy_bm),
                    lerp_color(pcm, pcb, y - ym, dy_bm),
                    dy_bm,
                )
            };

            let color_tb = lerp_color(pct, pcb, y - yt, dy_bt);

            let (xl, xr, cl, cr) = if x_edge_tb < x_edge_short {
                (x_edge_tb, x_edge_short, color_tb, color_short)
            } else {
                (x_edge_short, x_edge_tb, color_short, color_tb)
            };

            let xl = xl.max(0);
            let xr = xr.min(1023);
            if xl > xr {
                continue;
            }

            let dx = xr - xl;
            for x in xl..=xr {
                let pixel = if gouraud && dx > 0 {
                    lerp_color(cl, cr, x - xl, dx)
                } else {
                    pct
                };
                let idx = y as usize * 1024 + x as usize;
                self.vram[idx] = pixel;
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

    fn peek_gpuread(&self) -> u32 {
        let state = self.vram_state.get();
        match state {
            VramState::VramToCpu {
                x,
                y,
                width,
                height,
                remaining,
            } => {
                if remaining == 0 {
                    return 0;
                }

                let total = (width as u32) * (height as u32);
                let processed = total - remaining;

                let col = (processed % width as u32) as u16;
                let row = (processed / width as u32) as u16;
                let px = x.wrapping_add(col) & 0x3FF;
                let py = y.wrapping_add(row) & 0x1FF;
                let hw1 = self.vram[py as usize * 1024 + px as usize];

                let hw2 = if remaining > 1 {
                    let processed2 = processed + 1;
                    let col2 = (processed2 % width as u32) as u16;
                    let row2 = (processed2 / width as u32) as u16;
                    let px2 = x.wrapping_add(col2) & 0x3FF;
                    let py2 = y.wrapping_add(row2) & 0x1FF;
                    self.vram[py2 as usize * 1024 + px2 as usize]
                } else {
                    0
                };

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
                self.vram_state.set(VramState::Idle);
            }
            0x01 => {
                self.vram_state.set(VramState::Idle);
                self.stat.set(self.stat.get() | (1 << 26));
            }
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
                let bits: u32 = (param & 0x80) << 7 | (param & 0x40) << 10 | (param & 0x3F) << 17;
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
            VramState::SkipParams { remaining } => {
                write!(f, "SkipParams(rem={})", remaining)
            }
            VramState::PolygonRender {
                vertex_count,
                total_vertices,
                awaiting_color,
                ..
            } => {
                write!(
                    f,
                    "PolygonRender(v={}/{}, awaiting_color={})",
                    vertex_count, total_vertices, awaiting_color
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
