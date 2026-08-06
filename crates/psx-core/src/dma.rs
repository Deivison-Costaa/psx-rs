use crate::cdrom::Cdrom;
use crate::gpu::Gpu;
use crate::mdec::Mdec;
use crate::spu::Spu;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Dma {
    madr: [u32; 7],
    bcr: [u32; 7],
    chcr: [u32; 7],
    dpcr: u32,
    dicr: u32,
    irq_line: bool,
}

impl Dma {
    pub fn new() -> Self {
        let mut chcr = [0u32; 7];
        chcr[6] = 0x0000_0002;
        Dma {
            madr: [0u32; 7],
            bcr: [0u32; 7],
            chcr,
            dpcr: 0x0765_4321,
            dicr: 0,
            irq_line: false,
        }
    }

    pub fn read_madr(&self, ch: usize) -> u32 {
        self.madr[ch] & 0x00FF_FFFF
    }

    pub fn write_madr(&mut self, ch: usize, val: u32) {
        self.madr[ch] = val & 0x00FF_FFFF;
    }

    pub fn read_bcr(&self, ch: usize) -> u32 {
        self.bcr[ch]
    }

    pub fn write_bcr(&mut self, ch: usize, val: u32) {
        self.bcr[ch] = val;
    }

    pub fn read_chcr(&self, ch: usize) -> u32 {
        self.chcr[ch]
    }

    pub fn write_chcr(&mut self, ch: usize, val: u32) {
        if ch == 6 {
            self.chcr[6] = (self.chcr[6] & !0x5100_0000) | (val & 0x5100_0000);
        } else {
            self.chcr[ch] = val;
        }
    }

    pub fn read_dpcr(&self) -> u32 {
        self.dpcr
    }

    pub fn write_dpcr(&mut self, val: u32) {
        self.dpcr = val;
    }

    pub fn read_dicr(&self) -> u32 {
        self.dicr
    }

    pub fn write_dicr(&mut self, val: u32) {
        let flags = self.dicr & !(val & 0x7F00_0000);
        self.dicr = (val & 0x00FF_807F) | (flags & 0x7F00_0000);
        self.recalc_master_flag();
    }

    fn recalc_master_flag(&mut self) {
        let bus_error = self.dicr & (1 << 15) != 0;
        let master_enable = self.dicr & (1 << 23) != 0;
        let algum_flag = self.dicr & 0x7F00_0000 != 0;
        if bus_error || (master_enable && algum_flag) {
            self.dicr |= 1 << 31;
        } else {
            self.dicr &= !(1 << 31);
        }
    }

    fn ram_transfer_in_bounds(&mut self, ram: &[u8], addr: u32, offset: usize) -> bool {
        // § D#_MADR (04-dma.md L48-50): MADR e um campo de 24 bits (0-23) — enderecos
        // com bits 21-23 ligados sao mascarados/espelhados pro decodificador de RAM
        // de 21 bits e NAO sao erro (dma_otc.rs::...guarda_24_bits_e_nao_dobra_em_21,
        // ja existente, exige que o ponteiro gravado preserve esses bits). O bus error
        // e so o wraparound que a spec descreve: contador decrementando de 000000h
        // passa por baixo de zero e vira ~FFFFFFFCh em aritmetica de 32 bits, bem
        // acima do proprio campo de 24 bits do MADR — dai o teto em 0x00FF_FFFF.
        if addr <= 0x00FF_FFFF && offset + 4 <= ram.len() {
            true
        } else {
            self.dicr |= 1 << 15;
            self.recalc_master_flag();
            false
        }
    }

    fn signal_completion(&mut self, ch: usize) {
        let mascara = self.dicr & (1 << (16 + ch)) != 0;
        let master_enable = self.dicr & (1 << 23) != 0;
        if mascara && master_enable {
            self.dicr |= 1 << (24 + ch);
            self.recalc_master_flag();
        }
    }

    pub fn irq3_pending(&self) -> bool {
        self.dicr & (1 << 31) != 0
    }

    pub fn take_irq3_edge(&mut self) -> bool {
        let nivel = self.irq3_pending();
        let borda = nivel && !self.irq_line;
        self.irq_line = nivel;
        borda
    }

    pub fn try_execute_otc(&mut self, ram: &mut [u8]) {
        if self.dpcr & (1 << 27) == 0 {
            return;
        }
        if (self.chcr[6] & ((1 << 24) | (1 << 28))) != ((1 << 24) | (1 << 28)) {
            return;
        }
        let madr = self.madr[6] & 0x00FF_FFFC;
        let bcr = self.bcr[6] & 0xFFFF;
        let count = if bcr == 0 { 0x10000 } else { bcr as usize };
        let mut addr = madr;
        for i in 0..count {
            let offset = (addr & 0x1F_FF_FF) as usize;
            if self.ram_transfer_in_bounds(ram, addr, offset) {
                let val = if i == count - 1 {
                    0x00FF_FFFF
                } else {
                    addr.wrapping_sub(4) & 0x00FF_FFFC
                };
                ram[offset..offset + 4].copy_from_slice(&val.to_le_bytes());
            }
            addr = addr.wrapping_sub(4);
        }
        self.chcr[6] &= !((1 << 24) | (1 << 28));
        self.signal_completion(6);
    }

