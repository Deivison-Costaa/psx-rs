#[derive(Debug)]
pub struct Gte {
    pub regs: [u32; 64],
}

impl Default for Gte {
    fn default() -> Self {
        Gte { regs: [0u32; 64] }
    }
}

impl Gte {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn read_data(&self, rd: usize) -> u32 {
        self.regs[rd]
    }

    pub fn write_data(&mut self, rd: usize, val: u32) {
        self.regs[rd] = val;
    }

    pub fn read_control(&self, rd: usize) -> u32 {
        let idx = 32 + rd;
        let val = self.regs[idx];
        if rd == 31 {
            let raw = val & 0x7FFF_F000;
            let error_flag = flag_error_bit(raw);
            raw | error_flag
        } else if is_standalone_s16_control(rd) {
            (val as i16 as i32) as u32
        } else {
            val
        }
    }

    pub fn write_control(&mut self, rd: usize, val: u32) {
        let idx = 32 + rd;
        if rd == 31 {
            self.regs[idx] = val & 0x7FFF_F000;
        } else {
            self.regs[idx] = val;
        }
    }

    pub fn execute_command(&mut self, func: u32) {
        let cmd = func & 0x3F;
        let sf = (func >> 19) & 1;
        let lm = (func >> 10) & 1;

        self.regs[63] = 0;

        let mx = (func >> 17) & 3;
        let v = (func >> 15) & 3;
        let cv = (func >> 13) & 3;

        match cmd {
            0x01 => self.rtps(sf, lm),
            0x30 => self.rtpt(sf, lm),
            0x06 => self.nclip(),
            0x2D => self.avsz3(),
            0x2E => self.avsz4(),
            0x28 => self.sqr(sf),
            0x0C => self.op(sf, lm),
            0x12 => self.mvmva(sf, mx, v, cv, lm),
            _ => {}
        }
    }

    fn rtps(&mut self, sf: u32, lm: u32) {
        self.exec_rtps_vertex(0, sf, lm);
    }

    fn rtpt(&mut self, sf: u32, lm: u32) {
        self.exec_rtps_vertex(0, sf, lm);
        self.exec_rtps_vertex(1, sf, lm);
        self.exec_rtps_vertex(2, sf, lm);
    }

    fn nclip(&mut self) {
        let mut flag: u32 = 0;

        let sx0 = self.regs[12] as i16 as i64;
        let sy0 = (self.regs[12] >> 16) as i16 as i64;
        let sx1 = self.regs[13] as i16 as i64;
        let sy1 = (self.regs[13] >> 16) as i16 as i64;
        let sx2 = self.regs[14] as i16 as i64;
        let sy2 = (self.regs[14] >> 16) as i16 as i64;

        let mac0 = sx0 * sy1 + sx1 * sy2 + sx2 * sy0 - sx0 * sy2 - sx1 * sy0 - sx2 * sy1;

        if mac0 > 0x7FFF_FFFF {
            flag |= 1 << 16;
        } else if mac0 < -0x8000_0000i64 {
            flag |= 1 << 15;
        }

        self.regs[24] = mac0 as u32;
        self.regs[63] |= flag;
    }

    fn avsz3(&mut self) {
        let mut flag: u32 = 0;

        let zsf3 = self.regs[61] as i16 as i64;
        let sz1 = self.regs[17] as i64;
        let sz2 = self.regs[18] as i64;
        let sz3 = self.regs[19] as i64;

        let mac0 = zsf3 * (sz1 + sz2 + sz3);
        self.regs[24] = mac0 as u32;

        let otz = mac0 / 0x1000;
        let otz_clamped = if otz > 0xFFFF {
            flag |= 1 << 18;
            0xFFFFu32
        } else if otz < 0 {
            flag |= 1 << 18;
            0u32
        } else {
            otz as u32
        };

        self.regs[7] = otz_clamped;
        self.regs[63] |= flag;
    }

    fn avsz4(&mut self) {
        let mut flag: u32 = 0;

        let zsf4 = self.regs[62] as i16 as i64;
        let sz0 = self.regs[16] as i64;
        let sz1 = self.regs[17] as i64;
        let sz2 = self.regs[18] as i64;
        let sz3 = self.regs[19] as i64;

        let mac0 = zsf4 * (sz0 + sz1 + sz2 + sz3);
        self.regs[24] = mac0 as u32;

        let otz = mac0 / 0x1000;
        let otz_clamped = if otz > 0xFFFF {
            flag |= 1 << 18;
            0xFFFFu32
        } else if otz < 0 {
            flag |= 1 << 18;
            0u32
        } else {
            otz as u32
        };

        self.regs[7] = otz_clamped;
        self.regs[63] |= flag;
    }

