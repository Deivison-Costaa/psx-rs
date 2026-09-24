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
use psx_core::app::exibicao::{MedidorDeQuadros, Sobreposicao, Toast};
use psx_core::app::input_map::{Entrada, Perfil};
use psx_core::app::pastas::Pastas;
use psx_core::app::pausa::MenuDePausa;
use psx_core::app::sessao::Recentes;

const TITULO: &str = "psx-rs";
const JANELA_INICIAL: [f32; 2] = [1024.0, 768.0];
const JANELA_MINIMA: [f32; 2] = [480.0, 360.0];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tela {
    Biblioteca,
    Jogando,
    Pausa,
    Saves,
    Estados,
    Controles,
    Ajustes,
    Discos,
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
    pub(crate) discos: Vec<PathBuf>,
    pub(crate) menu_de_pausa: Option<MenuDePausa>,
    pub(crate) ajuda_na_pausa: bool,
    pub(crate) toast: Option<Toast>,
    pub(crate) sobreposicao: Sobreposicao,
    pub(crate) textura: Option<egui::TextureHandle>,
    pub(crate) segura_entrada: bool,
    pub(crate) controle_antes: Vec<Entrada>,
    pub(crate) medidor: MedidorDeQuadros,
    pub(crate) medida: Option<(std::time::Instant, u64)>,
    titulo: String,
    tela_do_recado: Tela,
    pub(crate) pastas: Pastas,
    pub(crate) paineis: telas::Paineis,
}

impl App {
    fn novo(pastas: Pastas, sobrepoe_bios: Option<String>, recado: Option<String>) -> Self {
        let config_caminho = pastas.arquivo_de_config();
        let (mut config, erro) = ajustes::carrega(&config_caminho);
        if let Some(bios) = sobrepoe_bios {
            config.bios = ajustes::pasta_atual()
                .join(bios)
                .to_string_lossy()
                .to_string();
        }
        let perfil_arquivo = pastas.perfil_de_controle();
        let perfil = match std::fs::read_to_string(&perfil_arquivo) {
            Ok(texto) => Perfil::de_texto("Do arquivo", &texto),
            Err(_) => Perfil::padrao(),
        };
        let jogos = biblioteca::varre(std::path::Path::new(
            &pastas.efetiva(&config).pasta_de_jogos,
        ));
        App {
            tela: Tela::Biblioteca,
            config,
            config_caminho,
            jogos,
            emulador: None,
            erro,
            recado,
            em_execucao: None,
            gamepads: Gamepads::novo(),
            perfil,
            perfil_arquivo,
            discos: Vec::new(),
            menu_de_pausa: None,
            ajuda_na_pausa: false,
            toast: None,
            sobreposicao: Sobreposicao::default(),
            textura: None,
            segura_entrada: false,
            controle_antes: Vec::new(),
            medidor: MedidorDeQuadros::default(),
            medida: None,
            titulo: TITULO.to_string(),
            tela_do_recado: Tela::Biblioteca,
            pastas,
            paineis: telas::Paineis::default(),
        }
    }

    /// A config com todos os caminhos absolutos, resolvidos contra a pasta da config.
    pub(crate) fn efetiva(&self) -> Config {
        self.pastas.efetiva(&self.config)
    }

