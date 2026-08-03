pub const REG_COUNT: usize = 32;
pub const RAM_END: u32 = 0x8_0000;

const D_APF1: usize = 0x00;
const D_APF2: usize = 0x01;
const V_IIR: usize = 0x02;
const V_COMB1: usize = 0x03;
const V_COMB2: usize = 0x04;
const V_COMB3: usize = 0x05;
const V_COMB4: usize = 0x06;
const V_WALL: usize = 0x07;
const V_APF1: usize = 0x08;
const V_APF2: usize = 0x09;
const M_LSAME: usize = 0x0A;
const M_RSAME: usize = 0x0B;
const M_LCOMB1: usize = 0x0C;
const M_RCOMB1: usize = 0x0D;
const M_LCOMB2: usize = 0x0E;
const M_RCOMB2: usize = 0x0F;
const D_LSAME: usize = 0x10;
const D_RSAME: usize = 0x11;
const M_LDIFF: usize = 0x12;
const M_RDIFF: usize = 0x13;
const M_LCOMB3: usize = 0x14;
const M_RCOMB3: usize = 0x15;
const M_LCOMB4: usize = 0x16;
const M_RCOMB4: usize = 0x17;
const D_LDIFF: usize = 0x18;
const D_RDIFF: usize = 0x19;
const M_LAPF1: usize = 0x1A;
const M_RAPF1: usize = 0x1B;
const M_LAPF2: usize = 0x1C;
const M_RAPF2: usize = 0x1D;
const V_LIN: usize = 0x1E;
const V_RIN: usize = 0x1F;

#[derive(Debug, Clone)]
pub struct Reverb {
    pub regs: [u16; REG_COUNT],
    pub vlout: u16,
    pub vrout: u16,
    pub mbase: u16,
    pub current: u32,
}

impl Default for Reverb {
    fn default() -> Self {
        Reverb {
            regs: [0; REG_COUNT],
            vlout: 0,
            vrout: 0,
            mbase: 0,
            current: 0,
        }
    }
}

impl Reverb {
    pub fn set_mbase(&mut self, val: u16) {
        self.mbase = val;
        self.current = u32::from(val) * 8;
    }

    pub fn advance(&mut self) {
        let base = u32::from(self.mbase) * 8;
        let proximo = (self.current + 2) & (RAM_END - 2);
        self.current = proximo.max(base);
    }

    fn vol(&self, reg: usize) -> i32 {
        i32::from(self.regs[reg] as i16)
    }

    /// Endereco absoluto de um registrador src/dst, relativo ao buffer corrente e
    /// enrolado dentro de mBASE..7FFFEh.
    fn addr(&self, deslocamento: i64) -> u32 {
        let base = i64::from(u32::from(self.mbase) * 8);
        let tamanho = i64::from(RAM_END) - base;
        if tamanho <= 0 {
            return base as u32;
        }
        let bruto = i64::from(self.current) + deslocamento;
        (base + (bruto - base).rem_euclid(tamanho)) as u32
    }

    fn reg_addr(&self, reg: usize) -> u32 {
        self.addr(i64::from(self.regs[reg]) * 8)
    }

    fn le(ram: &[u8], endereco: u32) -> i32 {
        let i = (endereco & (RAM_END - 2)) as usize;
        i32::from(i16::from_le_bytes([ram[i], ram[i + 1]]))
    }

    fn grava(ram: &mut [u8], endereco: u32, valor: i32, escrever: bool) {
        if !escrever {
            return;
        }
        let i = (endereco & (RAM_END - 2)) as usize;
        let bytes = (valor.clamp(-0x8000, 0x7FFF) as i16).to_le_bytes();
        ram[i] = bytes[0];
        ram[i + 1] = bytes[1];
    }

    /// § Reverb Formula (L947) de docs/reference/08-spu.md, na ordem em que a spec lista:
    /// mesmo lado, lado oposto, comb, APF1, APF2, volume de saida.
    pub fn run(&mut self, ram: &mut [u8], lin: i32, rin: i32, escrever: bool) -> (i32, i32) {
        let l_in = mul(lin, self.vol(V_LIN));
        let r_in = mul(rin, self.vol(V_RIN));
        let v_iir = self.vol(V_IIR);
        let v_wall = self.vol(V_WALL);

        for (dst, src, entrada) in [
            (M_LSAME, D_LSAME, l_in),
            (M_RSAME, D_RSAME, r_in),
            (M_LDIFF, D_RDIFF, l_in),
            (M_RDIFF, D_LDIFF, r_in),
        ] {
            let alvo = self.reg_addr(dst);
            let anterior = Self::le(ram, self.addr(i64::from(self.regs[dst]) * 8 - 2));
            let refletido = Self::le(ram, self.reg_addr(src));
            let valor = mul(entrada + mul(refletido, v_wall) - anterior, v_iir) + anterior;
            Self::grava(ram, alvo, valor, escrever);
        }

        let (mut lout, mut rout) = (0i32, 0i32);
        for (vol, esq, dir) in [
            (V_COMB1, M_LCOMB1, M_RCOMB1),
            (V_COMB2, M_LCOMB2, M_RCOMB2),
            (V_COMB3, M_LCOMB3, M_RCOMB3),
            (V_COMB4, M_LCOMB4, M_RCOMB4),
        ] {
            let v = self.vol(vol);
            lout += mul(Self::le(ram, self.reg_addr(esq)), v);
            rout += mul(Self::le(ram, self.reg_addr(dir)), v);
        }

        for (vol, deslocamento, esq, dir) in [
            (V_APF1, D_APF1, M_LAPF1, M_RAPF1),
            (V_APF2, D_APF2, M_LAPF2, M_RAPF2),
        ] {
            let v = self.vol(vol);
            let d = i64::from(self.regs[deslocamento]) * 8;
            for (canal, reg) in [(&mut lout, esq), (&mut rout, dir)] {
                let atrasado = Self::le(ram, self.addr(i64::from(self.regs[reg]) * 8 - d));
                let intermediario = *canal - mul(atrasado, v);
                Self::grava(ram, self.reg_addr(reg), intermediario, escrever);
                *canal = mul(intermediario, v) + atrasado;
            }
        }

        (
            mul(lout, self.vol_saida(true)),
            mul(rout, self.vol_saida(false)),
        )
    }

    fn vol_saida(&self, esquerda: bool) -> i32 {
        i32::from(if esquerda { self.vlout } else { self.vrout } as i16)
    }
}

/// § Notes (L1002) de docs/reference/08-spu.md: os produtos sao divididos por 8000h.
fn mul(amostra: i32, volume: i32) -> i32 {
    ((i64::from(amostra) * i64::from(volume)) >> 15) as i32
}
