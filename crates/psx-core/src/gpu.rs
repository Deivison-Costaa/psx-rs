use std::cell::Cell;
use std::fmt;

fn color24_to_16(color24: u32) -> u16 {
    let r = (color24 & 0xFF) as u8;
    let g = ((color24 >> 8) & 0xFF) as u8;
    let b = ((color24 >> 16) & 0xFF) as u8;
    ((r >> 3) as u16) | (((g >> 3) as u16) << 5) | (((b >> 3) as u16) << 10)
}

const DITHER_MATRIX: [[i32; 4]; 4] = [
    [-4, 0, -3, 1],
    [2, -2, 3, -1],
    [-3, 1, -4, 0],
    [3, -1, 2, -2],
];

fn color24_to_16_dithered(color24: u32, x: i32, y: i32, dither_enabled: bool) -> u16 {
    let r = (color24 & 0xFF) as u8;
    let g = ((color24 >> 8) & 0xFF) as u8;
    let b = ((color24 >> 16) & 0xFF) as u8;
    if dither_enabled {
        let offset = DITHER_MATRIX[(y & 3) as usize][(x & 3) as usize];
        let r_d = ((r as i32) + offset).clamp(0, 255) as u16;
        let g_d = ((g as i32) + offset).clamp(0, 255) as u16;
        let b_d = ((b as i32) + offset).clamp(0, 255) as u16;
        (r_d >> 3) | ((g_d >> 3) << 5) | ((b_d >> 3) << 10)
    } else {
        ((r >> 3) as u16) | (((g >> 3) as u16) << 5) | (((b >> 3) as u16) << 10)
    }
}

fn lerp_color24(a: u32, b: u32, t: i32, t_max: i32) -> u32 {
    if t_max == 0 {
        return a;
    }
    let ar = (a & 0xFF) as i32;
    let ag = ((a >> 8) & 0xFF) as i32;
    let ab = ((a >> 16) & 0xFF) as i32;
    let br = (b & 0xFF) as i32;
    let bg = ((b >> 8) & 0xFF) as i32;
    let bb = ((b >> 16) & 0xFF) as i32;
    let r = (ar + (br - ar) * t / t_max).clamp(0, 255) as u32;
    let g = (ag + (bg - ag) * t / t_max).clamp(0, 255) as u32;
    let b = (ab + (bb - ab) * t / t_max).clamp(0, 255) as u32;
    r | (g << 8) | (b << 16)
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
enum RectStage {
    AwaitVertex,
    AwaitUV,
    AwaitDims,
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
        textured: bool,
        quad: bool,
        semi_transparent: bool,
        color0: u32,
        vertices: [(i16, i16); 4],
        colors: [u32; 4],
        uvs: [(u8, u8); 4],
        vertex_count: u8,
        total_vertices: u8,
        awaiting_color: bool,
        awaiting_uv: bool,
    },
    LineRender {
        gouraud: bool,
        polyline: bool,
        semi_transparent: bool,
        color0: u32,
        vertex0: (i16, i16),
        prev_vertex: (i16, i16),
        prev_color: u32,
        pending_color: u32,
        has_prev: bool,
        vertex_count: u8,
        expecting_vertex: bool,
    },
    RectRender {
        size: u8,
        textured: bool,
        semi_transparent: bool,
        color: u32,
        vertex: (i16, i16),
        uv: u32,
        width: u16,
        height: u16,
        stage: RectStage,
    },
}

