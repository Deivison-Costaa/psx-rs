use psx_core::app::sessao::formata_tempo;

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
}
