use std::path::{Path, PathBuf};

use psx_core::app::config::Config;
use psx_core::app::input_map::{Eixos, Entrada, Perfil, direcao};
use psx_core::app::saves::{self, Save};
use psx_core::app::sessao;
use psx_core::app::troca::{PortaAberta, apertou_agora};
use psx_core::bus::{Bios, Bus, Ram};
use psx_core::cpu::Cpu;
use psx_core::dualshock::Rumble;
use psx_core::snapshot;

use crate::audio::AudioOut;
use crate::gamepad::Leitura;

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

/// Analogico esquerdo no teclado (I/J/K/L): so pesa no modo analogico, porque no digital
/// o jogo le o direcional das setas.
const STICK_ESQUERDO: [egui::Key; 4] = [egui::Key::J, egui::Key::L, egui::Key::I, egui::Key::K];
const TECLA_ANALOG: egui::Key = egui::Key::F3;

const CPU_HZ: f64 = 33_868_800.0;

pub struct Emulador {
    cpu: Cpu,
    bus: Bus,
    audio: AudioOut,
    textura: Option<egui::ColorImage>,
    memcard: PathBuf,
    serial: String,
    pasta_de_saves: PathBuf,
    pub slot: u8,
    pub aviso: Option<String>,
    pub velocidade: u32,
    ultimo: std::time::Instant,
    jogado: f64,
    disco: PathBuf,
    porta: Option<PortaAberta>,
    modo_antes: bool,
}

impl Emulador {
    /// Um cartao por jogo: `cartoes/<serial>.mcd`, criado zerado na primeira vez. Cartao
    /// unico compartilhado enche com 15 blocos e obriga o usuario a apagar save alheio.
    pub fn novo(bios_bytes: Vec<u8>, serial: &str, config: &Config) -> Result<Self, String> {
        let bios = Bios::from_bytes(bios_bytes).map_err(|e| format!("BIOS invalida: {e:?}"))?;
        let mut bus = Bus::new(Ram::new(), bios);
        bus.sio_mut().connect_dualshock(true);

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
            memcard,
            serial: serial.to_string(),
            pasta_de_saves: PathBuf::from(&config.pasta_de_saves),
            slot: config.slot_inicial,
            aviso: None,
            velocidade: 1,
            ultimo: std::time::Instant::now(),
            jogado: 0.0,
            disco: PathBuf::new(),
            porta: None,
            modo_antes: false,
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
        self.bus.inject_disc_image(layout, bin);
        self.bus.cdrom_mut().insert_disc();
        self.disco = cue.to_path_buf();
        Ok(())
    }

    pub fn disco(&self) -> &Path {
        &self.disco
    }

    pub fn nome_do_disco(&self) -> String {
        self.disco
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default()
    }

    pub fn porta_aberta(&self) -> bool {
        self.bus.lid_open()
    }

    /// Abre a porta, poe o disco novo na bandeja e agenda o fechamento para ~1 s emulado
    /// depois: fechar no mesmo instante o jogo nem percebe que houve troca.
    pub fn troca_disco(&mut self, cue: &Path) {
        let (layout, bin) = match crate::disco::carrega(cue) {
            Ok(d) => d,
            Err(e) => {
                self.aviso = Some(e);
                return;
            }
        };
        self.bus.open_lid();
        self.bus.swap_disc_image(layout, bin);
        self.porta = Some(PortaAberta::desde(self.bus.total_cycles()));
        self.disco = cue.to_path_buf();
        self.aviso = Some(format!("trocando para {}", self.nome_do_disco()));
    }