pub struct Gpu {
    stat: Cell<u32>,
    dma_direction: Cell<u8>,
    vram: Vec<u16>,
    vram_state: Cell<VramState>,
    drawing_x1: Cell<u16>,
    drawing_y1: Cell<u16>,
    drawing_x2: Cell<u16>,
    drawing_y2: Cell<u16>,
    drawing_offset_x: Cell<i16>,
    drawing_offset_y: Cell<i16>,
    clut_attribute: Cell<u16>,
    tex_window_mask_x: Cell<u8>,
    tex_window_mask_y: Cell<u8>,
    tex_window_offset_x: Cell<u8>,
    tex_window_offset_y: Cell<u8>,
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
            drawing_x1: Cell::new(0),
            drawing_y1: Cell::new(0),
            drawing_x2: Cell::new(1023),
            drawing_y2: Cell::new(511),
            drawing_offset_x: Cell::new(0),
            drawing_offset_y: Cell::new(0),
            clut_attribute: Cell::new(0),
            tex_window_mask_x: Cell::new(0),
            tex_window_mask_y: Cell::new(0),
            tex_window_offset_x: Cell::new(0),
            tex_window_offset_y: Cell::new(0),
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
                        0xE3 => {
                            let x = (val & 0x3FF) as u16;
                            let y = ((val >> 10) & 0x1FF) as u16;
                            self.drawing_x1.set(x);
                            self.drawing_y1.set(y);
                            VramState::Idle
                        }
                        0xE4 => {
                            let x = (val & 0x3FF) as u16;
                            let y = ((val >> 10) & 0x1FF) as u16;
                            self.drawing_x2.set(x);
                            self.drawing_y2.set(y);
                            VramState::Idle
                        }
                        0xE5 => {
                            let off_x = ((val & 0x7FF) << 5) as i16 >> 5;
                            let off_y = (((val >> 11) & 0x7FF) << 5) as i16 >> 5;
                            self.drawing_offset_x.set(off_x);
                            self.drawing_offset_y.set(off_y);
                            VramState::Idle
                        }
                        0x20..=0x3F => {
                            self.stat.set(self.stat.get() & !(1 << 26));
                            let gouraud = (cmd & 0x10) != 0;
                            let textured = (cmd & 0x04) != 0;
                            let quad = (cmd & 0x08) != 0;
                            let semi_transparent = (cmd & 0x02) != 0;
                            let total = if quad { 4 } else { 3 };
                            VramState::PolygonRender {
                                gouraud,
                                textured,
                                quad,
                                semi_transparent,
                                color0: val & 0x00FF_FFFF,
                                vertices: [(0, 0); 4],
                                colors: [0; 4],
                                uvs: [(0, 0); 4],
                                vertex_count: 0,
                                total_vertices: total,
                                awaiting_color: false,
                                awaiting_uv: false,
                            }
                        }
                        0x40..=0x5F => {
                            self.stat.set(self.stat.get() & !(1 << 26));
                            let gouraud = (cmd & 0x10) != 0;
                            let polyline = (cmd & 0x08) != 0;
                            let semi_transparent = (cmd & 0x02) != 0;
                            let color0 = val & 0x00FF_FFFF;
                            VramState::LineRender {
                                gouraud,
                                polyline,
                                semi_transparent,
                                color0,
                                vertex0: (0, 0),
                                prev_vertex: (0, 0),
                                prev_color: color0,
                                pending_color: 0,
                                has_prev: false,
                                vertex_count: 0,
                                expecting_vertex: true,
                            }
                        }
                        0x60..=0x7F => {
                            self.stat.set(self.stat.get() & !(1 << 26));
                            let size = (cmd >> 3) & 0x3;
                            let textured = (cmd & 0x04) != 0;
                            let semi_transparent = (cmd & 0x02) != 0;
                            let color = val & 0x00FF_FFFF;
                            VramState::RectRender {
                                size,
                                textured,
                                semi_transparent,
                                color,
                                vertex: (0, 0),
                                uv: 0,
                                width: 0,
                                height: 0,
                                stage: RectStage::AwaitVertex,
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
                        0xE2 => {
                            let mask_x = (val & 0x1F) as u8;
                            let mask_y = ((val >> 5) & 0x1F) as u8;
                            let offset_x = ((val >> 10) & 0x1F) as u8;
                            let offset_y = ((val >> 15) & 0x1F) as u8;
                            self.tex_window_mask_x.set(mask_x);
                            self.tex_window_mask_y.set(mask_y);
                            self.tex_window_offset_x.set(offset_x);
                            self.tex_window_offset_y.set(offset_y);
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
                textured,
                quad,
                semi_transparent,
                color0,
                mut vertices,
                mut colors,
                mut uvs,
                mut vertex_count,
                total_vertices,
                mut awaiting_color,
                mut awaiting_uv,
            } => {
                if awaiting_color {
                    colors[vertex_count as usize] = val & 0x00FF_FFFF;
                    awaiting_color = false;
                } else if awaiting_uv {
                    awaiting_uv = false;
                    let uv_idx = vertex_count.saturating_sub(1) as usize;
                    uvs[uv_idx] = ((val & 0xFF) as u8, ((val >> 8) & 0xFF) as u8);
                    self.apply_clut_if_first(uv_idx, val);
                    self.apply_texpage_if_second(uv_idx, val);
                    if vertex_count >= total_vertices {
                        self.render_polygon(
                            gouraud,
                            quad,
                            textured,
                            semi_transparent,
                            &mut vertices,
                            &mut colors,
                            &mut uvs,
                        );
                        self.stat.set(self.stat.get() | (1 << 26));
                        self.vram_state.set(VramState::Idle);
                        return;
                    }
                    if gouraud {
                        awaiting_color = true;
                    }
                } else {
                    let raw_x = ((val & 0xFFFF) << 5) as i16 >> 5;
                    let raw_y = (((val >> 16) & 0xFFFF) << 5) as i16 >> 5;
                    let off_x = self.drawing_offset_x.get();
                    let off_y = self.drawing_offset_y.get();
                    let x = raw_x.wrapping_add(off_x);
                    let y = raw_y.wrapping_add(off_y);
                    if vertex_count == 0 {
                        colors[0] = color0;
                    }
                    vertices[vertex_count as usize] = (x, y);
                    vertex_count += 1;

                    if textured {
                        awaiting_uv = true;
                    } else if vertex_count >= total_vertices {
                        self.render_polygon(
                            gouraud,
                            quad,
                            textured,
                            semi_transparent,
                            &mut vertices,
                            &mut colors,
                            &mut uvs,
                        );
                        self.stat.set(self.stat.get() | (1 << 26));
                        self.vram_state.set(VramState::Idle);
                        return;
                    }
                    if gouraud && !textured {
                        awaiting_color = true;
                    }
                }
                self.vram_state.set(VramState::PolygonRender {
                    gouraud,
                    textured,
                    quad,
                    semi_transparent,
                    color0,
                    vertices,
                    colors,
                    uvs,
                    vertex_count,
                    total_vertices,
                    awaiting_color,
                    awaiting_uv,
                });
            }
            VramState::LineRender {
                gouraud,
                polyline,
                semi_transparent,
                color0,
                mut vertex0,
                mut prev_vertex,
                mut prev_color,
                mut pending_color,
                mut has_prev,
                mut vertex_count,
                mut expecting_vertex,
            } => {
                let is_terminator = (val & 0xF000F000) == 0x50005000;
                if expecting_vertex {
                    if polyline && !gouraud && is_terminator {
                        self.stat.set(self.stat.get() | (1 << 26));
                        self.vram_state.set(VramState::Idle);
                        return;
                    }
                    let raw_x = ((val & 0xFFFF) << 5) as i16 >> 5;
                    let raw_y = (((val >> 16) & 0xFFFF) << 5) as i16 >> 5;
                    let off_x = self.drawing_offset_x.get();
                    let off_y = self.drawing_offset_y.get();
                    let x = raw_x.wrapping_add(off_x);
                    let y = raw_y.wrapping_add(off_y);

                    let dither = (self.stat.get() & (1 << 9)) != 0;
                    if polyline {
                        if !gouraud {
                            if has_prev {
                                self.draw_line_segment(
                                    prev_vertex,
                                    (x, y),
                                    color0,
                                    color0,
                                    false,
                                    semi_transparent,
                                    dither,
                                );
                            }
                            prev_vertex = (x, y);
                            has_prev = true;
                        } else {
                            if has_prev {
                                self.draw_line_segment(
                                    prev_vertex,
                                    (x, y),
                                    prev_color,
                                    pending_color,
                                    true,
                                    semi_transparent,
                                    dither,
                                );
                                prev_color = pending_color;
                            }
                            prev_vertex = (x, y);
                            has_prev = true;
                            expecting_vertex = false;
                        }
                    } else {
                        if gouraud {
                            if vertex_count == 0 {
                                vertex0 = (x, y);
                                vertex_count = 1;
                                expecting_vertex = false;
                            } else {
                                let dx = (vertex0.0 as i32 - x as i32).abs();
                                let dy = (vertex0.1 as i32 - y as i32).abs();
                                if dx > 1023 || dy > 511 {
                                    self.stat.set(self.stat.get() | (1 << 26));
                                    self.vram_state.set(VramState::Idle);
                                    return;
                                }
                                self.render_single_line(
                                    vertex0,
                                    (x, y),
                                    color0,
                                    prev_color,
                                    true,
                                    semi_transparent,
                                    dither,
                                );
                                self.stat.set(self.stat.get() | (1 << 26));
                                self.vram_state.set(VramState::Idle);
                                return;
                            }
                        } else {
                            if vertex_count == 0 {
                                vertex0 = (x, y);
                                vertex_count = 1;
                            } else {
                                let dx = (vertex0.0 as i32 - x as i32).abs();
                                let dy = (vertex0.1 as i32 - y as i32).abs();
                                if dx > 1023 || dy > 511 {
                                    self.stat.set(self.stat.get() | (1 << 26));
                                    self.vram_state.set(VramState::Idle);
                                    return;
                                }
                                self.render_single_line(
                                    vertex0,
                                    (x, y),
                                    color0,
                                    color0,
                                    false,
                                    semi_transparent,
                                    dither,
                                );
                                self.stat.set(self.stat.get() | (1 << 26));
                                self.vram_state.set(VramState::Idle);
                                return;
                            }
                        }
                    }
                } else {
                    if polyline && gouraud && is_terminator {
                        self.stat.set(self.stat.get() | (1 << 26));
                        self.vram_state.set(VramState::Idle);
                        return;
                    }
                    pending_color = val & 0x00FF_FFFF;
                    if !gouraud || !polyline {
                        prev_color = val & 0x00FF_FFFF;
                    }
                    expecting_vertex = true;
                }
                self.vram_state.set(VramState::LineRender {
                    gouraud,
                    polyline,
                    semi_transparent,
                    color0,
                    vertex0,
                    prev_vertex,
                    prev_color,
                    pending_color,
                    has_prev,
                    vertex_count,
                    expecting_vertex,
                });
            }
            VramState::RectRender {
                size,
                textured,
                semi_transparent,
                color,
                mut vertex,
                mut uv,
                mut width,
                mut height,
                stage,
            } => match stage {
                RectStage::AwaitVertex => {
                    let raw_x = ((val & 0xFFFF) << 5) as i16 >> 5;
                    let raw_y = (((val >> 16) & 0xFFFF) << 5) as i16 >> 5;
                    let off_x = self.drawing_offset_x.get();
                    let off_y = self.drawing_offset_y.get();
                    vertex = (raw_x.wrapping_add(off_x), raw_y.wrapping_add(off_y));
                    if textured {
                        self.vram_state.set(VramState::RectRender {
                            size,
                            textured,
                            semi_transparent,
                            color,
                            vertex,
                            uv,
                            width,
                            height,
                            stage: RectStage::AwaitUV,
                        });
                        return;
                    } else if size == 0 {
                        self.vram_state.set(VramState::RectRender {
                            size,
                            textured,
                            semi_transparent,
                            color,
                            vertex,
                            uv,
                            width,
                            height,
                            stage: RectStage::AwaitDims,
                        });
                        return;
                    }
                    self.render_rect(size, color, vertex, width, height, semi_transparent);
                    self.stat.set(self.stat.get() | (1 << 26));
                    self.vram_state.set(VramState::Idle);
                }
                RectStage::AwaitUV => {
                    uv = val;
                    if textured {
                        if size == 0 {
                            self.vram_state.set(VramState::RectRender {
                                size,
                                textured,
                                semi_transparent,
                                color,
                                vertex,
                                uv,
                                width,
                                height,
                                stage: RectStage::AwaitDims,
                            });
                        } else {
                            self.stat.set(self.stat.get() | (1 << 26));
                            self.vram_state.set(VramState::Idle);
                        }
                        return;
                    }
                    if size == 0 {
                        self.vram_state.set(VramState::RectRender {
                            size,
                            textured,
                            semi_transparent,
                            color,
                            vertex,
                            uv,
                            width,
                            height,
                            stage: RectStage::AwaitDims,
                        });
                        return;
                    }
                    self.render_rect(size, color, vertex, width, height, semi_transparent);
                    self.stat.set(self.stat.get() | (1 << 26));
                    self.vram_state.set(VramState::Idle);
                }
                RectStage::AwaitDims => {
                    width = (val & 0xFFFF) as u16;
                    height = ((val >> 16) & 0xFFFF) as u16;
                    if textured {
                        self.stat.set(self.stat.get() | (1 << 26));
                        self.vram_state.set(VramState::Idle);
                        return;
                    }
                    self.render_rect(size, color, vertex, width, height, semi_transparent);
                    self.stat.set(self.stat.get() | (1 << 26));
                    self.vram_state.set(VramState::Idle);
                }
            },
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

    fn apply_clut_if_first(&mut self, vertex_idx: usize, uv_word: u32) {
        if vertex_idx == 0 {
            let clut_attr = (uv_word >> 16) & 0xFFFF;
            self.clut_attribute.set(clut_attr as u16);
        }
    }

    fn apply_texpage_if_second(&mut self, vertex_idx: usize, uv_word: u32) {
        if vertex_idx == 1 {
            let texpage = (uv_word >> 16) & 0xFFFF;
            let new_bits = (texpage & 0x1FF) | (((texpage >> 11) & 1) << 15);
            let mask = 0x1FF | (1 << 15);
            let s = self.stat.get();
            self.stat.set((s & !mask) | new_bits);
        }
    }

    fn write_pixel(&mut self, idx: usize, pixel: u16, semi_transparent: bool) {
        if semi_transparent && (pixel & 0x8000) != 0 {
            let back = self.vram[idx];
            let mode = (self.stat.get() >> 5) & 3;
            let r_b = back & 0x1F;
            let g_b = (back >> 5) & 0x1F;
            let b_b = (back >> 10) & 0x1F;
            let r_f = pixel & 0x1F;
            let g_f = (pixel >> 5) & 0x1F;
            let b_f = (pixel >> 10) & 0x1F;
            let (r, g, b) = match mode {
                0 => (
                    (r_b >> 1) + (r_f >> 1),
                    (g_b >> 1) + (g_f >> 1),
                    (b_b >> 1) + (b_f >> 1),
                ),
                1 => (
                    (r_b + r_f).min(31),
                    (g_b + g_f).min(31),
                    (b_b + b_f).min(31),
                ),
                2 => (
                    (r_b as i32 - r_f as i32).max(0) as u16,
                    (g_b as i32 - g_f as i32).max(0) as u16,
                    (b_b as i32 - b_f as i32).max(0) as u16,
                ),
                3 => (
                    (r_b + (r_f >> 2)).min(31),
                    (g_b + (g_f >> 2)).min(31),
                    (b_b + (b_f >> 2)).min(31),
                ),
                _ => (r_f, g_f, b_f),
            };
            self.vram[idx] = r | (g << 5) | (b << 10);
        } else {
            self.vram[idx] = pixel;
        }
    }

    fn sample_texel(&self, u: i32, v: i32) -> u16 {
        let stat = self.stat.get();
        let tex_colors = (stat >> 7) & 3;
        let mask_x = self.tex_window_mask_x.get() as u32;
        let mask_y = self.tex_window_mask_y.get() as u32;
        let off_x = self.tex_window_offset_x.get() as u32;
        let off_y = self.tex_window_offset_y.get() as u32;
        let u_win = (u as u32 & !(mask_x * 8)) | ((off_x & mask_x) * 8);
        let v_win = (v as u32 & !(mask_y * 8)) | ((off_y & mask_y) * 8);
        let u_clamped = (u_win as i32).clamp(0, 255) as u16;
        let v_clamped = (v_win as i32).clamp(0, 255) as u16;
        let page_x = ((stat & 0xF) as u16) * 64;
        let page_y = (((stat >> 4) & 1) as u16) * 256;

        match tex_colors {
            0 => {
                let pixel_index = v_clamped as usize * 256 + u_clamped as usize;
                let hw_x = page_x.wrapping_add((pixel_index / 4) as u16) as usize & 0x3FF;
                let hw_y = page_y.wrapping_add(v_clamped) as usize & 0x1FF;
                let hw = self.vram[hw_y * 1024 + hw_x];
                let nibble = (hw >> ((u_clamped % 4) * 4)) & 0xF;
                self.lookup_clut(nibble)
            }
            1 => {
                let pixel_index = v_clamped as usize * 256 + u_clamped as usize;
                let hw_x = page_x.wrapping_add((pixel_index / 2) as u16) as usize & 0x3FF;
                let hw_y = page_y.wrapping_add(v_clamped) as usize & 0x1FF;
                let hw = self.vram[hw_y * 1024 + hw_x];
                let byte = if u_clamped % 2 == 0 {
                    hw & 0xFF
                } else {
                    (hw >> 8) & 0xFF
                };
                self.lookup_clut(byte)
            }
            _ => {
                let addr = (page_y.wrapping_add(v_clamped) as usize & 0x1FF) * 1024
                    + (page_x.wrapping_add(u_clamped) as usize & 0x3FF);
                self.vram[addr]
            }
        }
    }

    fn lookup_clut(&self, index: u16) -> u16 {
        let attr = self.clut_attribute.get();
        let clut_x = (attr & 0x3F) * 16;
        let clut_y = (attr >> 6) & 0x1FF;
        let addr = (clut_y as usize & 0x1FF) * 1024 + (clut_x.wrapping_add(index) as usize & 0x3FF);
        self.vram[addr]
    }

    #[allow(clippy::too_many_arguments)]
    fn render_polygon(
        &mut self,
        gouraud: bool,
        quad: bool,
        textured: bool,
        semi_transparent: bool,
        vertices: &mut [(i16, i16); 4],
        colors: &mut [u32; 4],
        uvs: &mut [(u8, u8); 4],
    ) {
        let n = if quad { 4 } else { 3 };
        if !gouraud {
            let flat_color = colors[0];
            for c in colors.iter_mut().take(n) {
                *c = flat_color;
            }
        }
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = (vertices[i].0 as i32 - vertices[j].0 as i32).abs();
                let dy = (vertices[i].1 as i32 - vertices[j].1 as i32).abs();
                if dx > 1023 || dy > 511 {
                    return;
                }
            }
        }
        let tex_active = textured && {
            let tex_colors = (self.stat.get() >> 7) & 3;
            tex_colors <= 3
        };
        let dither = gouraud && !tex_active && (self.stat.get() & (1 << 9)) != 0;
        if quad {
            self.render_triangle(
                gouraud,
                tex_active,
                semi_transparent,
                dither,
                [vertices[0], vertices[1], vertices[2]],
                [colors[0], colors[1], colors[2]],
                [uvs[0], uvs[1], uvs[2]],
            );
            self.render_triangle(
                gouraud,
                tex_active,
                semi_transparent,
                dither,
                [vertices[1], vertices[2], vertices[3]],
                [colors[1], colors[2], colors[3]],
                [uvs[1], uvs[2], uvs[3]],
            );
        } else {
            self.render_triangle(
                gouraud,
                tex_active,
                semi_transparent,
                dither,
                [vertices[0], vertices[1], vertices[2]],
                [colors[0], colors[1], colors[2]],
                [uvs[0], uvs[1], uvs[2]],
            );
        }
    }

