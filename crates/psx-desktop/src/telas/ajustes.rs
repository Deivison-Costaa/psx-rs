use psx_core::app::config::VOLUME_MAX;
use psx_core::app::exibicao::ModoDeImagem;

use crate::App;

impl App {
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
            ui.label("Pasta de cartões");
            ui.text_edit_singleline(&mut self.config.pasta_de_cartoes);
            ui.end_row();
            ui.label("Pasta de save states");
            ui.text_edit_singleline(&mut self.config.pasta_de_saves);
            ui.end_row();
            ui.label("Imagem");
            ui.horizontal(|ui| {
                for modo in ModoDeImagem::TODOS {
                    ui.radio_value(&mut self.config.modo_de_imagem, modo, modo.rotulo());
                }
            });
            ui.end_row();
            ui.label("Filtro linear");
            ui.checkbox(&mut self.config.filtro_linear, "suavizar a imagem");
            ui.end_row();
            ui.label("Áudio");
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
        ui.small("Imagem e filtro valem no próximo quadro; o resto, no próximo jogo.");
    }
}