    fn sqr(&mut self, sf: u32) {
        let mut flag: u32 = 0;

        let ir1 = self.regs[9] as i16 as i64;
        let ir2 = self.regs[10] as i16 as i64;
        let ir3 = self.regs[11] as i16 as i64;

        let shift = sf * 12;
        let mac1 = (ir1 * ir1) >> shift;
        let mac2 = (ir2 * ir2) >> shift;
        let mac3 = (ir3 * ir3) >> shift;

        self.regs[25] = mac1 as u32;
        self.regs[26] = mac2 as u32;
        self.regs[27] = mac3 as u32;

        let ir1_clamped = saturate_ir(mac1, 1, 24, &mut flag, false);
        let ir2_clamped = saturate_ir(mac2, 1, 23, &mut flag, false);
        let ir3_clamped = saturate_ir(mac3, 1, 22, &mut flag, false);

        self.regs[9] = ir1_clamped as u32;
        self.regs[10] = ir2_clamped as u32;
        self.regs[11] = ir3_clamped as u32;
        self.regs[63] |= flag;
    }

    fn op(&mut self, sf: u32, lm: u32) {
        let mut flag: u32 = 0;

        let ir1 = self.regs[9] as i16 as i64;
        let ir2 = self.regs[10] as i16 as i64;
        let ir3 = self.regs[11] as i16 as i64;

        let d1 = self.regs[32] as i16 as i64;
        let d2 = self.regs[34] as i16 as i64;
        let d3 = self.regs[36] as i16 as i64;

        let shift = sf * 12;
        let mac1 = (ir3 * d2 - ir2 * d3) >> shift;
        let mac2 = (ir1 * d3 - ir3 * d1) >> shift;
        let mac3 = (ir2 * d1 - ir1 * d2) >> shift;

        self.regs[25] = mac1 as u32;
        self.regs[26] = mac2 as u32;
        self.regs[27] = mac3 as u32;

        let ir1_clamped = saturate_ir(mac1, lm, 24, &mut flag, false);
        let ir2_clamped = saturate_ir(mac2, lm, 23, &mut flag, false);
        let ir3_clamped = saturate_ir(mac3, lm, 22, &mut flag, false);

        self.regs[9] = ir1_clamped as u32;
        self.regs[10] = ir2_clamped as u32;
        self.regs[11] = ir3_clamped as u32;
        self.regs[63] |= flag;
    }