    fn render_triangle(
        &mut self,
        gouraud: bool,
        textured: bool,
        semi_transparent: bool,
        dither: bool,
        verts: [(i16, i16); 3],
        colors: [u32; 3],
        uvs: [(u8, u8); 3],
    ) {
        if dither && gouraud && !textured {
            self.render_triangle_dithered(
                verts,
                colors,
                semi_transparent,
            );
            return;
        }
        let (v0, v1, v2) = (verts[0], verts[1], verts[2]);
        let (c0, c1, c2) = (colors[0], colors[1], colors[2]);
        let (uv0, uv1, uv2) = (uvs[0], uvs[1], uvs[2]);
        let mut sorted = [
            (
                v0.0 as i32,
                v0.1 as i32,
                color24_to_16(c0),
                uv0.0 as i32,
                uv0.1 as i32,
            ),
            (
                v1.0 as i32,
                v1.1 as i32,
                color24_to_16(c1),
                uv1.0 as i32,
                uv1.1 as i32,
            ),
            (
                v2.0 as i32,
                v2.1 as i32,
                color24_to_16(c2),
                uv2.0 as i32,
                uv2.1 as i32,
            ),
        ];
        sorted.sort_by_key(|v| v.1);

        let (xm, ym, pcm, um, vm) = sorted[1];
        let (xb, yb, pcb, ub, vb) = sorted[2];
        let (xt, yt, pct, ut, vt) = sorted[0];

        let dy_mt = ym - yt;
        let dy_bt = yb - yt;
        let dy_bm = yb - ym;

        if dy_bt <= 0 {
            return;
        }

        let area_x1 = self.drawing_x1.get() as i32;
        let area_y1 = self.drawing_y1.get() as i32;
        let area_x2 = self.drawing_x2.get() as i32;
        let area_y2 = self.drawing_y2.get() as i32;
        let y_start = yt.max(area_y1).max(0);
        let y_end = yb.min(area_y2 + 1).min(512);

        for y in y_start..y_end {
            let x_edge_tb = lerp_i32(xt, xb, y - yt, dy_bt);
            let (x_edge_short, color_short, u_short, v_short) = if y < ym {
                (
                    lerp_i32(xt, xm, y - yt, dy_mt),
                    lerp_color(pct, pcm, y - yt, dy_mt),
                    lerp_i32(ut, um, y - yt, dy_mt),
                    lerp_i32(vt, vm, y - yt, dy_mt),
                )
            } else {
                (
                    lerp_i32(xm, xb, y - ym, dy_bm),
                    lerp_color(pcm, pcb, y - ym, dy_bm),
                    lerp_i32(um, ub, y - ym, dy_bm),
                    lerp_i32(vm, vb, y - ym, dy_bm),
                )
            };

            let color_tb = lerp_color(pct, pcb, y - yt, dy_bt);
            let u_tb = lerp_i32(ut, ub, y - yt, dy_bt);
            let v_tb = lerp_i32(vt, vb, y - yt, dy_bt);

            let (xl, xr, cl, cr, ul, ur, vl, vr) = if x_edge_tb < x_edge_short {
                (
                    x_edge_tb,
                    x_edge_short,
                    color_tb,
                    color_short,
                    u_tb,
                    u_short,
                    v_tb,
                    v_short,
                )
            } else {
                (
                    x_edge_short,
                    x_edge_tb,
                    color_short,
                    color_tb,
                    u_short,
                    u_tb,
                    v_short,
                    v_tb,
                )
            };

            let xl = xl.max(area_x1).max(0);
            let xr = xr.max(0).min(area_x2 + 1).min(1024);
            if xl >= xr {
                continue;
            }

            let dx = xr - xl;
            for x in xl..xr {
                let pixel = if textured {
                    let tex_u = if dx > 0 {
                        lerp_i32(ul, ur, x - xl, dx)
                    } else {
                        ul
                    };
                    let tex_v = if dx > 0 {
                        lerp_i32(vl, vr, x - xl, dx)
                    } else {
                        vl
                    };
                    let texel = self.sample_texel(tex_u, tex_v);
                    if texel == 0 {
                        continue;
                    }
                    texel
                } else if gouraud && dx > 0 {
                    lerp_color(cl, cr, x - xl, dx)
                } else {
                    pct
                };
                let idx = y as usize * 1024 + x as usize;
                self.write_pixel(idx, pixel, semi_transparent);
            }
        }
    }

