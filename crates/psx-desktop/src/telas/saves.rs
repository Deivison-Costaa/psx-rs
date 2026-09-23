use crate::{App, Tela};

impl App {
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
}
