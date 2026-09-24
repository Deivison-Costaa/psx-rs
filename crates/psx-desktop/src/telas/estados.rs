use std::path::PathBuf;
use std::time::SystemTime;

use psx_core::app::estados::{self, MINIATURA_ALTURA, MINIATURA_LARGURA, SLOTS};

use crate::emulador;
use crate::{App, Tela};

const COLUNAS: u8 = 5;
const LARGURA: f32 = MINIATURA_LARGURA as f32;
const ALTURA: f32 = MINIATURA_ALTURA as f32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Alvo {
    serial: String,
    titulo: String,
    pasta: PathBuf,
    jogo: Option<usize>,
}

struct Slot {
    gravado: Option<SystemTime>,
    textura: Option<egui::TextureHandle>,
}

#[derive(Default)]
pub(crate) struct Painel {
    da_biblioteca: Option<Alvo>,
    carregado: Option<Alvo>,
    slots: Vec<Slot>,
    escolhido: Option<u8>,
    confirmar_apagar: bool,
    deslocamento: Option<i64>,
}

enum Acao {
    Escolhe(u8),
    Salva(u8),
    Carrega(u8),
    Apaga(u8),
    ConfirmaApagar,
    Cancela,
    Volta,
}

/// `date +%z` uma vez: sem crate de fuso horario, e o jeito barato de achar a hora local.
fn deslocamento_local() -> i64 {
    std::process::Command::new("date")
        .arg("+%z")
        .output()
        .ok()
        .and_then(|s| estados::deslocamento_de(&String::from_utf8_lossy(&s.stdout)))
        .unwrap_or(0)
}

fn gravado_em(alvo: &Alvo, slot: u8) -> Option<SystemTime> {
    let caminho = emulador::caminho_do_estado(&alvo.pasta, &alvo.serial, slot);
    std::fs::metadata(caminho).and_then(|m| m.modified()).ok()
}

impl Painel {
    /// Relê do disco só quando algum `.state` mudou (data de modificação diferente).
    fn atualiza(&mut self, ctx: &egui::Context, alvo: &Alvo) {
        let datas: Vec<Option<SystemTime>> = (0..SLOTS).map(|s| gravado_em(alvo, s)).collect();
        let atuais: Vec<Option<SystemTime>> = self.slots.iter().map(|s| s.gravado).collect();
        if self.carregado.as_ref() == Some(alvo) && datas == atuais {
            return;
        }
        self.slots = datas
            .into_iter()
            .enumerate()
            .map(|(i, gravado)| {
                let slot = i as u8;
                let miniatura = emulador::caminho_da_miniatura(&alvo.pasta, &alvo.serial, slot);
                let textura = gravado.and(emulador::le_png(&miniatura)).map(|img| {
                    ctx.load_texture(format!("estado-{slot}"), img, egui::TextureOptions::LINEAR)
                });
                Slot { gravado, textura }
            })
            .collect();
        if self.carregado.as_ref() != Some(alvo) {
            self.escolhido = None;
            self.confirmar_apagar = false;
        }
        self.carregado = Some(alvo.clone());
    }

    fn data(&mut self, slot: u8) -> Option<String> {
        let gravado = self.slots.get(usize::from(slot))?.gravado?;
        let segundos = gravado
            .duration_since(SystemTime::UNIX_EPOCH)
            .ok()?
            .as_secs();
        let fuso = *self.deslocamento.get_or_insert_with(deslocamento_local);
        Some(estados::data_hora(segundos as i64 + fuso))
    }

    fn ocupado(&self, slot: u8) -> bool {
        self.slots
            .get(usize::from(slot))
            .is_some_and(|s| s.gravado.is_some())
    }

    fn cartao(&mut self, ui: &mut egui::Ui, slot: u8, com_jogo: bool) -> Option<Acao> {
        let data = self.data(slot);
        let selecionado = self.escolhido == Some(slot);
        let textura = self
            .slots
            .get(usize::from(slot))
            .and_then(|s| s.textura.clone());
        let mut acao = None;
        ui.vertical(|ui| {
            let (rect, resp) =
                ui.allocate_exact_size(egui::vec2(LARGURA, ALTURA), egui::Sense::click());
            let visuais = ui.style().interact_selectable(&resp, selecionado);
            match (&textura, data.is_some()) {
                (Some(t), _) => {
                    egui::Image::new(t).paint_at(ui, rect);
                }
                (None, ocupado) => {
                    ui.painter()
                        .rect_filled(rect, 4.0, ui.visuals().extreme_bg_color);
                    let texto = match (ocupado, com_jogo) {
                        (true, _) => "sem miniatura",
                        (false, true) => "vazio: clique para salvar",
                        (false, false) => "vazio",
                    };
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        texto,
                        egui::FontId::proportional(13.0),
                        ui.visuals().weak_text_color(),
                    );
                }
            }
            let borda = if selecionado {
                egui::Stroke::new(3.0_f32, ui.visuals().selection.bg_fill)
            } else {
                visuais.bg_stroke
            };
            ui.painter()
                .rect_stroke(rect, 4.0, borda, egui::StrokeKind::Inside);
            ui.strong(format!("Slot {slot}"));
            ui.small(data.clone().unwrap_or_else(|| "—".to_string()));
            if resp.clicked() {
                acao = Some(match (data.is_some(), com_jogo) {
                    (false, true) => Acao::Salva(slot),
                    _ => Acao::Escolhe(slot),
                });
            }
        });
        acao
    }
}