    fn render_triangle_dithered(
        &mut self,
        verts: [(i16, i16); 3],
        colors: [u32; 3],
        semi_transparent: bool,
    ) {
        let (v0, v1, v2) = (verts[0], verts[1], verts[2]);
        let (c0, c1, c2) = (colors[0], colors[1], colors[2]);
        let mut sorted = [
            (v0.0 as i32, v0.1 as i32, c0),
            (v1.0 as i32, v1.1 as i32, c1),
            (v2.0 as i32, v2.1 as i32, c2),
        ];
        sorted.sort_by_key(|v| v.1);

        let (xm, ym, pcm) = sorted[1];
        let (xb, yb, pcb) = sorted[2];
        let (xt, yt, pct) = sorted[0];

        let dy_mt = ym - yt;
        let dy_bt = yb - yt;
        let dy_bm = yb - ym;

        if dy_bt <= 0 {
            return;
        }

        let area_x1 = self.drawing_x1.get() as i32;
        let area_y1 = self.drawing_y1.get() as i32;
        let area_x2 = self.drawing_x2.get() as i32;
        let area_y2 = self.drawing_y2.get() as i32;
        let y_start = yt.max(area_y1).max(0);
        let y_end = yb.min(area_y2 + 1).min(512);

        for y in y_start..y_end {
            let x_edge_tb = lerp_i32(xt, xb, y - yt, dy_bt);
            let (x_edge_short, color_short) = if y < ym {
                (
                    lerp_i32(xt, xm, y - yt, dy_mt),
                    lerp_color24(pct, pcm, y - yt, dy_mt),
                )
            } else {
                (
                    lerp_i32(xm, xb, y - ym, dy_bm),
                    lerp_color24(pcm, pcb, y - ym, dy_bm),
                )
            };

            let color_tb = lerp_color24(pct, pcb, y - yt, dy_bt);

            let (xl, xr, cl, cr) = if x_edge_tb < x_edge_short {
                (x_edge_tb, x_edge_short, color_tb, color_short)
            } else {
                (x_edge_short, x_edge_tb, color_short, color_tb)
            };

            let xl = xl.max(area_x1).max(0);
            let xr = xr.max(0).min(area_x2 + 1).min(1024);
            if xl >= xr {
                continue;
            }

            let dx = xr - xl;
            for x in xl..xr {
                let color24 = if dx > 0 {
                    lerp_color24(cl, cr, x - xl, dx)
                } else {
                    pct
                };
                let pixel = color24_to_16_dithered(color24, x, y, true);
                let idx = y as usize * 1024 + x as usize;
                self.write_pixel(idx, pixel, semi_transparent);
            }
        }
    }

