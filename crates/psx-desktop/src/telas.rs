use psx_core::app::config::{ESCALA_MAX, ESCALA_MIN, VOLUME_MAX};
use psx_core::app::input_map::{self, Entrada, Perfil};
use psx_core::app::sessao::formata_tempo;
use psx_core::pad_script;

use crate::emulador::{self, Emulador};
use crate::{App, Tela};

impl App {
    pub(crate) fn tela_biblioteca(&mut self, ui: &mut egui::Ui) {
        ui.heading("Biblioteca");
        ui.horizontal(|ui| {
            if ui.button("Atualizar").clicked() {
                self.revarre();
            }
            if ui.button("Ajustes").clicked() {
                self.tela = Tela::Ajustes;
            }
            if ui.button("Controles").clicked() {
                self.tela = Tela::Controles;
            }
            ui.label(format!(
                "{} jogo(s) em {}",
                self.jogos.len(),
                self.config.pasta_de_jogos
            ));
        });
        self.avisos(ui);
        ui.separator();

        if self.jogos.is_empty() {
            ui.label("Nenhum .cue encontrado. Aponte a pasta de jogos em Ajustes.");
            return;
        }

        let recentes = self.recentes().clone();
        if let Some(topo) = recentes.itens().first() {
            ui.small(format!(
                "Ultimo jogado: {} ({})",
                topo.titulo,
                formata_tempo(topo.segundos)
            ));
            ui.separator();
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
                        let tempo = recentes.tempo_de(jogo.serial());
                        if tempo > 0 {
                            ui.small(format!(
                                "{} · jogado {}",
                                jogo.detalhe(),
                                formata_tempo(tempo)
                            ));
                        } else {
                            ui.small(jogo.detalhe());
                        }
                    });
                });
                ui.separator();
            }
        });
        if let Some(i) = escolhido {
            self.inicia(i);
        }
    }

    fn avisos(&self, ui: &mut egui::Ui) {
        if let Some(erro) = &self.erro {
            ui.colored_label(egui::Color32::RED, erro);
        }
        if let Some(recado) = &self.recado {
            ui.small(recado);
        }
    }

    pub(crate) fn tela_jogando(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        if self.emulador.is_none() {
            self.tela = Tela::Biblioteca;
            return;
        }
        let do_controle = self.gamepads.pressionados();
        let ganho = self.config.ganho();
        let escala = self.config.escala as f32;
        let filtro = if self.config.filtro_linear {
            egui::TextureOptions::LINEAR
        } else {
            egui::TextureOptions::NEAREST
        };
        let Some(emu) = self.emulador.as_mut() else {
            return;
        };
        emu.entrada(ctx, &self.perfil, &do_controle);
        Self::atalhos_de_estado(ctx, emu);
        emu.quadro(ganho);

        match emu.textura() {
            Some(imagem) => {
                let tamanho = egui::vec2(imagem.width() as f32, imagem.height() as f32) * escala;
                let handle = ctx.load_texture("framebuffer", imagem.clone(), filtro);
                ui.add(egui::Image::new(&handle).fit_to_exact_size(tamanho));
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
            if emu.velocidade > 1 {
                ui.small(format!("{}x", emu.velocidade));
            }
            ui.small(formata_tempo(emu.segundos_jogados()));
        });
        ui.small(
            "Esc: sair · F5/F8: salvar/carregar · F6/F7: slot · F9: cartao · F10: controles              · F11: ajustes · F12: velocidade",
        );
        if let Some(aviso) = &emu.aviso {
            ui.small(aviso.clone());
        }

        if ctx.input(|i| i.key_pressed(egui::Key::F12)) {
            emu.troca_velocidade();
        }
        let (f9, f10, f11, esc) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::F9),
                i.key_pressed(egui::Key::F10),
                i.key_pressed(egui::Key::F11),
                i.key_pressed(egui::Key::Escape),
            )
        });
        if f9 {
            self.tela = Tela::Saves;
        }
        if f10 {
            self.tela = Tela::Controles;
        }
        if f11 {
            self.tela = Tela::Ajustes;
        }
        if esc {
            self.encerra_partida();
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

    pub(crate) fn tela_saves(&mut self, ui: &mut egui::Ui) {
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
        if ui.button("Voltar").clicked() {
            self.volta_do_menu();
        }
    }

    pub(crate) fn tela_controles(&mut self, ui: &mut egui::Ui) {
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
        self.avisos(ui);
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
            self.volta_do_menu();
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

    pub(crate) fn tela_ajustes(&mut self, ui: &mut egui::Ui) {
        ui.heading("Ajustes");
        ui.small(format!("Arquivo: {}", self.config_caminho.display()));
        for problema in self.config.valida() {
            ui.colored_label(egui::Color32::from_rgb(220, 160, 60), problema);
        }
        self.avisos(ui);
        ui.separator();

        egui::Grid::new("ajustes").num_columns(2).show(ui, |ui| {
            ui.label("BIOS");
            ui.text_edit_singleline(&mut self.config.bios);
            ui.end_row();
            ui.label("Pasta de jogos");
            ui.text_edit_singleline(&mut self.config.pasta_de_jogos);
            ui.end_row();
            ui.label("Pasta de cartoes");
            ui.text_edit_singleline(&mut self.config.pasta_de_cartoes);
            ui.end_row();
            ui.label("Pasta de save states");
            ui.text_edit_singleline(&mut self.config.pasta_de_saves);
            ui.end_row();
            ui.label("Escala da imagem");
            ui.add(egui::Slider::new(
                &mut self.config.escala,
                ESCALA_MIN..=ESCALA_MAX,
            ));
            ui.end_row();
            ui.label("Filtro linear");
            ui.checkbox(&mut self.config.filtro_linear, "suavizar a imagem");
            ui.end_row();
            ui.label("Audio");
            ui.checkbox(&mut self.config.audio_ligado, "ligado");
            ui.end_row();
            ui.label("Volume");
            ui.add(egui::Slider::new(&mut self.config.volume, 0..=VOLUME_MAX));
            ui.end_row();
        });

        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("Gravar").clicked() {
                self.grava_config();
                self.revarre();
            }
            if ui.button("Voltar").clicked() {
                self.volta_do_menu();
            }
        });
        ui.small("Ajuste de escala e de filtro vale no proximo quadro; o resto, no proximo jogo.");
    }
}