impl App {
    fn alvo_dos_estados(&self) -> Option<Alvo> {
        if let Some(emu) = self.emulador.as_ref() {
            return Some(Alvo {
                serial: emu.serial().to_string(),
                titulo: self.em_execucao.clone().unwrap_or_default(),
                pasta: emu.pasta_de_saves().to_path_buf(),
                jogo: None,
            });
        }
        self.paineis.estados.da_biblioteca.clone()
    }

    /// Estados de um jogo da biblioteca, sem abrir o jogo: carregar um slot abre o jogo.
    pub(crate) fn abre_estados_do_jogo(&mut self, indice: usize) {
        let Some(jogo) = self.jogos.get(indice) else {
            return;
        };
        self.paineis.estados.da_biblioteca = Some(Alvo {
            serial: jogo.serial().to_string(),
            titulo: jogo.titulo.clone(),
            pasta: PathBuf::from(self.efetiva().pasta_de_saves),
            jogo: Some(indice),
        });
        self.tela = Tela::Estados;
    }

    pub(crate) fn tela_estados(&mut self, ui: &mut egui::Ui) {
        let Some(alvo) = self.alvo_dos_estados() else {
            self.tela = Tela::Biblioteca;
            return;
        };
        let com_jogo = self.emulador.is_some();
        self.paineis.estados.atualiza(ui.ctx(), &alvo);

        ui.heading(format!("Estados salvos — {}", alvo.titulo));
        ui.small(if com_jogo {
            "Clique num slot vazio para salvar; num ocupado, para carregar, sobrescrever ou apagar. No jogo: F5 salva, F8 carrega, F6/F7 trocam de slot."
        } else {
            "Carregar um estado abre o jogo nele. Para salvar, abra o jogo e use F4 ou F5."
        });
        self.avisos(ui);
        if let Some(aviso) = self.emulador.as_ref().and_then(|e| e.aviso.clone()) {
            ui.small(aviso);
        }
        ui.separator();

        let mut acao = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("slots")
                .spacing([12.0, 12.0])
                .show(ui, |ui| {
                    for slot in 0..SLOTS {
                        if let Some(a) = self.paineis.estados.cartao(ui, slot, com_jogo) {
                            acao = Some(a);
                        }
                        if slot % COLUNAS == COLUNAS - 1 {
                            ui.end_row();
                        }
                    }
                });
            ui.separator();
            if let Some(a) = self.pergunta(ui, com_jogo) {
                acao = Some(a);
            }
            if ui.button("Voltar").clicked() {
                acao = Some(Acao::Volta);
            }
        });
        if ui.ctx().input(|i| i.key_pressed(egui::Key::Escape)) {
            acao = Some(Acao::Volta);
        }
        if let Some(a) = acao {
            self.aplica(a, &alvo);
        }
    }

    fn pergunta(&mut self, ui: &mut egui::Ui, com_jogo: bool) -> Option<Acao> {
        let slot = self.paineis.estados.escolhido?;
        if !self.paineis.estados.ocupado(slot) {
            return None;
        }
        let data = self.paineis.estados.data(slot).unwrap_or_default();
        let mut acao = None;
        ui.strong(format!("Slot {slot} — salvo em {data}"));
        ui.horizontal(|ui| {
            if self.paineis.estados.confirmar_apagar {
                ui.label("Apagar este estado? Não dá para desfazer.");
                if ui.button("Sim, apagar").clicked() {
                    acao = Some(Acao::Apaga(slot));
                }
            } else {
                if ui.button("Carregar").clicked() {
                    acao = Some(Acao::Carrega(slot));
                }
                if com_jogo && ui.button("Sobrescrever").clicked() {
                    acao = Some(Acao::Salva(slot));
                }
                if ui.button("Apagar").clicked() {
                    acao = Some(Acao::ConfirmaApagar);
                }
            }
            if ui.button("Cancelar").clicked() {
                acao = Some(Acao::Cancela);
            }
        });
        acao
    }

    fn aplica(&mut self, acao: Acao, alvo: &Alvo) {
        let painel = &mut self.paineis.estados;
        match acao {
            Acao::Escolhe(slot) => {
                painel.escolhido = Some(slot);
                painel.confirmar_apagar = false;
            }
            Acao::ConfirmaApagar => painel.confirmar_apagar = true,
            Acao::Cancela => {
                painel.escolhido = None;
                painel.confirmar_apagar = false;
            }
            Acao::Salva(slot) => {
                painel.escolhido = None;
                if let Some(emu) = self.emulador.as_mut() {
                    emu.salva_no_slot(slot);
                }
            }
            Acao::Apaga(slot) => {
                painel.escolhido = None;
                painel.confirmar_apagar = false;
                self.recado = Some(
                    match emulador::apaga_estado(&alvo.pasta, &alvo.serial, slot) {
                        Ok(()) => format!("slot {slot} apagado"),
                        Err(e) => e,
                    },
                );
            }
            Acao::Carrega(slot) => self.carrega_estado(slot, alvo),
            Acao::Volta => {
                self.recado = None;
                painel.escolhido = None;
                painel.da_biblioteca = None;
                self.volta_do_menu();
            }
        }
    }

    fn carrega_estado(&mut self, slot: u8, alvo: &Alvo) {
        self.paineis.estados.escolhido = None;
        if self.emulador.is_none() {
            if let Some(jogo) = alvo.jogo {
                self.inicia(jogo);
            }
        }
        self.paineis.estados.da_biblioteca = None;
        if let Some(emu) = self.emulador.as_mut() {
            emu.carrega_do_slot(slot);
            self.tela = Tela::Jogando;
        }
    }
}