    fn render_single_line(
        &mut self,
        v0: (i16, i16),
        v1: (i16, i16),
        c0: u32,
        c1: u32,
        gouraud: bool,
        semi_transparent: bool,
        dither: bool,
    ) {
        let x0 = v0.0 as i32;
        let y0 = v0.1 as i32;
        let x1 = v1.0 as i32;
        let y1 = v1.1 as i32;

        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let mut x = x0;
        let mut y = y0;
        let steps = dx.max(-dy);

        let area_x1 = self.drawing_x1.get() as i32;
        let area_y1 = self.drawing_y1.get() as i32;
        let area_x2 = self.drawing_x2.get() as i32;
        let area_y2 = self.drawing_y2.get() as i32;

        let mut step = 0i32;
        loop {
            let color24 = if gouraud && steps > 0 {
                lerp_color24(c0, c1, step, steps)
            } else {
                c0
            };
            let pixel = color24_to_16_dithered(color24, x, y, dither);
            if x >= area_x1
                && x <= area_x2
                && y >= area_y1
                && y <= area_y2
                && (0..1024).contains(&x)
                && (0..512).contains(&y)
            {
                self.write_pixel(y as usize * 1024 + x as usize, pixel, semi_transparent);
            }
            step += 1;
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    fn draw_line_segment(
        &mut self,
        v0: (i16, i16),
        v1: (i16, i16),
        c0: u32,
        c1: u32,
        gouraud: bool,
        semi_transparent: bool,
        dither: bool,
    ) {
        let dx = (v0.0 as i32 - v1.0 as i32).abs();
        let dy = (v0.1 as i32 - v1.1 as i32).abs();
        if dx > 1023 || dy > 511 {
            return;
        }
        self.render_single_line(v0, v1, c0, c1, gouraud, semi_transparent, dither);
    }

    fn render_rect(
        &mut self,
        size: u8,
        color: u32,
        vertex: (i16, i16),
        w: u16,
        h: u16,
        semi_transparent: bool,
    ) {
        let (w_actual, h_actual) = match size {
            0 => (w as u32, h as u32),
            1 => (1, 1),
            2 => (8, 8),
            3 => (16, 16),
            _ => return,
        };

        if w_actual == 0 || h_actual == 0 {
            return;
        }

        let pixel = color24_to_16(color);
        let x_start = vertex.0 as i32;
        let y_start = vertex.1 as i32;

        let area_x1 = self.drawing_x1.get() as i32;
        let area_y1 = self.drawing_y1.get() as i32;
        let area_x2 = self.drawing_x2.get() as i32;
        let area_y2 = self.drawing_y2.get() as i32;

        let draw_x0 = x_start.max(area_x1).max(0);
        let draw_x1 = (x_start + w_actual as i32).min(area_x2 + 1).min(1024);
        let draw_y0 = y_start.max(area_y1).max(0);
        let draw_y1 = (y_start + h_actual as i32).min(area_y2 + 1).min(512);

        for py in draw_y0..draw_y1 {
            for px in draw_x0..draw_x1 {
                self.write_pixel(py as usize * 1024 + px as usize, pixel, semi_transparent);
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
                self.drawing_x1.set(0);
                self.drawing_y1.set(0);
                self.drawing_x2.set(1023);
                self.drawing_y2.set(511);
                self.drawing_offset_x.set(0);
                self.drawing_offset_y.set(0);
                self.clut_attribute.set(0);
                self.tex_window_mask_x.set(0);
                self.tex_window_mask_y.set(0);
                self.tex_window_offset_x.set(0);
                self.tex_window_offset_y.set(0);
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
                awaiting_uv,
                textured,
                ..
            } => {
                write!(
                    f,
                    "PolygonRender(v={}/{}, tex={}, await_color={}, await_uv={})",
                    vertex_count, total_vertices, textured, awaiting_color, awaiting_uv
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
            VramState::LineRender {
                vertex_count,
                expecting_vertex,
                gouraud,
                polyline,
                ..
            } => {
                write!(
                    f,
                    "LineRender(v={}, g={}, pl={}, exp_v={})",
                    vertex_count, gouraud, polyline, expecting_vertex
                )
            }
            VramState::RectRender {
                size,
                textured,
                stage,
                ..
            } => {
                write!(
                    f,
                    "RectRender(sz={}, tex={}, stage={:?})",
                    size, textured, stage
                )
            }
        }
    }
}

impl fmt::Debug for RectStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RectStage::AwaitVertex => write!(f, "AwaitVertex"),
            RectStage::AwaitUV => write!(f, "AwaitUV"),
            RectStage::AwaitDims => write!(f, "AwaitDims"),
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