    pub fn try_execute_dma3(&mut self, ram: &mut [u8], cdrom: &Cdrom) {
        if self.dpcr & (1 << 15) == 0 {
            return;
        }
        if (self.chcr[3] & ((1 << 24) | (1 << 28))) != ((1 << 24) | (1 << 28)) {
            return;
        }
        if !cdrom.drqsts_active() {
            return;
        }
        let bcr = self.bcr[3];
        let word_count = if (bcr & 0xFFFF) == 0 {
            0x10000
        } else {
            (bcr & 0xFFFF) as usize
        };
        let mut addr = self.madr[3] & 0x00FF_FFFC;

        for _ in 0..word_count {
            let b0 = cdrom.read8(2) as u32;
            let b1 = cdrom.read8(2) as u32;
            let b2 = cdrom.read8(2) as u32;
            let b3 = cdrom.read8(2) as u32;
            let word = b0 | (b1 << 8) | (b2 << 16) | (b3 << 24);
            let offset = (addr & 0x1F_FF_FF) as usize;
            if self.ram_transfer_in_bounds(ram, addr, offset) {
                ram[offset..offset + 4].copy_from_slice(&word.to_le_bytes());
            }
            addr = addr.wrapping_add(4);
        }
        self.madr[3] = (self.madr[3] & !0x00FF_FFFF) | (addr & 0x00FF_FFFF);
        self.chcr[3] &= !((1 << 24) | (1 << 28));
        self.signal_completion(3);
    }

    pub fn try_execute_dma2(&mut self, ram: &mut [u8], gpu: &mut Gpu) {
        if self.dpcr & (1 << 11) == 0 {
            return;
        }
        if self.chcr[2] & (1 << 24) == 0 {
            return;
        }
        let sync_mode = (self.chcr[2] >> 9) & 3;
        match sync_mode {
            0 => self.execute_burst(ram, gpu),
            1 => self.execute_block(ram, gpu),
            2 => self.execute_linked_list(ram, gpu),
            _ => {}
        }
    }

    fn execute_burst(&mut self, ram: &mut [u8], gpu: &mut Gpu) {
        let bc = self.bcr[2] & 0xFFFF;
        let count = if bc == 0 { 0x1_0000 } else { bc as usize };
        let step: i32 = if self.chcr[2] & 2 != 0 { -4 } else { 4 };
        let para_dispositivo = (self.chcr[2] & 1) != 0;
        let mut addr = self.madr[2] & 0x00FF_FFFC;

        for _ in 0..count {
            let offset = (addr & 0x1F_FF_FF) as usize;
            if self.ram_transfer_in_bounds(ram, addr, offset) {
                if para_dispositivo {
                    let word = u32::from_le_bytes([
                        ram[offset],
                        ram[offset + 1],
                        ram[offset + 2],
                        ram[offset + 3],
                    ]);
                    gpu.write32(0, word);
                } else {
                    let word = gpu.read32(0);
                    ram[offset..offset + 4].copy_from_slice(&word.to_le_bytes());
                }
            }
            addr = if step < 0 {
                addr.wrapping_sub(4)
            } else {
                addr.wrapping_add(4)
            };
        }
        // § D#_MADR / D#_BCR (04-dma.md L48-50, L80-81): em SyncMode=0 o hardware so
        // atualiza MADR e zera o campo BC se o chopping (CHCR.8) estiver ligado; sem
        // chopping os dois ficam congelados nos valores escritos antes do start.
        if self.chcr[2] & (1 << 8) != 0 {
            self.madr[2] = addr;
            self.bcr[2] &= !0xFFFF;
        }
        self.chcr[2] &= !(1 << 24);
        self.signal_completion(2);
    }

