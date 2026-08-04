use std::path::Path;

use psx_core::bus::{Bios, Bus, Ram};
use psx_core::cpu::Cpu;

use crate::audio::AudioOut;

const TECLAS: [(egui::Key, u32); 14] = [
    (egui::Key::ArrowUp, 4),
    (egui::Key::ArrowDown, 6),
    (egui::Key::ArrowLeft, 7),
    (egui::Key::ArrowRight, 5),
    (egui::Key::Z, 14),
    (egui::Key::Space, 13),
    (egui::Key::A, 15),
    (egui::Key::S, 12),
    (egui::Key::Enter, 3),
    (egui::Key::Tab, 0),
    (egui::Key::D, 10),
    (egui::Key::F, 11),
    (egui::Key::E, 8),
    (egui::Key::R, 9),
];

pub struct Emulador {
    cpu: Cpu,
    bus: Bus,
    audio: AudioOut,
    textura: Option<egui::ColorImage>,
    passos_por_quadro: usize,
    memcard: Option<String>,
}

impl Emulador {
    pub fn novo(bios_bytes: Vec<u8>, memcard: Option<String>) -> Result<Self, String> {
        let bios = Bios::from_bytes(bios_bytes).map_err(|e| format!("BIOS invalida: {e:?}"))?;
        let mut bus = Bus::new(Ram::new(), bios);
        let passos_por_quadro = bus.gpu().frame_cycles() as usize;
        bus.sio_mut().connect_digital_pad(true);

        if let Some(caminho) = memcard.as_deref() {
            let bytes =
                std::fs::read(caminho).unwrap_or_else(|_| vec![0u8; psx_core::memcard::CARD_BYTES]);
            bus.sio_mut()
                .load_memory_card(&bytes)
                .map_err(|e| format!("memory card invalido: {e:?}"))?;
        }

        Ok(Emulador {
            cpu: Cpu::new(),
            bus,
            audio: AudioOut::new(),
            textura: None,
            passos_por_quadro,
            memcard,
        })
    }

    pub fn insere_disco(&mut self, cue: &Path) -> Result<(), String> {
        let (layout, bin) = crate::disco::carrega(cue)?;
        self.bus.inject_disc(layout, bin);
        self.bus.cdrom_mut().insert_disc();
        Ok(())
    }

    pub fn teclado(&mut self, ctx: &egui::Context) {
        let mut botoes: u16 = 0xFFFF;
        for (tecla, bit) in TECLAS {
            if ctx.input(|i| i.key_down(tecla)) {
                botoes &= !(1u16 << bit);
            }
        }
        self.bus.sio_mut().set_buttons(botoes);
    }

    pub fn quadro(&mut self) {
        for _ in 0..self.passos_por_quadro {
            self.cpu.step(&mut self.bus);
        }
        let quadros = self.bus.drain_audio();
        self.audio.push(&quadros);
        self.salva_memcard();
        self.atualiza_textura();
    }

    fn salva_memcard(&mut self) {
        let Some(caminho) = self.memcard.clone() else {
            return;
        };
        if self.bus.sio().memory_card_dirty() {
            let _ = std::fs::write(caminho, self.bus.sio().memory_card_image());
        }
    }

    fn atualiza_textura(&mut self) {
        let Some(fb) = self.bus.gpu().framebuffer_for_display() else {
            self.textura = None;
            return;
        };
        let tamanho = [fb.width as usize, fb.height as usize];
        self.textura = Some(egui::ColorImage::from_rgba_unmultiplied(tamanho, &fb.data));
    }

    pub fn textura(&self) -> Option<&egui::ColorImage> {
        self.textura.as_ref()
    }

    pub fn audio_ativo(&self) -> bool {
        self.audio.ativo()
    }

    pub fn audio_hz(&self) -> u32 {
        self.audio.device_hz()
    }
}
