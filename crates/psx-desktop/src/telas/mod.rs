mod ajustes;
mod biblioteca;
mod controles;
mod discos;
mod estados;
mod jogando;
mod saves;

/// Estado de interface que sobrevive entre quadros (busca, slot escolhido, confirmações).
#[derive(Default)]
pub(crate) struct Paineis {
    pub(crate) biblioteca: biblioteca::Painel,
    pub(crate) estados: estados::Painel,
    pub(crate) cartoes: saves::Painel,
}

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
