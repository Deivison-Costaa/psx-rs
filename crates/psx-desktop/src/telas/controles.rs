use psx_core::app::input_map::{self, Entrada, Perfil};
use psx_core::pad_script;

use crate::App;

impl App {
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
}
