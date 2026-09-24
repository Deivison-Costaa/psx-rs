use std::time::Duration;

use psx_core::app::input_map::{Comando, Entrada, Navegador, Sentido};

use crate::telas::controles;
use crate::{App, Tela};

/// Acima disso sem passar pelos menus (estava jogando), o que estava apertado vira o
/// "antes": quem entra no menu segurando ○ nao volta sozinho.
const ESTADO_VELHO_S: f64 = 0.25;
const SONDAGEM: Duration = Duration::from_millis(33);

#[derive(Debug, Clone, Default)]
struct Memoria {
    navegador: Navegador,
    visto_em: f64,
}

fn id_da_memoria() -> egui::Id {
    egui::Id::new("psx-rs-navegacao-nos-menus")
}

fn tecla(key: egui::Key, pressed: bool) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    }
}

fn aperta(raw: &mut egui::RawInput, key: egui::Key) {
    raw.events.push(tecla(key, true));
    raw.events.push(tecla(key, false));
}

fn seta(sentido: Sentido) -> egui::Key {
    match sentido {
        Sentido::Cima => egui::Key::ArrowUp,
        Sentido::Baixo => egui::Key::ArrowDown,
        Sentido::Esquerda => egui::Key::ArrowLeft,
        Sentido::Direita => egui::Key::ArrowRight,
    }
}

fn tira_esc(raw: &mut egui::RawInput) -> bool {
    let antes = raw.events.len();
    raw.events.retain(|e| {
        !matches!(
            e,
            egui::Event::Key {
                key: egui::Key::Escape,
                pressed: true,
                ..
            }
        )
    });
    raw.events.len() != antes
}

impl App {
    /// Roda antes do egui ler a entrada do quadro: o foco direcional do egui so anda com
    /// eventos de seta, entao o controle vira Tab/setas/Enter aqui.
    pub(crate) fn navega_nos_menus(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        if self.tela == Tela::Jogando {
            return;
        }
        let capturando = controles::capturando(ctx);
        if !capturando && tira_esc(raw) {
            self.volta_do_menu();
            return;
        }
        let agora = self.gamepads.le().entradas;
        let comandos = self.passo_do_navegador(ctx, raw.time.unwrap_or(0.0), &agora);
        if self.gamepads.conectado() || capturando {
            ctx.request_repaint_after(SONDAGEM);
        }
        if capturando {
            return;
        }
        let tem_foco = ctx.memory(|m| m.focused().is_some());
        for comando in comandos {
            match comando {
                Comando::Voltar => self.volta_do_menu(),
                Comando::Ativar => aperta(raw, egui::Key::Enter),
                Comando::Mover(_) if !tem_foco => aperta(raw, egui::Key::Tab),
                Comando::Mover(s) => aperta(raw, seta(s)),
            }
        }
    }

    fn passo_do_navegador(&self, ctx: &egui::Context, t: f64, agora: &[Entrada]) -> Vec<Comando> {
        let memoria = ctx
            .data(|d| d.get_temp::<Memoria>(id_da_memoria()))
            .unwrap_or_default();
        let navegador = if t - memoria.visto_em > ESTADO_VELHO_S {
            memoria.navegador.com_antes(agora)
        } else {
            memoria.navegador
        };
        let (navegador, comandos) = navegador.passo(&self.perfil, agora, t);
        ctx.data_mut(|d| {
            d.insert_temp(
                id_da_memoria(),
                Memoria {
                    navegador,
                    visto_em: t,
                },
            )
        });
        comandos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn esc_apertado_sai_da_entrada_e_e_avisado() {
        let mut raw = egui::RawInput::default();
        raw.events.push(tecla(egui::Key::Escape, true));
        raw.events.push(tecla(egui::Key::Z, true));
        assert!(tira_esc(&mut raw));
        assert_eq!(raw.events, vec![tecla(egui::Key::Z, true)]);
        assert!(!tira_esc(&mut raw), "sem Esc nao ha o que tirar");
    }

    #[test]
    fn comando_do_controle_vira_aperto_e_soltura_no_mesmo_quadro() {
        let mut raw = egui::RawInput::default();
        aperta(&mut raw, seta(Sentido::Baixo));
        assert_eq!(
            raw.events,
            vec![
                tecla(egui::Key::ArrowDown, true),
                tecla(egui::Key::ArrowDown, false)
            ],
            "sem a soltura o egui acharia a tecla presa e o jogo receberia o botao"
        );
    }
}