    fn execute_block(&mut self, ram: &mut [u8], gpu: &mut Gpu) {
        let bcr = self.bcr[2];
        let bs = if (bcr & 0xFFFF) == 0 {
            0x10000
        } else {
            (bcr & 0xFFFF) as usize
        };
        let ba = if ((bcr >> 16) & 0xFFFF) == 0 {
            0x10000
        } else {
            ((bcr >> 16) & 0xFFFF) as usize
        };
        let step: i32 = if self.chcr[2] & 2 != 0 { -4 } else { 4 };
        let para_dispositivo = self.chcr[2] & 1 != 0;
        let mut addr = self.madr[2] & 0x00FF_FFFC;

        for _ in 0..ba {
            for _ in 0..bs {
                let offset = (addr & 0x1F_FF_FF) as usize;
                if self.ram_transfer_in_bounds(ram, addr, offset) {
                    if para_dispositivo {
                        let word = u32::from_le_bytes([
                            ram[offset],
                            ram[offset + 1],
                            ram[offset + 2],
                            ram[offset + 3],
                        ]);
                        gpu.write32(0, word);
                    } else {
                        let word = gpu.read32(0);
                        ram[offset..offset + 4].copy_from_slice(&word.to_le_bytes());
                    }
                }
                addr = if step < 0 {
                    addr.wrapping_sub(4)
                } else {
                    addr.wrapping_add(4)
                };
            }
        }
        self.madr[2] = (self.madr[2] & !0x00FF_FFFF) | (addr & 0x00FF_FFFF);
        self.chcr[2] &= !(1 << 24);
        self.signal_completion(2);
    }

    // O teto de nos NAO e escolha de projeto: cada no comeca num endereco alinhado
    // a palavra e o proximo endereco sai inteiro do header, entao uma cadeia com
    // mais nos do que ha palavras na RAM repetiu algum endereco (casa dos pombos) e
    // portanto tem ciclo. Ciclo nunca completa em hardware (dma/chain-looping).
    // Um teto menor que esse corta cadeia legitima: a do Crash tem >4096 nos e o
    // jogo respondia `GPU timeout` em laco (achado 0185.2).
    fn execute_linked_list(&mut self, ram: &mut [u8], gpu: &mut Gpu) {
        let mut addr = self.madr[2] & 0x00FF_FFFC;
        let mut node_count = 0;
        let mut alcancou_fim = false;
        let teto_de_nos = ram.len() / 4;

        loop {
            node_count += 1;
            if node_count > teto_de_nos {
                break;
            }
            let offset = (addr & 0x1F_FF_FF) as usize;
            if !self.ram_transfer_in_bounds(ram, addr, offset) {
                alcancou_fim = true;
                break;
            }
            let header = u32::from_le_bytes([
                ram[offset],
                ram[offset + 1],
                ram[offset + 2],
                ram[offset + 3],
            ]);
            let next_addr = header & 0x00FF_FFFF;
            let word_count = (header >> 24) as usize;

            let mut data_addr = addr.wrapping_add(4);
            for _ in 0..word_count {
                let doff = (data_addr & 0x1F_FF_FF) as usize;
                if self.ram_transfer_in_bounds(ram, data_addr, doff) {
                    let word = u32::from_le_bytes([
                        ram[doff],
                        ram[doff + 1],
                        ram[doff + 2],
                        ram[doff + 3],
                    ]);
                    gpu.write32(0, word);
                }
                data_addr = data_addr.wrapping_add(4);
            }

            if next_addr == 0x00FF_FFFF || (next_addr & 0x0080_0000) != 0 {
                self.madr[2] = (self.madr[2] & !0x00FF_FFFF) | next_addr;
                alcancou_fim = true;
                break;
            }

            addr = next_addr & 0x00FF_FFFC;
        }
        if alcancou_fim {
            self.chcr[2] &= !(1 << 24);
            self.signal_completion(2);
        }
    }

    // § D#_BCR (docs/reference/04-dma.md L36-58): formato depende do SyncMode
    // do CHCR (bits9-10). SyncMode=0 usa um unico campo BC (bits0-15); SyncMode=1
    // usa BS*BA (bits0-15 * bits16-31). Os dois "podem ser 0001h..FFFFh, ou
    // 0=10000h". Confundir os dois formatos faz o canal 4 (SPU, unico deste lote
    // que usa SyncMode=0 em testDMA{Write,Read}ToRamSyncMode0) pedir bilhoes de
    // palavras em vez de poucas — motivo da rodada de correcao 0174.
    fn total_words(&self, ch: usize) -> usize {
        let bcr = self.bcr[ch];
        let field16 = |v: u32| -> usize { if v == 0 { 0x10000 } else { v as usize } };
        let sync_mode = (self.chcr[ch] >> 9) & 0x3;
        if sync_mode == 0 {
            field16(bcr & 0xFFFF)
        } else {
            field16(bcr & 0xFFFF) * field16((bcr >> 16) & 0xFFFF)
        }
    }

