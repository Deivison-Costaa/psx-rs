mod ajustes;
mod audio;
mod biblioteca;
mod disco;
mod emulador;
mod gamepad;
mod telas;

use std::path::PathBuf;

use biblioteca::Jogo;
use emulador::Emulador;
use gamepad::Gamepads;
use psx_core::app::config::Config;
use psx_core::app::input_map::Perfil;
use psx_core::app::sessao::Recentes;

const PERFIL_DE_CONTROLE: &str = "controles.txt";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tela {
    Biblioteca,
    Jogando,
    Saves,
    Controles,
    Ajustes,
}

pub(crate) struct App {
    pub(crate) tela: Tela,
    pub(crate) config: Config,
    pub(crate) config_caminho: PathBuf,
    pub(crate) jogos: Vec<Jogo>,
    pub(crate) emulador: Option<Emulador>,
    pub(crate) erro: Option<String>,
    pub(crate) recado: Option<String>,
    pub(crate) em_execucao: Option<String>,
    pub(crate) gamepads: Gamepads,
    pub(crate) perfil: Perfil,
    pub(crate) perfil_arquivo: PathBuf,
}

impl App {
    fn novo(config_caminho: PathBuf, sobrepoe_bios: Option<String>) -> Self {
        let (mut config, erro) = ajustes::carrega(&config_caminho);
        if let Some(bios) = sobrepoe_bios {
            config.bios = bios;
        }
        let perfil_arquivo = PathBuf::from(PERFIL_DE_CONTROLE);
        let perfil = match std::fs::read_to_string(&perfil_arquivo) {
            Ok(texto) => Perfil::de_texto("Do arquivo", &texto),
            Err(_) => Perfil::padrao(),
        };
        let jogos = biblioteca::varre(std::path::Path::new(&config.pasta_de_jogos));
        App {
            tela: Tela::Biblioteca,
            config,
            config_caminho,
            jogos,
            emulador: None,
            erro,
            recado: None,
            em_execucao: None,
            gamepads: Gamepads::novo(),
            perfil,
            perfil_arquivo,
        }
    }

    pub(crate) fn revarre(&mut self) {
        self.jogos = biblioteca::varre(std::path::Path::new(&self.config.pasta_de_jogos));
    }

    pub(crate) fn grava_perfil(&mut self) {
        self.recado = Some(
            match std::fs::write(&self.perfil_arquivo, self.perfil.para_texto()) {
                Ok(()) => format!("perfil gravado em {}", self.perfil_arquivo.display()),
                Err(e) => format!("gravando perfil: {e}"),
            },
        );
    }

    pub(crate) fn grava_config(&mut self) {
        self.config = self.config.ajustada();
        self.recado = Some(match ajustes::grava(&self.config_caminho, &self.config) {
            Ok(()) => format!("ajustes gravados em {}", self.config_caminho.display()),
            Err(e) => e,
        });
    }

    pub(crate) fn inicia(&mut self, indice: usize) {
        let Some(jogo) = self.jogos.get(indice).cloned() else {
            return;
        };
        let bios = match std::fs::read(&self.config.bios) {
            Ok(b) => b,
            Err(e) => {
                self.erro = Some(format!("lendo BIOS '{}': {e}", self.config.bios));
                return;
            }
        };
        let mut emu = match Emulador::novo(bios, jogo.serial(), &self.config) {
            Ok(e) => e,
            Err(e) => {
                self.erro = Some(e);
                return;
            }
        };
        if let Err(e) = emu.insere_disco(&jogo.cue) {
            self.erro = Some(e);
            return;
        }
        self.em_execucao = Some(jogo.titulo.clone());
        self.emulador = Some(emu);
        self.erro = None;
        self.tela = Tela::Jogando;
    }

    /// Segundos desde a epoca. Fica no frontend porque o `psx-core` nao le relogio (R3).
    pub(crate) fn agora() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    /// Chamado ao sair do jogo: acumula o tempo jogado e grava, para a lista de recentes
    /// sobreviver a fechar o app.
    pub(crate) fn encerra_partida(&mut self) {
        let Some(emu) = self.emulador.take() else {
            return;
        };
        let titulo = self.em_execucao.clone().unwrap_or_default();
        let novas = self.config.recentes.registra(
            emu.serial(),
            &titulo,
            emu.segundos_jogados(),
            Self::agora(),
        );
        if novas != self.config.recentes {
            self.config.recentes = novas;
            let _ = ajustes::grava(&self.config_caminho, &self.config);
        }
        self.em_execucao = None;
    }

    pub(crate) fn recentes(&self) -> &Recentes {
        &self.config.recentes
    }

    pub(crate) fn volta_do_menu(&mut self) {
        self.tela = if self.emulador.is_some() {
            Tela::Jogando
        } else {
            Tela::Biblioteca
        };
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| match self.tela {
            Tela::Biblioteca => self.tela_biblioteca(ui),
            Tela::Jogando => self.tela_jogando(ctx, ui),
            Tela::Saves => self.tela_saves(ui),
            Tela::Controles => self.tela_controles(ui),
            Tela::Ajustes => self.tela_ajustes(ui),
        });
        if self.tela == Tela::Jogando {
            ctx.request_repaint();
        }
    }
}

fn argumentos() -> (PathBuf, Option<String>) {
    let brutos: Vec<String> = std::env::args().skip(1).collect();
    let mut config = None;
    let mut bios = None;
    let mut i = 0;
    while i < brutos.len() {
        match brutos[i].as_str() {
            "--config" if i + 1 < brutos.len() => {
                config = Some(PathBuf::from(&brutos[i + 1]));
                i += 2;
            }
            "--bios" if i + 1 < brutos.len() => {
                bios = Some(brutos[i + 1].clone());
                i += 2;
            }
            outro => {
                if bios.is_none() && !outro.starts_with("--") {
                    bios = Some(outro.to_string());
                }
                i += 1;
            }
        }
    }
    (config.unwrap_or_else(ajustes::caminho_padrao), bios)
}

fn main() -> Result<(), eframe::Error> {
    let (config_caminho, bios) = argumentos();
    let app = App::novo(config_caminho, bios);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([840.0, 640.0]),
        ..Default::default()
    };
    eframe::run_native("psx-rs", options, Box::new(move |_cc| Ok(Box::new(app))))
}
