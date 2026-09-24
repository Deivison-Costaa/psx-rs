use psx_core::app::exibicao::{Sobreposicao, retangulo_da_imagem};
use psx_core::app::pausa::pede_pausa;
use psx_core::app::sessao::formata_tempo;

use crate::emulador::{self, Emulador};
use crate::gamepad::Leitura;
use crate::{App, Tela};

const ESCURECIDO: egui::Color32 = egui::Color32::from_black_alpha(170);
const FUNDO_DE_PAINEL: egui::Color32 = egui::Color32::from_rgba_premultiplied(22, 22, 26, 235);
const BORDA_DE_PAINEL: egui::Color32 = egui::Color32::from_gray(70);
const MARGEM_DE_SOBREPOSICAO: f32 = 16.0;
const TEXTO_DE_SOBREPOSICAO: f32 = 15.0;

pub(crate) const ATALHOS: [(&str, &str); 16] = [
    ("Esc / Start+Select", "menu de pausa"),
    ("F1", "status; de novo, esta ajuda"),
    ("F11 / Alt+Enter", "tela cheia"),
    ("F2", "trocar disco"),
    ("F3 / Home do controle", "botão Analog"),
    ("F5 / F8", "salvar / carregar estado"),
    ("F6 / F7", "slot anterior / próximo"),
    ("F9", "memory card"),
    ("F10", "controles"),
    ("F12", "velocidade (1x, 2x, 4x, 8x)"),
    ("Setas", "direcional"),
    ("Z / Espaço / A / S", "X / Bola / Quadrado / Triângulo"),
    ("Enter / Tab", "Start / Select"),
    ("D / F", "L1 / R1"),
    ("E / R", "L2 / R2"),
    ("I J K L", "analógico esquerdo"),
];

impl App {
    pub(crate) fn tela_jogando(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        if self.emulador.is_none() {
            self.tela = Tela::Biblioteca;
            return;
        }
        let controle = self.gamepads.le();
        let esc = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        let pausa = esc || pede_pausa(&self.controle_antes, &controle.entradas);
        self.controle_antes = controle.entradas.clone();
        if pausa {
            self.abre_pausa();
            self.desenha_imagem(ctx, ui, true);
            return;
        }
        self.roda_quadro(ctx, &controle);
        self.desenha_imagem(ctx, ui, false);
        match self.sobreposicao {
            Sobreposicao::Nenhuma => {}
            Sobreposicao::Status => self.desenha_status(ctx),
            Sobreposicao::Ajuda => Self::desenha_ajuda(ctx),
        }
        self.atalhos_de_tela(ctx);
    }

    fn roda_quadro(&mut self, ctx: &egui::Context, controle: &Leitura) {
        let ganho = self.config.ganho();
        let solto = ctx.input(|i| i.keys_down.is_empty()) && controle.entradas.is_empty();
        if self.segura_entrada && solto {
            self.segura_entrada = false;
        }
        let Some(emu) = self.emulador.as_mut() else {
            return;
        };
        if self.segura_entrada {
            emu.solta_tudo();
        } else {
            emu.entrada(ctx, &self.perfil, controle);
        }
        Self::atalhos_de_estado(ctx, emu);
        emu.quadro(ganho);
        self.gamepads.vibra(emu.vibracao());

        let agora = std::time::Instant::now();
        let ciclos = emu.ciclos();
        if let Some((antes, ciclos_antes)) = self.medida {
            self.medidor = self.medidor.acumula(
                ciclos.saturating_sub(ciclos_antes),
                (agora - antes).as_secs_f64(),
                emu.ciclos_por_quadro(),
            );
        }
        self.medida = Some((agora, ciclos));
    }