    fn mvmva(&mut self, sf: u32, mx: u32, v: u32, cv: u32, lm: u32) {
        let mut flag: u32 = 0;

        let (base_mat, base_tr) = match (mx, cv) {
            (0, _) => (32, 37),
            (1, _) => (40, 45),
            (2, _) => (48, 53),
            _ => (0, 0),
        };

        let m11: i32;
        let m12: i32;
        let m13: i32;
        let m21: i32;
        let m22: i32;
        let m23: i32;
        let m31: i32;
        let m32: i32;
        let m33: i32;

        if mx == 3 {
            let r = (self.regs[6] & 0xFF) as i32;
            m11 = -(r * 0x10);
            m12 = r * 0x10;
            m13 = self.regs[8] as i16 as i32;
            m21 = self.regs[33] as i16 as i32;
            m22 = self.regs[33] as i16 as i32;
            m23 = self.regs[33] as i16 as i32;
            m31 = self.regs[34] as i16 as i32;
            m32 = self.regs[34] as i16 as i32;
            m33 = self.regs[34] as i16 as i32;
        } else {
            let b = base_mat;
            m11 = self.regs[b] as i16 as i32;
            m12 = (self.regs[b] >> 16) as i16 as i32;
            m13 = self.regs[b + 1] as i16 as i32;
            m21 = (self.regs[b + 1] >> 16) as i16 as i32;
            m22 = self.regs[b + 2] as i16 as i32;
            m23 = (self.regs[b + 2] >> 16) as i16 as i32;
            m31 = self.regs[b + 3] as i16 as i32;
            m32 = (self.regs[b + 3] >> 16) as i16 as i32;
            m33 = self.regs[b + 4] as i16 as i32;
        }

        let (vx, vy, vz): (i32, i32, i32) = match v {
            0 => (
                self.regs[0] as i16 as i32,
                (self.regs[0] >> 16) as i16 as i32,
                self.regs[1] as i16 as i32,
            ),
            1 => (
                self.regs[2] as i16 as i32,
                (self.regs[2] >> 16) as i16 as i32,
                self.regs[3] as i16 as i32,
            ),
            2 => (
                self.regs[4] as i16 as i32,
                (self.regs[4] >> 16) as i16 as i32,
                self.regs[5] as i16 as i32,
            ),
            _ => (
                self.regs[9] as i16 as i32,
                self.regs[10] as i16 as i32,
                self.regs[11] as i16 as i32,
            ),
        };

        let (tx, ty, tz): (i64, i64, i64) = if cv == 2 {
            (0, 0, 0)
        } else if cv == 3 {
            (0, 0, 0)
        } else {
            let t = base_tr;
            (
                self.regs[t] as i32 as i64,
                self.regs[t + 1] as i32 as i64,
                self.regs[t + 2] as i32 as i64,
            )
        };

        let raw1: i64;
        let raw2: i64;
        let raw3: i64;

        if cv == 2 {
            raw1 = (m12 as i64) * (vx as i64) + (m13 as i64) * (vy as i64);
            raw2 = (m22 as i64) * (vx as i64) + (m23 as i64) * (vy as i64);
            raw3 = (m32 as i64) * (vx as i64) + (m33 as i64) * (vy as i64);
        } else {
            raw1 = tx * 0x1000
                + (m11 as i64) * (vx as i64)
                + (m12 as i64) * (vy as i64)
                + (m13 as i64) * (vz as i64);
            raw2 = ty * 0x1000
                + (m21 as i64) * (vx as i64)
                + (m22 as i64) * (vy as i64)
                + (m23 as i64) * (vz as i64);
            raw3 = tz * 0x1000
                + (m31 as i64) * (vx as i64)
                + (m32 as i64) * (vy as i64)
                + (m33 as i64) * (vz as i64);
        }

        let shift = sf * 12;
        let mac1 = (raw1 >> shift) as i32;
        let mac2 = (raw2 >> shift) as i32;
        let mac3 = (raw3 >> shift) as i32;

        self.regs[25] = mac1 as u32;
        self.regs[26] = mac2 as u32;
        self.regs[27] = mac3 as u32;

        let ir1 = saturate_ir(mac1 as i64, lm, 24, &mut flag, false);
        let ir2 = saturate_ir(mac2 as i64, lm, 23, &mut flag, false);
        let ir3 = saturate_ir(mac3 as i64, lm, 22, &mut flag, false);

        self.regs[9] = ir1 as u32;
        self.regs[10] = ir2 as u32;
        self.regs[11] = ir3 as u32;

        self.regs[63] |= flag;
    }

