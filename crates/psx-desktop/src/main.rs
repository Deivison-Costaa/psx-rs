mod audio;
mod biblioteca;
mod disco;
mod emulador;
mod gamepad;

use std::path::PathBuf;

use biblioteca::Jogo;
use emulador::Emulador;
use gamepad::Gamepads;
use psx_core::app::input_map::{self, Entrada, Perfil};
use psx_core::pad_script;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tela {
    Biblioteca,
    Jogando,
    Saves,
    Controles,
}

struct Argumentos {
    bios: String,
    cartoes: PathBuf,
    jogos: PathBuf,
}

struct App {
    tela: Tela,
    args: Argumentos,
    jogos: Vec<Jogo>,
    emulador: Option<Emulador>,
    erro: Option<String>,
    em_execucao: Option<String>,
    gamepads: Gamepads,
    perfil: Perfil,
    perfil_arquivo: PathBuf,
}

impl App {
    fn novo(args: Argumentos) -> Self {
        let jogos = biblioteca::varre(&args.jogos);
        let perfil_arquivo = args.cartoes.with_file_name("controles.txt");
        let perfil = match std::fs::read_to_string(&perfil_arquivo) {
            Ok(texto) => Perfil::de_texto("Do arquivo", &texto),
            Err(_) => Perfil::padrao(),
        };
        App {
            tela: Tela::Biblioteca,
            args,
            jogos,
            emulador: None,
            erro: None,
            em_execucao: None,
            gamepads: Gamepads::novo(),
            perfil,
            perfil_arquivo,
        }
    }

    fn grava_perfil(&mut self) {
        if let Some(pai) = self.perfil_arquivo.parent() {
            let _ = std::fs::create_dir_all(pai);
        }
        if let Err(e) = std::fs::write(&self.perfil_arquivo, self.perfil.para_texto()) {
            self.erro = Some(format!("gravando perfil de controle: {e}"));
        }
    }

    fn inicia(&mut self, indice: usize) {
        let Some(jogo) = self.jogos.get(indice).cloned() else {
            return;
        };
        let bios = match std::fs::read(&self.args.bios) {
            Ok(b) => b,
            Err(e) => {
                self.erro = Some(format!("lendo BIOS '{}': {e}", self.args.bios));
                return;
            }
        };
        let mut emu = match Emulador::novo(bios, jogo.serial(), &self.args.cartoes) {
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

    fn tela_biblioteca(&mut self, ui: &mut egui::Ui) {
        ui.heading("Biblioteca");
        ui.label(format!("Pasta: {}", self.args.jogos.display()));
        ui.horizontal(|ui| {
            if ui.button("Atualizar").clicked() {
                self.jogos = biblioteca::varre(&self.args.jogos);
            }
            ui.label(format!("{} jogo(s)", self.jogos.len()));
        });
        if let Some(erro) = &self.erro {
            ui.colored_label(egui::Color32::RED, erro);
        }
        ui.separator();

        if self.jogos.is_empty() {
            ui.label("Nenhum .cue encontrado. Use --jogos <pasta> para apontar a biblioteca.");
            return;
        }

        let mut escolhido = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (i, jogo) in self.jogos.iter().enumerate() {
                ui.horizontal(|ui| {
                    if ui.button("Jogar").clicked() {
                        escolhido = Some(i);
                    }
                    ui.vertical(|ui| {
                        ui.strong(&jogo.titulo);
                        ui.small(jogo.detalhe());
                    });
                });
                ui.separator();
            }
        });
        if let Some(i) = escolhido {
            self.inicia(i);
        }
    }