    pub(crate) fn desenha_imagem(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        escurece: bool,
    ) {
        let area = ui.max_rect();
        let filtro = if self.config.filtro_linear {
            egui::TextureOptions::LINEAR
        } else {
            egui::TextureOptions::NEAREST
        };
        let Some(imagem) = self.emulador.as_ref().and_then(Emulador::textura) else {
            ui.painter().text(
                area.center(),
                egui::Align2::CENTER_CENTER,
                "Display desligado",
                egui::FontId::proportional(16.0),
                egui::Color32::GRAY,
            );
            return;
        };
        let base = tamanho_na_tela(imagem.width(), imagem.height(), 1.0);
        let ppp = ctx.pixels_per_point();
        let r = retangulo_da_imagem(
            (area.width() * ppp, area.height() * ppp),
            (base.x, base.y),
            self.config.modo_de_imagem,
        );
        let destino = egui::Rect::from_min_size(
            area.min + egui::vec2(r.x, r.y) / ppp,
            egui::vec2(r.largura, r.altura) / ppp,
        );
        let handle = match self.textura.as_mut() {
            Some(h) => {
                h.set(imagem.clone(), filtro);
                h.id()
            }
            None => {
                let h = ctx.load_texture("framebuffer", imagem.clone(), filtro);
                let id = h.id();
                self.textura = Some(h);
                id
            }
        };
        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        ui.painter()
            .image(handle, destino, uv, egui::Color32::WHITE);
        if escurece {
            ui.painter().rect_filled(area, 0.0, ESCURECIDO);
        }
    }

    fn atalhos_de_tela(&mut self, ctx: &egui::Context) {
        let (f1, f2, f9, f10, f12) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::F1),
                i.key_pressed(egui::Key::F2),
                i.key_pressed(egui::Key::F9),
                i.key_pressed(egui::Key::F10),
                i.key_pressed(egui::Key::F12),
            )
        });
        if f1 {
            self.sobreposicao = self.sobreposicao.proxima();
        }
        if f12 {
            if let Some(emu) = self.emulador.as_mut() {
                emu.troca_velocidade();
                emu.aviso = Some(format!("velocidade {}x", emu.velocidade));
            }
        }
        if f9 {
            self.tela = Tela::Saves;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F4)) {
            self.tela = Tela::Estados;
        }
        if f2 {
            self.gamepads.vibra(psx_core::dualshock::Rumble::default());
            self.abre_troca_de_disco();
        }
        if f10 {
            self.tela = Tela::Controles;
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
        if f6 || f7 {
            let delta = if f6 { -1 } else { 1 };
            emu.slot = psx_core::app::pausa::slot_vizinho(emu.slot, delta, emulador::SLOTS);
            let marca = if emu.slot_existe(emu.slot) {
                "salvo"
            } else {
                "vazio"
            };
            emu.aviso = Some(format!("slot {} ({marca})", emu.slot));
        }
        if f5 {
            emu.salva_estado();
        }
        if f8 {
            emu.carrega_estado();
        }
    }

    fn linhas_de_status(&self, emu: &Emulador) -> Vec<String> {
        let porta = if emu.porta_aberta() {
            " (porta aberta)"
        } else {
            ""
        };
        let marca = if emu.slot_existe(emu.slot) {
            "salvo"
        } else {
            "vazio"
        };
        let fps = self
            .medidor
            .fps()
            .map_or("medindo…".to_string(), |f| format!("{f:.1}"));
        let audio = if emu.audio_ativo() {
            format!("{} Hz", emu.audio_hz())
        } else {
            "desligado (sem dispositivo de saída)".to_string()
        };
        vec![
            format!("Jogo: {}", self.em_execucao.as_deref().unwrap_or("?")),
            format!("Disco: {}{porta}", emu.nome_do_disco()),
            format!(
                "Controle: DualShock {}",
                if emu.modo_analogico() {
                    "analógico"
                } else {
                    "digital"
                }
            ),
            format!("Slot: {} ({marca})", emu.slot),
            format!("Velocidade: {}x", emu.velocidade),
            format!("FPS emulado: {fps}"),
            format!("Tempo jogado: {}", formata_tempo(emu.segundos_jogados())),
            format!("Áudio: {audio}"),
        ]
    }

    fn desenha_status(&self, ctx: &egui::Context) {
        let Some(emu) = self.emulador.as_ref() else {
            return;
        };
        let linhas = self.linhas_de_status(emu);
        egui::Area::new(egui::Id::new("status"))
            .anchor(
                egui::Align2::LEFT_TOP,
                egui::vec2(MARGEM_DE_SOBREPOSICAO, MARGEM_DE_SOBREPOSICAO),
            )
            .interactable(false)
            .show(ctx, |ui| {
                painel().show(ui, |ui| {
                    for linha in linhas {
                        ui.label(
                            egui::RichText::new(linha)
                                .size(TEXTO_DE_SOBREPOSICAO)
                                .color(egui::Color32::WHITE),
                        );
                    }
                    ui.small("F1: atalhos");
                });
            });
    }

    pub(crate) fn desenha_ajuda(ctx: &egui::Context) {
        egui::Area::new(egui::Id::new("ajuda"))
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .interactable(false)
            .show(ctx, |ui| {
                painel().show(ui, |ui| {
                    ui.heading("Atalhos");
                    egui::Grid::new("atalhos").num_columns(2).show(ui, |ui| {
                        for (tecla, faz) in ATALHOS {
                            ui.label(
                                egui::RichText::new(tecla)
                                    .strong()
                                    .color(egui::Color32::WHITE),
                            );
                            ui.label(faz);
                            ui.end_row();
                        }
                    });
                });
            });
    }

    pub(crate) fn desenha_toast(&self, ctx: &egui::Context) {
        let Some(toast) = &self.toast else {
            return;
        };
        let opacidade = toast.opacidade(ctx.input(|i| i.time));
        egui::Area::new(egui::Id::new("toast"))
            .anchor(
                egui::Align2::RIGHT_TOP,
                egui::vec2(-MARGEM_DE_SOBREPOSICAO, MARGEM_DE_SOBREPOSICAO),
            )
            .order(egui::Order::Tooltip)
            .interactable(false)
            .show(ctx, |ui| {
                ui.multiply_opacity(opacidade);
                painel().show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(&toast.texto)
                            .size(TEXTO_DE_SOBREPOSICAO)
                            .color(egui::Color32::WHITE),
                    );
                });
            });
    }
}

