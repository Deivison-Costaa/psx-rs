mod support;

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use psx_core::bus::{Bus, BusRead, BusWrite};
use psx_core::cdrom_bin_cue::{DiscLayout, TrackInfo, TrackType};
use psx_core::disc_image::{DiscImage, RAW_SECTOR_BYTES, RawSector, VecDisc};
use support::asm;

const CD_BASE: u32 = 0x1F80_1800;
const ESPERA_PRIMEIRA_RESPOSTA: u32 = 0x1_4000;
const SETORES: u32 = 64;

#[test]
fn vec_disc_conta_so_setores_inteiros() {
    let disco = VecDisc::new(vec![0u8; 3 * RAW_SECTOR_BYTES + 100]);
    assert_eq!(disco.sector_count(), 3);
}

#[test]
fn vec_disc_le_o_setor_pelo_lba() {
    let mut bytes = vec![0u8; 3 * RAW_SECTOR_BYTES];
    bytes[RAW_SECTOR_BYTES] = 0xAB;
    bytes[2 * RAW_SECTOR_BYTES - 1] = 0xCD;
    let setor = VecDisc::new(bytes).read_sector(1).expect("setor 1 existe");
    assert_eq!(setor[0], 0xAB);
    assert_eq!(setor[RAW_SECTOR_BYTES - 1], 0xCD);
}

#[test]
fn vec_disc_recusa_setor_incompleto_ou_fora_da_imagem() {
    let disco = VecDisc::new(vec![0u8; 2 * RAW_SECTOR_BYTES + 10]);
    assert!(disco.read_sector(2).is_none());
    assert!(disco.read_sector(u32::MAX).is_none());
}

/// Imagem que fabrica cada setor na hora do pedido: prova que o drive so depende da
/// interface setor a setor, sem a imagem inteira na memoria.
#[derive(Debug)]
struct DiscoSintetico {
    leituras: Arc<AtomicU32>,
}

fn bcd(v: u32) -> u8 {
    (((v / 10) << 4) | (v % 10)) as u8
}

impl DiscImage for DiscoSintetico {
    fn sector_count(&self) -> u32 {
        SETORES
    }

    fn read_sector(&self, lba: u32) -> Option<RawSector> {
        if lba >= SETORES {
            return None;
        }
        self.leituras.fetch_add(1, Ordering::Relaxed);
        let mut s = [0u8; RAW_SECTOR_BYTES];
        s[1..11].fill(0xFF);
        let abs = lba + 150;
        s[0x0C] = bcd(abs / 4500);
        s[0x0D] = bcd((abs / 75) % 60);
        s[0x0E] = bcd(abs % 75);
        s[0x0F] = 0x02;
        for (i, b) in s.iter_mut().enumerate().skip(0x18) {
            *b = (lba as u8).wrapping_mul(7).wrapping_add(i as u8);
        }
        Some(s)
    }
}

fn layout() -> DiscLayout {
    DiscLayout {
        bin_path: "sintetico.bin".to_string(),
        tracks: vec![TrackInfo {
            number: 1,
            file: "sintetico.bin".to_string(),
            start_lba: 0,
            track_type: TrackType::Mode2_2352,
            index01_mm: 0,
            index01_ss: 0,
            index01_ff: 0,
            index00_mm: None,
            index00_ss: None,
            index00_ff: None,
            pregap_mm: None,
            pregap_ss: None,
            pregap_ff: None,
        }],
    }
}

fn cd_read(bus: &Bus, offset: u32) -> u8 {
    bus.read8::<BusRead>(CD_BASE + offset)
}

fn cd_write(bus: &mut Bus, offset: u32, val: u8) {
    bus.write8::<BusWrite>(CD_BASE + offset, val);
}

fn comando(bus: &mut Bus, cmd: u8, params: &[u8]) {
    cd_write(bus, 0, 0);
    for p in params {
        cd_write(bus, 2, *p);
    }
    cd_write(bus, 1, cmd);
    bus.tick_timers(ESPERA_PRIMEIRA_RESPOSTA);
    let _ = cd_read(bus, 1);
    cd_write(bus, 0, 1);
    cd_write(bus, 3, 0x07);
    cd_write(bus, 0, 0);
}

#[test]
fn drive_le_setores_de_uma_imagem_sob_demanda() {
    let leituras = Arc::new(AtomicU32::new(0));
    let mut bus = asm::bus_with_bios_empty();
    bus.inject_disc_image(
        layout(),
        Box::new(DiscoSintetico {
            leituras: Arc::clone(&leituras),
        }),
    );
    bus.cdrom_mut().insert_disc();
    comando(&mut bus, 0x0E, &[0x20]);
    comando(&mut bus, 0x02, &[0x00, 0x02, 0x10]);
    comando(&mut bus, 0x06, &[]);
    let busca = bus.cdrom().second_response_cycles() as u32;
    bus.tick_timers(busca);
    cd_write(&mut bus, 0, 1);
    assert_eq!(
        cd_read(&bus, 3) & 0x07,
        1,
        "o primeiro setor chega com INT1"
    );
    cd_write(&mut bus, 0, 0);
    let _ = cd_read(&bus, 1);
    cd_write(&mut bus, 0, 1);
    cd_write(&mut bus, 3, 0x07);
    cd_write(&mut bus, 0, 0);
    cd_write(&mut bus, 3, 0x00);
    cd_write(&mut bus, 3, 0x80);
    let cabecalho: Vec<u8> = (0..4).map(|_| cd_read(&bus, 2)).collect();
    assert_eq!(cabecalho, vec![0x00, 0x02, 0x10, 0x02]);
    assert!(leituras.load(Ordering::Relaxed) > 0);
}