    pub(crate) fn revarre(&mut self) {
        self.jogos = biblioteca::varre(std::path::Path::new(&self.efetiva().pasta_de_jogos));
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
        let efetiva = self.efetiva();
        let bios = match std::fs::read(&efetiva.bios) {
            Ok(b) => b,
            Err(e) => {
                self.erro = Some(format!("lendo BIOS '{}': {e}", efetiva.bios));
                return;
            }
        };
        let mut emu = match Emulador::novo(bios, jogo.serial(), &efetiva) {
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
        self.menu_de_pausa = None;
        self.medidor = MedidorDeQuadros::default();
        self.medida = None;
        self.segura_entrada = true;
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
        self.menu_de_pausa = None;
        self.textura = None;
    }

    /// Varre na hora de abrir o menu, nao a cada quadro: sao chamadas de disco.
    pub(crate) fn abre_troca_de_disco(&mut self) {
        let Some(emu) = self.emulador.as_ref() else {
            return;
        };
        let atual = emu.disco();
        let pasta = atual.parent();
        let jogos = self.efetiva().pasta_de_jogos;
        let mut raizes = vec![std::path::Path::new(&jogos)];
        raizes.extend(pasta);
        raizes.extend(pasta.and_then(std::path::Path::parent));
        self.discos = disco::lista_cues(&raizes);
        self.tela = Tela::Discos;
    }

    pub(crate) fn recentes(&self) -> &Recentes {
        &self.config.recentes
    }

    /// Recado vale para a tela em que apareceu: trocar de tela o descarta.
    fn esquece_recado_de_outra_tela(&mut self) {
        if self.recado.is_some() && self.tela != self.tela_do_recado {
            self.recado = None;
        }
        if self.recado.is_none() {
            self.tela_do_recado = self.tela;
        }
    }

    pub(crate) fn volta_do_menu(&mut self) {
        self.tela = if self.emulador.is_none() {
            Tela::Biblioteca
        } else if self.menu_de_pausa.is_some() {
            Tela::Pausa
        } else {
            Tela::Jogando
        };
    }

    fn alterna_tela_cheia(&mut self, ctx: &egui::Context) {
        let pediu = ctx.input_mut(|i| {
            i.consume_key(egui::Modifiers::NONE, egui::Key::F11)
                || i.consume_key(egui::Modifiers::ALT, egui::Key::Enter)
        });
        if pediu {
            let cheia = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!cheia));
            self.segura_entrada = true;
        }
    }

    fn atualiza_titulo(&mut self, ctx: &egui::Context) {
        let titulo = match &self.em_execucao {
            Some(jogo) => format!("{TITULO} — {jogo}"),
            None => TITULO.to_string(),
        };
        if titulo != self.titulo {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(titulo.clone()));
            self.titulo = titulo;
        }
    }

    fn recolhe_aviso(&mut self, ctx: &egui::Context) {
        let agora = ctx.input(|i| i.time);
        if let Some(texto) = self.emulador.as_mut().and_then(|e| e.aviso.take()) {
            self.toast = Some(Toast::novo(&texto, agora));
        }
        if self.toast.as_ref().is_some_and(|t| t.expirou(agora)) {
            self.toast = None;
        }
    }
}

impl eframe::App for App {
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        self.navega_nos_menus(ctx, raw_input);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.alterna_tela_cheia(ctx);
        self.atualiza_titulo(ctx);
        self.recolhe_aviso(ctx);
        self.esquece_recado_de_outra_tela();
        let de_jogo = matches!(self.tela, Tela::Jogando | Tela::Pausa);
        let painel = if de_jogo {
            egui::CentralPanel::default().frame(egui::Frame::NONE.fill(egui::Color32::BLACK))
        } else {
            egui::CentralPanel::default()
        };
        painel.show(ctx, |ui| match self.tela {
            Tela::Biblioteca => self.tela_biblioteca(ui),
            Tela::Jogando => self.tela_jogando(ctx, ui),
            Tela::Pausa => self.tela_pausa(ctx, ui),
            Tela::Saves => self.tela_saves(ui),
            Tela::Estados => self.tela_estados(ui),
            Tela::Controles => self.tela_controles(ui),
            Tela::Ajustes => self.tela_ajustes(ui),
            Tela::Discos => self.tela_discos(ui),
        });
        self.desenha_toast(ctx);
        if de_jogo || self.toast.is_some() {
            ctx.request_repaint();
        }
    }
}

fn argumentos() -> (Option<PathBuf>, Option<String>) {
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
    (config, bios)
}

fn main() -> Result<(), eframe::Error> {
    let (config_caminho, bios) = argumentos();
    let atual = ajustes::pasta_atual();
    let pastas = ajustes::pastas(config_caminho.clone(), &atual);
    let recado = config_caminho
        .is_none()
        .then(|| ajustes::migra(&pastas, &atual))
        .flatten();
    let app = App::novo(pastas, bios, recado);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(TITULO)
            .with_inner_size(JANELA_INICIAL)
            .with_min_inner_size(JANELA_MINIMA),
        ..Default::default()
    };
    eframe::run_native(TITULO, options, Box::new(move |_cc| Ok(Box::new(app))))
}
