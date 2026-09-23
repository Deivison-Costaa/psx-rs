use std::fmt::Debug;

pub const RAW_SECTOR_BYTES: usize = 2352;

pub type RawSector = [u8; RAW_SECTOR_BYTES];

/// Fonte de setores crus da imagem, indexada pelo LBA da imagem (0 = MSF 00:02:00).
/// O nucleo so pede setor a setor; quem implementa decide se le da RAM ou do arquivo.
pub trait DiscImage: Debug + Send {
    fn sector_count(&self) -> u32;

    fn read_sector(&self, lba: u32) -> Option<RawSector>;
}

#[derive(Debug, Clone, Default)]
pub struct VecDisc {
    bytes: Vec<u8>,
}

impl VecDisc {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
}

impl DiscImage for VecDisc {
    fn sector_count(&self) -> u32 {
        u32::try_from(self.bytes.len() / RAW_SECTOR_BYTES).unwrap_or(u32::MAX)
    }

    fn read_sector(&self, lba: u32) -> Option<RawSector> {
        let inicio = usize::try_from(lba).ok()?.checked_mul(RAW_SECTOR_BYTES)?;
        let fim = inicio.checked_add(RAW_SECTOR_BYTES)?;
        let fatia = self.bytes.get(inicio..fim)?;
        let mut setor = [0u8; RAW_SECTOR_BYTES];
        setor.copy_from_slice(fatia);
        Some(setor)
    }
}
