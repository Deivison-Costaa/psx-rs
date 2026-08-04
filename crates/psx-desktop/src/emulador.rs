use std::path::{Path, PathBuf};

use psx_core::app::config::Config;
use psx_core::app::input_map::{Entrada, Perfil};
use psx_core::app::saves::{self, Save};
use psx_core::app::sessao;
use psx_core::bus::{Bios, Bus, Ram};
use psx_core::cpu::Cpu;
use psx_core::snapshot;

use crate::audio::AudioOut;

pub const SLOTS: u8 = 10;

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
    memcard: PathBuf,
    serial: String,
    pasta_de_saves: PathBuf,
    pub slot: u8,
    pub aviso: Option<String>,
    pub velocidade: u32,
    quadros: u64,
}

impl Emulador {
    /// Um cartao por jogo: `cartoes/<serial>.mcd`, criado zerado na primeira vez. Cartao
    /// unico compartilhado enche com 15 blocos e obriga o usuario a apagar save alheio.
    pub fn novo(bios_bytes: Vec<u8>, serial: &str, config: &Config) -> Result<Self, String> {
        let bios = Bios::from_bytes(bios_bytes).map_err(|e| format!("BIOS invalida: {e:?}"))?;
        let mut bus = Bus::new(Ram::new(), bios);
        let passos_por_quadro = bus.gpu().frame_cycles() as usize;
        bus.sio_mut().connect_digital_pad(true);

        let memcard = Path::new(&config.pasta_de_cartoes).join(saves::nome_do_cartao(serial));
        let bytes =
            std::fs::read(&memcard).unwrap_or_else(|_| vec![0u8; psx_core::memcard::CARD_BYTES]);
        bus.sio_mut()
            .load_memory_card(&bytes)
            .map_err(|e| format!("memory card invalido: {e:?}"))?;

        Ok(Emulador {
            cpu: Cpu::new(),
            bus,
            audio: AudioOut::new(),
            textura: None,
            passos_por_quadro,
            memcard,
            serial: serial.to_string(),
            pasta_de_saves: PathBuf::from(&config.pasta_de_saves),
            slot: config.slot_inicial,
            aviso: None,
            velocidade: 1,
            quadros: 0,
        })
    }

    pub fn serial(&self) -> &str {
        &self.serial
    }

    pub fn caminho_do_cartao(&self) -> &Path {
        &self.memcard
    }

    pub fn saves_do_cartao(&self) -> Vec<Save> {
        saves::lista(&self.bus.sio().memory_card_image())
    }

    pub fn insere_disco(&mut self, cue: &Path) -> Result<(), String> {
        let (layout, bin) = crate::disco::carrega(cue)?;
        self.bus.inject_disc(layout, bin);
        self.bus.cdrom_mut().insert_disc();
        Ok(())
    }

    pub fn caminho_do_slot(&self, slot: u8) -> PathBuf {
        self.pasta_de_saves
            .join(format!("{}-{slot}.state", self.serial))
    }

    /// F5. Um slot por arquivo: sobrescrever o anterior e o comportamento esperado, mas
    /// perder o save por falta da pasta nao e — por isso a pasta e criada aqui.
    pub fn salva_estado(&mut self) {
        let caminho = self.caminho_do_slot(self.slot);
        if let Some(pai) = caminho.parent() {
            if let Err(e) = std::fs::create_dir_all(pai) {
                self.aviso = Some(format!("nao consegui criar '{}': {e}", pai.display()));
                return;
            }
        }
        let bytes = snapshot::salva(&self.cpu, &self.bus, &self.serial);
        self.aviso = Some(match std::fs::write(&caminho, &bytes) {
            Ok(()) => format!("slot {} salvo ({} KiB)", self.slot, bytes.len() / 1024),
            Err(e) => format!("nao consegui gravar o slot {}: {e}", self.slot),
        });
    }

    /// F8. Estado recusado nao mexe na maquina: o `carrega` decodifica tudo antes de
    /// escrever qualquer campo.
    pub fn carrega_estado(&mut self) {
        let caminho = self.caminho_do_slot(self.slot);
        let Ok(bytes) = std::fs::read(&caminho) else {
            self.aviso = Some(format!("slot {} vazio", self.slot));
            return;
        };
        self.aviso = Some(
            match snapshot::carrega(&mut self.cpu, &mut self.bus, &bytes, &self.serial) {
                Ok(()) => format!("slot {} carregado", self.slot),
                Err(e) => format!("slot {}: {e}", self.slot),
            },
        );
    }

    pub fn slot_existe(&self, slot: u8) -> bool {
        self.caminho_do_slot(slot).exists()
    }

    /// Teclado e controle valem ao mesmo tempo: o pad do PS1 recebe a UNIAO dos dois,
    /// que e o que um jogador que larga o controle e pega o teclado espera.
    pub fn entrada(&mut self, ctx: &egui::Context, perfil: &Perfil, do_controle: &[Entrada]) {
        let mut botoes: u16 = 0xFFFF;
        for (tecla, bit) in TECLAS {
            if ctx.input(|i| i.key_down(tecla)) {
                botoes &= !(1u16 << bit);
            }
        }
        botoes &= perfil.palavra(do_controle);
        self.bus.sio_mut().set_buttons(botoes);
    }

    pub fn quadro(&mut self, ganho: f32) {
        let passos = sessao::passos_por_quadro(self.passos_por_quadro, self.velocidade);
        for _ in 0..passos {
            self.cpu.step(&mut self.bus);
        }
        self.quadros += 1;
        let quadros = self.bus.drain_audio();
        self.audio.push(&quadros, ganho);
        self.salva_memcard();
        self.atualiza_textura();
    }

    fn salva_memcard(&mut self) {
        if !self.bus.sio().memory_card_dirty() {
            return;
        }
        if let Some(pai) = self.memcard.parent() {
            let _ = std::fs::create_dir_all(pai);
        }
        let _ = std::fs::write(&self.memcard, self.bus.sio().memory_card_image());
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

    /// Tempo de jogo em segundos EMULADOS: conta quadros a 60 Hz, nao relogio de parede.
    /// Uma hora em fast-forward 8x continua sendo uma hora de jogo para quem jogou.
    pub fn segundos_jogados(&self) -> u64 {
        self.quadros / 60
    }

    pub fn troca_velocidade(&mut self) {
        self.velocidade = sessao::proxima_velocidade(self.velocidade);
    }
}
