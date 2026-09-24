mod ajustes;
mod biblioteca;
mod controles;
mod discos;
mod jogando;
mod pausa;
mod saves;

use crate::App;

impl App {
    pub(crate) fn avisos(&self, ui: &mut egui::Ui) {
        if let Some(erro) = &self.erro {
            ui.colored_label(egui::Color32::RED, erro);
        }
        if let Some(recado) = &self.recado {
            ui.small(recado);
        }
    }
}