pub(crate) fn painel() -> egui::Frame {
    egui::Frame::NONE
        .fill(FUNDO_DE_PAINEL)
        .stroke(egui::Stroke::new(1.0_f32, BORDA_DE_PAINEL))
        .corner_radius(6.0)
        .inner_margin(egui::Margin::symmetric(14, 10))
}

const LINHAS_PROGRESSIVO_MAX: usize = 320;

fn tamanho_na_tela(largura: usize, altura: usize, escala: f32) -> egui::Vec2 {
    let linhas = if altura > LINHAS_PROGRESSIVO_MAX {
        altura / 2
    } else {
        altura
    };
    let alto = linhas as f32 * escala;
    if largura == 0 {
        return egui::Vec2::ZERO;
    }
    egui::vec2(alto * 4.0 / 3.0, alto)
}

#[cfg(test)]
mod tests {
    use super::tamanho_na_tela;

    #[test]
    fn modos_diferentes_ocupam_a_mesma_area_4_por_3() {
        let base = tamanho_na_tela(320, 224, 3.0);
        assert_eq!(base, egui::vec2(896.0, 672.0), "320x224 x3 em 4:3");
        for (w, h) in [(640, 448), (512, 224), (365, 224), (352, 448), (256, 224)] {
            assert_eq!(
                tamanho_na_tela(w, h, 3.0),
                base,
                "{w}x{h} deve ocupar a mesma area que 320x224"
            );
        }
    }

    #[test]
    fn pal_288_linhas_fica_mais_alto() {
        assert_eq!(tamanho_na_tela(320, 288, 2.0).y, 576.0, "288 linhas x2");
        assert_eq!(tamanho_na_tela(640, 576, 2.0).y, 576.0, "576i vale 288 x2");
    }
}