    fn exec_rtps_vertex(&mut self, vi: usize, sf: u32, lm: u32) {
        let mut flag: u32 = 0;

        let vx = self.regs[vi * 2] as i16 as i32;
        let vy = (self.regs[vi * 2] >> 16) as i16 as i32;
        let vz = self.regs[vi * 2 + 1] as i16 as i32;

        let rt11 = self.regs[32] as i16 as i32;
        let rt12 = (self.regs[32] >> 16) as i16 as i32;
        let rt13 = self.regs[33] as i16 as i32;
        let rt21 = (self.regs[33] >> 16) as i16 as i32;
        let rt22 = self.regs[34] as i16 as i32;
        let rt23 = (self.regs[34] >> 16) as i16 as i32;
        let rt31 = self.regs[35] as i16 as i32;
        let rt32 = (self.regs[35] >> 16) as i16 as i32;
        let rt33 = self.regs[36] as i16 as i32;

        let trx = self.regs[37] as i32;
        let try_ = self.regs[38] as i32;
        let trz = self.regs[39] as i32;

        let raw1 = (trx as i64) * 0x1000
            + (rt11 as i64) * (vx as i64)
            + (rt12 as i64) * (vy as i64)
            + (rt13 as i64) * (vz as i64);
        let raw2 = (try_ as i64) * 0x1000
            + (rt21 as i64) * (vx as i64)
            + (rt22 as i64) * (vy as i64)
            + (rt23 as i64) * (vz as i64);
        let raw3 = (trz as i64) * 0x1000
            + (rt31 as i64) * (vx as i64)
            + (rt32 as i64) * (vy as i64)
            + (rt33 as i64) * (vz as i64);

        let shift = sf * 12;
        let mac1 = (raw1 >> shift) as i32;
        let mac2 = (raw2 >> shift) as i32;
        let mac3 = (raw3 >> shift) as i32;

        self.regs[25] = mac1 as u32;
        self.regs[26] = mac2 as u32;
        self.regs[27] = mac3 as u32;

        let ir1 = saturate_ir(mac1 as i64, lm, 24, &mut flag, false);
        let ir2 = saturate_ir(mac2 as i64, lm, 23, &mut flag, false);
        let ir3 = saturate_ir(mac3 as i64, lm, 22, &mut flag, true);

        self.regs[9] = ir1 as u32;
        self.regs[10] = ir2 as u32;
        self.regs[11] = ir3 as u32;

        let sz3_raw = mac3 as i64 >> (12 - shift);
        let sz3 = if sz3_raw > 0xFFFF {
            flag |= 1 << 18;
            0xFFFFu32
        } else if sz3_raw < 0 {
            flag |= 1 << 18;
            0u32
        } else {
            sz3_raw as u32
        };

        self.regs[16] = self.regs[17];
        self.regs[17] = self.regs[18];
        self.regs[18] = self.regs[19];
        self.regs[19] = sz3;

        let h = self.regs[58] as u16 as u32;
        let ofx = self.regs[56] as i32;
        let ofy = self.regs[57] as i32;
        let dqa = self.regs[59] as i16 as i32;
        let dqb = self.regs[60] as i32;

        let n = gte_divide(h, sz3, &mut flag);

        let mac0_1 = n as i64 * ir1 as i64 + ofx as i64;
        let sx2 = (mac0_1 / 0x10000) as i32;
        let sx2_clamped = if sx2 > 0x3FF {
            flag |= 1 << 14;
            0x3FF
        } else if sx2 < -0x400 {
            flag |= 1 << 14;
            -0x400
        } else {
            sx2
        };

        let mac0_2 = n as i64 * ir2 as i64 + ofy as i64;
        let sy2 = (mac0_2 / 0x10000) as i32;
        let sy2_clamped = if sy2 > 0x3FF {
            flag |= 1 << 13;
            0x3FF
        } else if sy2 < -0x400 {
            flag |= 1 << 13;
            -0x400
        } else {
            sy2
        };

        let mac0_3 = n as i64 * dqa as i64 + dqb as i64;
        let ir0 = (mac0_3 / 0x1000) as i32;
        let ir0_clamped = if ir0 > 0x1000 {
            flag |= 1 << 12;
            0x1000
        } else if ir0 < 0 {
            flag |= 1 << 12;
            0
        } else {
            ir0
        };

        self.regs[24] = mac0_3 as u32;
        self.regs[8] = ir0_clamped as u32;

        self.regs[12] = self.regs[13];
        self.regs[13] = self.regs[14];
        self.regs[14] = (sy2_clamped as u32) << 16 | (sx2_clamped as u32 & 0xFFFF);

        self.regs[63] |= flag;
    }
}

fn is_standalone_s16_control(rd: usize) -> bool {
    matches!(rd, 4 | 12 | 20 | 27 | 29 | 30)
}

fn flag_error_bit(flag: u32) -> u32 {
    let bits_30_23 = (flag >> 23) & 0xFF;
    let bits_18_13 = (flag >> 13) & 0x3F;
    let or_result = bits_30_23 | bits_18_13;
    if or_result != 0 { 1u32 << 31 } else { 0 }
}

fn saturate_ir(val: i64, lm: u32, flag_bit: u32, flag: &mut u32, force_lm0: bool) -> i32 {
    let use_lm0 = force_lm0 || lm == 0;
    let (min_val, max_val) = if use_lm0 {
        (-0x8000i64, 0x7FFFi64)
    } else {
        (0i64, 0x7FFFi64)
    };
    if val > max_val {
        *flag |= 1 << flag_bit;
        max_val as i32
    } else if val < min_val {
        *flag |= 1 << flag_bit;
        min_val as i32
    } else {
        val as i32
    }
}