    /// DMA0 (MDECin, RAM->MDEC). § DMA (docs/reference/09-mdec.md L114-124).
    pub fn try_execute_dma0(&mut self, ram: &[u8], mdec: &mut Mdec) {
        if self.dpcr & (1 << 3) == 0 {
            return;
        }
        if self.chcr[0] & (1 << 24) == 0 {
            return;
        }
        let step: i32 = if self.chcr[0] & 2 != 0 { -4 } else { 4 };
        let mut addr = self.madr[0] & 0x00FF_FFFC;
        for _ in 0..self.total_words(0) {
            let offset = (addr & 0x1F_FFFF) as usize;
            if self.ram_transfer_in_bounds(ram, addr, offset) {
                let word = u32::from_le_bytes([
                    ram[offset],
                    ram[offset + 1],
                    ram[offset + 2],
                    ram[offset + 3],
                ]);
                mdec.write32(0, word);
            }
            addr = if step < 0 {
                addr.wrapping_sub(4)
            } else {
                addr.wrapping_add(4)
            };
        }
        self.madr[0] = (self.madr[0] & !0x00FF_FFFF) | (addr & 0x00FF_FFFF);
        self.chcr[0] &= !(1 << 24);
        self.signal_completion(0);
    }

    /// DMA1 (MDECout, MDEC->RAM). Transfere no maximo o que o MDEC realmente
    /// decodificou (`mdec.output_len()`): pedir mais do que isso e sintoma de
    /// BS/BA mal dimensionados pelo software guest (visto em mdec/4bit e
    /// mdec/8bit, iter 0174) e travaria a RAM inteira num motor sem timing por
    /// palavra — o canal fica "em andamento" (bit24 mantido) em vez de
    /// completar, igual a uma DMA real que nunca recebe mais DREQ
    /// (docs/reference/09-mdec.md L108-112).
    pub fn try_execute_dma1(&mut self, ram: &mut [u8], mdec: &Mdec) {
        if self.dpcr & (1 << 7) == 0 {
            return;
        }
        if self.chcr[1] & (1 << 24) == 0 {
            return;
        }
        let requested = self.total_words(1);
        // Em cor o DMA1 costura quatro blocos 8x8 num macrobloco 16x16: so pode levar
        // macroblocos inteiros, senao o reordenamento sai pela metade.
        let disponivel = match mdec.palavras_por_macrobloco() {
            Some(n) if n > 0 => (mdec.output_len() / n) * n,
            _ => mdec.output_len(),
        };
        let available = disponivel.min(requested);
        let step: i32 = if self.chcr[1] & 2 != 0 { -4 } else { 4 };
        let mut addr = self.madr[1] & 0x00FF_FFFC;
        for _ in 0..available {
            let word = mdec.read32_dma();
            let offset = (addr & 0x1F_FFFF) as usize;
            if self.ram_transfer_in_bounds(ram, addr, offset) {
                let bytes_lidas = word.to_le_bytes();
                ram[offset..offset + 4].copy_from_slice(&bytes_lidas);
            }
            addr = if step < 0 {
                addr.wrapping_sub(4)
            } else {
                addr.wrapping_add(4)
            };
        }
        self.madr[1] = (self.madr[1] & !0x00FF_FFFF) | (addr & 0x00FF_FFFF);
        if available == requested {
            self.chcr[1] &= !(1 << 24);
            self.signal_completion(1);
        }
    }

    /// DMA4 (SPU). § SPU RAM DMA-Write/-Read (docs/reference/08-spu.md L755-773).
    pub fn try_execute_dma4(&mut self, ram: &mut [u8], spu: &mut Spu) {
        if self.dpcr & (1 << 19) == 0 {
            return;
        }
        if self.chcr[4] & (1 << 24) == 0 {
            return;
        }
        let from_ram = self.chcr[4] & 1 != 0;
        let step: i32 = if self.chcr[4] & 2 != 0 { -4 } else { 4 };
        let mut addr = self.madr[4] & 0x00FF_FFFC;
        for _ in 0..self.total_words(4) {
            let offset = (addr & 0x1F_FFFF) as usize;
            if from_ram {
                if self.ram_transfer_in_bounds(ram, addr, offset) {
                    let word = u32::from_le_bytes([
                        ram[offset],
                        ram[offset + 1],
                        ram[offset + 2],
                        ram[offset + 3],
                    ]);
                    spu.dma_push_word(word);
                }
            } else {
                let word = spu.dma_pop_word();
                if self.ram_transfer_in_bounds(ram, addr, offset) {
                    ram[offset..offset + 4].copy_from_slice(&word.to_le_bytes());
                }
            }
            addr = if step < 0 {
                addr.wrapping_sub(4)
            } else {
                addr.wrapping_add(4)
            };
        }
        self.madr[4] = (self.madr[4] & !0x00FF_FFFF) | (addr & 0x00FF_FFFF);
        self.chcr[4] &= !(1 << 24);
        self.signal_completion(4);
    }
}

impl Default for Dma {
    fn default() -> Self {
        Self::new()
    }
}