    fn tela_jogando(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        if self.emulador.is_none() {
            self.tela = Tela::Biblioteca;
            return;
        }
        let do_controle = self.gamepads.pressionados();
        let Some(emu) = self.emulador.as_mut() else {
            return;
        };
        emu.entrada(ctx, &self.perfil, &do_controle);
        Self::atalhos_de_estado(ctx, emu);
        emu.quadro();

        match emu.textura() {
            Some(imagem) => {
                let handle =
                    ctx.load_texture("framebuffer", imagem.clone(), egui::TextureOptions::NEAREST);
                ui.image(&handle);
            }
            None => {
                ui.label("Display desligado");
            }
        }
        ui.horizontal(|ui| {
            if let Some(titulo) = &self.em_execucao {
                ui.small(titulo);
            }
            if emu.audio_ativo() {
                ui.small(format!("audio {} Hz", emu.audio_hz()));
            } else {
                ui.small("audio desligado (sem dispositivo de saida)");
            }
            let marca = if emu.slot_existe(emu.slot) { "*" } else { "" };
            ui.small(format!("slot {}{marca}", emu.slot));
            ui.small("Esc: sair · F5/F8: slot · F6/F7: trocar · F9: cartao · F10: controles");
        });
        if let Some(aviso) = &emu.aviso {
            ui.small(aviso.clone());
        }

        if ctx.input(|i| i.key_pressed(egui::Key::F9)) {
            self.tela = Tela::Saves;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F10)) {
            self.tela = Tela::Controles;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.emulador = None;
            self.em_execucao = None;
            self.tela = Tela::Biblioteca;
        }
    }

    fn tela_saves(&mut self, ui: &mut egui::Ui) {
        ui.heading("Memory card");
        let Some(emu) = self.emulador.as_ref() else {
            self.tela = Tela::Biblioteca;
            return;
        };
        ui.small(format!("Arquivo: {}", emu.caminho_do_cartao().display()));
        let saves = emu.saves_do_cartao();
        let ocupados: u8 = saves.iter().map(|s| s.blocos).sum();
        ui.label(format!(
            "{} arquivo(s), {ocupados} de 15 blocos usados",
            saves.len()
        ));
        ui.separator();
        if saves.is_empty() {
            ui.label("Cartao vazio. O jogo precisa gravar uma vez para o arquivo aparecer.");
        }
        egui::ScrollArea::vertical().show(ui, |ui| {
            for save in &saves {
                let titulo = if save.titulo.is_empty() {
                    "(sem titulo)"
                } else {
                    &save.titulo
                };
                ui.strong(titulo);
                ui.small(format!(
                    "{} · bloco {} · {} bloco(s)",
                    save.nome, save.bloco, save.blocos
                ));
                ui.separator();
            }
        });
        if ui.button("Voltar ao jogo").clicked() {
            self.tela = Tela::Jogando;
        }
    }

    fn tela_controles(&mut self, ui: &mut egui::Ui) {
        ui.heading("Controles");
        let nomes = self.gamepads.nomes();
        if nomes.is_empty() {
            ui.label("Nenhum controle detectado. O teclado continua valendo.");
        } else {
            ui.label(format!("Conectado(s): {}", nomes.join(", ")));
        }
        ui.horizontal(|ui| {
            ui.label(format!("Perfil: {}", self.perfil.nome));
            if ui.button("Padrao").clicked() {
                self.perfil = Perfil::padrao();
            }
            if ui.button("Faces trocadas").clicked() {
                self.perfil = Perfil::faces_trocadas();
            }
            if ui.button("Gravar").clicked() {
                self.grava_perfil();
            }
        });
        ui.small(format!("Arquivo: {}", self.perfil_arquivo.display()));
        ui.separator();

        let mut mudanca: Option<(Entrada, Option<&'static str>)> = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            for entrada in Self::entradas_mapeaveis() {
                let atual = self.perfil.nome_do_botao(entrada);
                ui.horizontal(|ui| {
                    ui.label(entrada.nome());
                    egui::ComboBox::from_id_salt(entrada.nome())
                        .selected_text(atual.unwrap_or("(nenhum)"))
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(atual.is_none(), "(nenhum)").clicked() {
                                mudanca = Some((entrada, None));
                            }
                            for bit in 0..16u8 {
                                let Some(nome) = pad_script::button_name(bit) else {
                                    continue;
                                };
                                if ui.selectable_label(atual == Some(nome), nome).clicked() {
                                    mudanca = Some((entrada, Some(nome)));
                                }
                            }
                        });
                });
            }
        });

        match mudanca {
            Some((entrada, Some(nome))) => {
                if let Ok(novo) = self.perfil.liga(entrada, nome) {
                    self.perfil = novo;
                }
            }
            Some((entrada, None)) => self.perfil = self.perfil.desliga(entrada),
            None => {}
        }

        if ui.button("Voltar").clicked() {
            self.tela = if self.emulador.is_some() {
                Tela::Jogando
            } else {
                Tela::Biblioteca
            };
        }
    }

    fn entradas_mapeaveis() -> Vec<Entrada> {
        let mut fora = input_map::TODAS_FIXAS.to_vec();
        for n in 0..2u8 {
            fora.push(Entrada::EixoNegativo(n));
            fora.push(Entrada::EixoPositivo(n));
        }
        fora
    }

    fn atalhos_de_estado(ctx: &egui::Context, emu: &mut Emulador) {
        let (f5, f6, f7, f8) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::F5),
                i.key_pressed(egui::Key::F6),
                i.key_pressed(egui::Key::F7),
                i.key_pressed(egui::Key::F8),
            )
        });
        if f6 {
            emu.slot = (emu.slot + emulador::SLOTS - 1) % emulador::SLOTS;
        }
        if f7 {
            emu.slot = (emu.slot + 1) % emulador::SLOTS;
        }
        if f5 {
            emu.salva_estado();
        }
        if f8 {
            emu.carrega_estado();
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| match self.tela {
            Tela::Biblioteca => self.tela_biblioteca(ui),
            Tela::Jogando => self.tela_jogando(ctx, ui),
            Tela::Saves => self.tela_saves(ui),
            Tela::Controles => self.tela_controles(ui),
        });
        if self.tela == Tela::Jogando {
            ctx.request_repaint();
        }
    }
}

fn argumentos() -> Result<Argumentos, String> {
    let brutos: Vec<String> = std::env::args().skip(1).collect();
    let mut posicionais = Vec::new();
    let mut jogos = None;
    let mut cartoes = None;
    let mut i = 0;
    while i < brutos.len() {
        match brutos[i].as_str() {
            "--jogos" if i + 1 < brutos.len() => {
                jogos = Some(PathBuf::from(&brutos[i + 1]));
                i += 2;
            }
            "--cartoes" if i + 1 < brutos.len() => {
                cartoes = Some(PathBuf::from(&brutos[i + 1]));
                i += 2;
            }
            outro => {
                posicionais.push(outro.to_string());
                i += 1;
            }
        }
    }
    let bios = posicionais.first().cloned().ok_or_else(|| {
        "Uso: psx-desktop <BIOS.bin> [--jogos <pasta>] [--cartoes <pasta>]".to_string()
    })?;
    Ok(Argumentos {
        bios,
        cartoes: cartoes.unwrap_or_else(|| PathBuf::from("cartoes")),
        jogos: jogos.unwrap_or_else(|| PathBuf::from(".")),
    })
}

fn main() -> Result<(), eframe::Error> {
    let args = match argumentos() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };

    let app = App::novo(args);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([720.0, 560.0]),
        ..Default::default()
    };
    eframe::run_native("psx-rs", options, Box::new(move |_cc| Ok(Box::new(app))))
}
