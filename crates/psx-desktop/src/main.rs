mod audio;
mod biblioteca;
mod disco;
mod emulador;

use std::path::PathBuf;

use biblioteca::Jogo;
use emulador::Emulador;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tela {
    Biblioteca,
    Jogando,
}

struct Argumentos {
    bios: String,
    memcard: Option<String>,
    jogos: PathBuf,
}

struct App {
    tela: Tela,
    args: Argumentos,
    jogos: Vec<Jogo>,
    emulador: Option<Emulador>,
    erro: Option<String>,
    em_execucao: Option<String>,
}

impl App {
    fn novo(args: Argumentos) -> Self {
        let jogos = biblioteca::varre(&args.jogos);
        App {
            tela: Tela::Biblioteca,
            args,
            jogos,
            emulador: None,
            erro: None,
            em_execucao: None,
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
        let mut emu = match Emulador::novo(bios, self.args.memcard.clone()) {
            Ok(e) => e,
            Err(e) => {
                self.erro = Some(e);
                return;
            }
        };
        if let Err(e) = emu.insere_disco(&jogo.cue, jogo.serial()) {
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
        let Some(emu) = self.emulador.as_mut() else {
            self.tela = Tela::Biblioteca;
            return;
        };
        emu.teclado(ctx);
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
            ui.small("Esc: biblioteca · F5/F8: salvar/carregar · F6/F7: slot");
        });
        if let Some(aviso) = &emu.aviso {
            ui.small(aviso.clone());
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.emulador = None;
            self.em_execucao = None;
            self.tela = Tela::Biblioteca;
        }
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
    let mut i = 0;
    while i < brutos.len() {
        match brutos[i].as_str() {
            "--jogos" if i + 1 < brutos.len() => {
                jogos = Some(PathBuf::from(&brutos[i + 1]));
                i += 2;
            }
            outro => {
                posicionais.push(outro.to_string());
                i += 1;
            }
        }
    }
    let bios = posicionais
        .first()
        .cloned()
        .ok_or_else(|| "Uso: psx-desktop <BIOS.bin> [cartao.mcd] [--jogos <pasta>]".to_string())?;
    Ok(Argumentos {
        bios,
        memcard: posicionais.get(1).cloned(),
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
