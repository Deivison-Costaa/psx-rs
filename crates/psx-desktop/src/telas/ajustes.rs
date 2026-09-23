use psx_core::app::config::{ESCALA_MAX, ESCALA_MIN, VOLUME_MAX};

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