    /// Estado salvo com a porta aberta volta com ela aberta e sem ninguem para fechar:
    /// sem agendamento, reagenda a partir de agora.
    fn cuida_da_porta(&mut self) {
        let agora = self.bus.total_cycles();
        match self.porta {
            Some(p) if p.deve_fechar(agora) => {
                if self.bus.lid_open() {
                    self.bus.close_lid();
                }
                self.porta = None;
            }
            Some(_) => {}
            None if self.bus.lid_open() => self.porta = Some(PortaAberta::desde(agora)),
            None => {}
        }
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
        let bytes = match snapshot::salva(&self.cpu, &self.bus, &self.serial) {
            Ok(b) => b,
            Err(e) => {
                self.aviso = Some(format!("slot {}: {e}", self.slot));
                return;
            }
        };
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
                Ok(()) => {
                    self.bus.sio_mut().connect_dualshock(true);
                    format!("slot {} carregado", self.slot)
                }
                Err(e) => format!("slot {}: {e}", self.slot),
            },
        );
    }

    pub fn slot_existe(&self, slot: u8) -> bool {
        self.caminho_do_slot(slot).exists()
    }

    /// Teclado e controle valem ao mesmo tempo: o pad do PS1 recebe a UNIAO dos dois,
    /// que e o que um jogador que larga o controle e pega o teclado espera.
    pub fn entrada(&mut self, ctx: &egui::Context, perfil: &Perfil, controle: &Leitura) {
        let modo_agora = controle.entradas.contains(&Entrada::Modo);
        let tecla_analog = ctx.input(|i| i.key_pressed(TECLA_ANALOG));
        if tecla_analog || apertou_agora(self.modo_antes, modo_agora) {
            self.aperta_analog();
        }
        self.modo_antes = modo_agora;

        let analogico = self.bus.sio().analog_mode();
        let mut botoes: u16 = 0xFFFF;
        for (tecla, bit) in TECLAS {
            if ctx.input(|i| i.key_down(tecla)) {
                botoes &= !(1u16 << bit);
            }
        }
        botoes &= perfil.palavra_no_modo(&controle.entradas, analogico);
        self.bus.sio_mut().set_buttons(botoes);

        let [esq, dir, cima, baixo] = STICK_ESQUERDO.map(|t| ctx.input(|i| i.key_down(t)));
        let teclado = Eixos {
            esquerdo_x: direcao(esq, dir),
            esquerdo_y: direcao(cima, baixo),
            ..Eixos::default()
        };
        self.bus
            .sio_mut()
            .set_sticks(controle.eixos.une(&teclado).sticks());
    }

    fn aperta_analog(&mut self) {
        let trocou = self.bus.sio_mut().press_analog_button();
        self.aviso = Some(match (trocou, self.bus.sio().analog_mode()) {
            (false, _) => "modo analogico travado pelo jogo".to_string(),
            (true, true) => "modo analogico (LED aceso)".to_string(),
            (true, false) => "modo digital".to_string(),
        });
    }

    pub fn modo_analogico(&self) -> bool {
        self.bus.sio().analog_mode()
    }

    pub fn vibracao(&self) -> Rumble {
        self.bus.sio().rumble()
    }

    pub fn quadro(&mut self, ganho: f32) {
        let agora = std::time::Instant::now();
        let dt = (agora - self.ultimo).as_secs_f64().min(0.05);
        self.ultimo = agora;
        self.jogado += dt;
        let alvo =
            self.bus.total_cycles() + (dt * CPU_HZ * f64::from(self.velocidade.max(1))) as u64;
        while self.bus.total_cycles() < alvo {
            self.cpu.step(&mut self.bus);
        }
        self.cuida_da_porta();
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
        let Some(fb) = self
            .bus
            .gpu()
            .framebuffer_for_display()
            .filter(|fb| fb.width > 0 && fb.height > 0)
        else {
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

    /// Tempo REAL com o jogo aberto; fast-forward 8x por uma hora conta uma hora.
    pub fn segundos_jogados(&self) -> u64 {
        self.jogado as u64
    }

    pub fn troca_velocidade(&mut self) {
        self.velocidade = sessao::proxima_velocidade(self.velocidade);
    }
}
