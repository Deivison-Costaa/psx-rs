use psx_core::app::troca;

use crate::{App, Tela};

impl App {
    pub(crate) fn tela_discos(&mut self, ui: &mut egui::Ui) {
        ui.heading("Trocar disco");
        let Some(emu) = self.emulador.as_ref() else {
            self.tela = Tela::Biblioteca;
            return;
        };
        let atual = emu.nome_do_disco();
        ui.label(format!("No drive: {atual}"));
        ui.small("A porta abre, o disco escolhido entra e ela fecha ~1 s depois.");
        ui.separator();

        let nomes: Vec<String> = self
            .discos
            .iter()
            .map(|c| {
                c.file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default()
            })
            .collect();
        let candidatos = troca::ordena_candidatos(&atual, &nomes);
        if candidatos.is_empty() {
            ui.label("Nenhum .cue encontrado na pasta de jogos nem ao lado do disco atual.");
        }
        let mut escolhido = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut outros_rotulados = false;
            for c in &candidatos {
                if !c.mesmo_jogo && !outros_rotulados {
                    ui.separator();
                    ui.small("Outros jogos");
                    outros_rotulados = true;
                }
                let nome = nomes.get(c.indice).map(String::as_str).unwrap_or("?");
                let rotulo = if c.atual {
                    format!("{nome} (no drive)")
                } else {
                    nome.to_string()
                };
                if ui.button(rotulo).clicked() {
                    escolhido = self.discos.get(c.indice).cloned();
                }
            }
        });
        if let Some(cue) = escolhido {
            if let Some(emu) = self.emulador.as_mut() {
                emu.troca_disco(&cue);
            }
            self.volta_do_menu();
        }
        if ui.button("Voltar").clicked() {
            self.volta_do_menu();
        }
    }
}
