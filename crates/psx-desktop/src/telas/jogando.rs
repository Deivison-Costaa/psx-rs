use psx_core::app::sessao::formata_tempo;

use crate::emulador::{self, Emulador};
use crate::{App, Tela};

impl App {
    pub(crate) fn tela_jogando(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        if self.emulador.is_none() {
            self.tela = Tela::Biblioteca;
            return;
        }
        let do_controle = self.gamepads.le();
        let ganho = self.config.ganho();
        let escala = self.config.escala as f32;
        let filtro = if self.config.filtro_linear {
            egui::TextureOptions::LINEAR
        } else {
            egui::TextureOptions::NEAREST
        };
        let Some(emu) = self.emulador.as_mut() else {
            return;
        };
        emu.entrada(ctx, &self.perfil, &do_controle);
        Self::atalhos_de_estado(ctx, emu);
        emu.quadro(ganho);
        self.gamepads.vibra(emu.vibracao());

        match emu.textura() {
            Some(imagem) => {
                let tamanho = tamanho_na_tela(imagem.width(), imagem.height(), escala);
                let handle = ctx.load_texture("framebuffer", imagem.clone(), filtro);
                ui.add(egui::Image::new(&handle).fit_to_exact_size(tamanho));
            }
            None => {
                ui.label("Display desligado");
            }
        }
        ui.horizontal(|ui| {
            if let Some(titulo) = &self.em_execucao {
                ui.small(titulo);
            }
            let porta = if emu.porta_aberta() {
                " (porta aberta)"
            } else {
                ""
            };
            ui.small(format!("disco: {}{porta}", emu.nome_do_disco()));
            ui.small(if emu.modo_analogico() {
                "DualShock analogico"
            } else {
                "DualShock digital"
            });
            if emu.audio_ativo() {
                ui.small(format!("audio {} Hz", emu.audio_hz()));
            } else {
                ui.small("audio desligado (sem dispositivo de saida)");
            }
            let marca = if emu.slot_existe(emu.slot) { "*" } else { "" };
            ui.small(format!("slot {}{marca}", emu.slot));
            if emu.velocidade > 1 {
                ui.small(format!("{}x", emu.velocidade));
            }
            ui.small(formata_tempo(emu.segundos_jogados()));
        });
        ui.small(
            "Esc: sair · F2: trocar disco · F4: estados salvos · F3/Home do controle: Analog · IJKL: analogico · F5/F8: salvar/carregar · F6/F7: slot · F9: cartao · F10: controles · F11: ajustes · F12: velocidade",
        );
        if let Some(aviso) = &emu.aviso {
            ui.small(aviso.clone());
        }

        if ctx.input(|i| i.key_pressed(egui::Key::F12)) {
            emu.troca_velocidade();
        }
        let (f2, f9, f10, f11, esc) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::F2),
                i.key_pressed(egui::Key::F9),
                i.key_pressed(egui::Key::F10),
                i.key_pressed(egui::Key::F11),
                i.key_pressed(egui::Key::Escape),
            )
        });
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
        if f11 {
            self.tela = Tela::Ajustes;
        }
        if esc {
            self.encerra_partida();
            self.tela = Tela::Biblioteca;
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
        if f6 {
            emu.slot = (emu.slot + emulador::SLOTS - 1) % emulador::SLOTS;
        }
        if f7 {
            emu.slot = (emu.slot + 1) % emulador::SLOTS;
        }
        if f5 {
            emu.salva_estado();
        }
        if f8 {
            emu.carrega_estado();
        }
    }
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