static UNR_TABLE: [u8; 257] = [
    0xFF, 0xFD, 0xFB, 0xF9, 0xF7, 0xF5, 0xF3, 0xF1, 0xEF, 0xEE, 0xEC, 0xEA, 0xE8, 0xE6, 0xE4, 0xE3,
    0xE1, 0xDF, 0xDD, 0xDC, 0xDA, 0xD8, 0xD6, 0xD5, 0xD3, 0xD1, 0xD0, 0xCE, 0xCD, 0xCB, 0xC9, 0xC8,
    0xC6, 0xC5, 0xC3, 0xC1, 0xC0, 0xBE, 0xBD, 0xBB, 0xBA, 0xB8, 0xB7, 0xB5, 0xB4, 0xB2, 0xB1, 0xB0,
    0xAE, 0xAD, 0xAB, 0xAA, 0xA9, 0xA7, 0xA6, 0xA4, 0xA3, 0xA2, 0xA0, 0x9F, 0x9E, 0x9C, 0x9B, 0x9A,
    0x99, 0x97, 0x96, 0x95, 0x94, 0x92, 0x91, 0x90, 0x8F, 0x8D, 0x8C, 0x8B, 0x8A, 0x89, 0x87, 0x86,
    0x85, 0x84, 0x83, 0x82, 0x81, 0x7F, 0x7E, 0x7D, 0x7C, 0x7B, 0x7A, 0x79, 0x78, 0x77, 0x75, 0x74,
    0x73, 0x72, 0x71, 0x70, 0x6F, 0x6E, 0x6D, 0x6C, 0x6B, 0x6A, 0x69, 0x68, 0x67, 0x66, 0x65, 0x64,
    0x63, 0x62, 0x61, 0x60, 0x5F, 0x5E, 0x5D, 0x5D, 0x5C, 0x5B, 0x5A, 0x59, 0x58, 0x57, 0x56, 0x55,
    0x54, 0x53, 0x53, 0x52, 0x51, 0x50, 0x4F, 0x4E, 0x4D, 0x4D, 0x4C, 0x4B, 0x4A, 0x49, 0x48, 0x48,
    0x47, 0x46, 0x45, 0x44, 0x43, 0x43, 0x42, 0x41, 0x40, 0x3F, 0x3F, 0x3E, 0x3D, 0x3C, 0x3C, 0x3B,
    0x3A, 0x39, 0x39, 0x38, 0x37, 0x36, 0x36, 0x35, 0x34, 0x33, 0x33, 0x32, 0x31, 0x31, 0x30, 0x2F,
    0x2E, 0x2E, 0x2D, 0x2C, 0x2C, 0x2B, 0x2A, 0x2A, 0x29, 0x28, 0x28, 0x27, 0x26, 0x26, 0x25, 0x24,
    0x24, 0x23, 0x22, 0x22, 0x21, 0x20, 0x20, 0x1F, 0x1E, 0x1E, 0x1D, 0x1D, 0x1C, 0x1B, 0x1B, 0x1A,
    0x19, 0x19, 0x18, 0x18, 0x17, 0x16, 0x16, 0x15, 0x15, 0x14, 0x14, 0x13, 0x12, 0x12, 0x11, 0x11,
    0x10, 0x0F, 0x0F, 0x0E, 0x0E, 0x0D, 0x0D, 0x0C, 0x0C, 0x0B, 0x0A, 0x0A, 0x09, 0x09, 0x08, 0x08,
    0x07, 0x07, 0x06, 0x06, 0x05, 0x05, 0x04, 0x04, 0x03, 0x03, 0x02, 0x02, 0x01, 0x01, 0x00, 0x00,
    0x00,
];

fn gte_divide(h: u32, sz3: u32, flag: &mut u32) -> u32 {
    let h = h as u16;
    let sz3 = sz3 as u16;
    if sz3 == 0 || (h as u32) >= (sz3 as u32) * 2 {
        *flag |= 1 << 17;
        return 0x1FFFF;
    }
    if h == 0 {
        return 0;
    }
    let z = sz3.leading_zeros();
    let n = (h as u32) << z;
    let d = ((sz3 as u32) << z).clamp(0x8000, 0xFFFF);
    let idx = ((d - 0x7FC0) >> 7) as usize;
    let u = UNR_TABLE[idx] as u32 + 0x101;
    let d2 = ((0x2000080u64 - (d as u64) * (u as u64)) >> 8) as u32;
    let d3 = ((0x0000080u64 + (d2 as u64) * (u as u64)) >> 8) as u32;
    let result = ((n as u64) * (d3 as u64) + 0x8000) >> 16;
    if result > 0x1FFFF {
        0x1FFFF
    } else {
        result as u32
    }
}
