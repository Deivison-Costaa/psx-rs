use psx_core::app::input_map::{Alvo, Entrada, Estilo, LINHAS, Perfil, primeira_nova};

use crate::App;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Coluna {
    Teclado,
    Controle,
}

#[derive(Debug, Clone, PartialEq)]
struct Captura {
    alvo: Alvo,
    coluna: Coluna,
    antes: Vec<Entrada>,
    celula: egui::Id,
}

pub(crate) const ATALHOS: [(&str, &str); 10] = [
    ("Esc", "Sair do jogo / voltar nos menus"),
    ("F2", "Trocar disco"),
    ("F5", "Salvar estado no slot"),
    ("F6 / F7", "Slot anterior / próximo"),
    ("F8", "Carregar estado do slot"),
    ("F9", "Cartão de memória"),
    ("F10", "Controles"),
    ("F11", "Ajustes"),
    ("F12", "Velocidade (acelerar)"),
    ("○ / B do controle", "Voltar nos menus (✖ / A confirma)"),
];

const RESERVADAS: [egui::Key; 12] = [
    egui::Key::Escape,
    egui::Key::F1,
    egui::Key::F2,
    egui::Key::F4,
    egui::Key::F5,
    egui::Key::F6,
    egui::Key::F7,
    egui::Key::F8,
    egui::Key::F9,
    egui::Key::F10,
    egui::Key::F11,
    egui::Key::F12,
];

const ALTURA_DA_FAIXA: f32 = 24.0;

fn id_da_captura() -> egui::Id {
    egui::Id::new("psx-rs-captura-de-controle")
}

fn captura(ctx: &egui::Context) -> Option<Captura> {
    ctx.data(|d| d.get_temp::<Captura>(id_da_captura()))
}

fn guarda_captura(ctx: &egui::Context, nova: Option<Captura>) {
    ctx.data_mut(|d| match nova {
        Some(c) => d.insert_temp(id_da_captura(), c),
        None => d.remove::<Captura>(id_da_captura()),
    });
}

/// Devolve o foco a celula: quem navega pelo teclado ou controle continua de onde estava.
fn encerra_captura(ctx: &egui::Context) {
    if let Some(c) = captura(ctx) {
        ctx.memory_mut(|m| m.request_focus(c.celula));
    }
    guarda_captura(ctx, None);
}

pub(crate) fn capturando(ctx: &egui::Context) -> bool {
    captura(ctx).is_some()
}

pub(crate) fn nome_da_tecla(nome: &str) -> String {
    match nome {
        "Space" => "Espaço".into(),
        "Up" => "Seta ⬆".into(),
        "Down" => "Seta ⬇".into(),
        "Left" => "Seta ⬅".into(),
        "Right" => "Seta ➡".into(),
        "Escape" => "Esc".into(),
        outro => outro.into(),
    }
}

fn tecla_apertada(ctx: &egui::Context) -> Option<egui::Key> {
    ctx.input(|i| {
        i.events.iter().find_map(|e| match e {
            egui::Event::Key {
                key,
                pressed: true,
                repeat: false,
                ..
            } => Some(*key),
            _ => None,
        })
    })
}

/// Id fixo por celula: o id automatico muda quando a faixa de captura ganha botoes, e o
/// foco devolvido no fim da captura cairia em outro widget.
fn botao_da_celula(
    ui: &mut egui::Ui,
    alvo: Alvo,
    coluna: Coluna,
    texto: egui::RichText,
    largura: f32,
) -> egui::Response {
    let id = (alvo.chave(), coluna == Coluna::Teclado);
    ui.push_id(id, |ui| {
        ui.add(egui::Button::new(texto).min_size(egui::vec2(largura, 0.0)))
    })
    .inner
}

impl App {
    pub(crate) fn tela_controles(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        self.processa_captura(&ctx);

        ui.heading("Controles");
        let nomes = self.gamepads.nomes();
        if nomes.is_empty() {
            ui.label("Nenhum controle detectado. O teclado continua valendo.");
        } else {
            ui.label(format!("Controle conectado: {}", nomes.join(", ")));
        }
        self.barra_de_perfis(ui);
        ui.push_id("cabecalho-variavel", |ui| {
            self.faixa_da_captura(ui);
            self.aviso_de_conflito(ui);
            self.avisos(ui);
        });
        ui.separator();

        let estilo = self.gamepads.estilo();
        egui::ScrollArea::vertical()
            .id_salt("controles-rolagem")
            .show(ui, |ui| {
                egui::Grid::new("tabela-de-controles")
                    .striped(true)
                    .num_columns(3)
                    .spacing([24.0, 6.0])
                    .show(ui, |ui| {
                        ui.strong("Botão do PlayStation");
                        ui.strong("Teclado");
                        ui.strong("Controle");
                        ui.end_row();
                        for alvo in LINHAS {
                            self.linha(ui, alvo, estilo);
                            ui.end_row();
                        }
                    });
                ui.add_space(12.0);
                ui.strong("Atalhos do emulador (fixos)");
                egui::Grid::new("atalhos").striped(true).show(ui, |ui| {
                    for (tecla, acao) in ATALHOS {
                        ui.monospace(tecla);
                        ui.label(acao);
                        ui.end_row();
                    }
                });
            });
    }

    fn barra_de_perfis(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label("Perfis prontos do controle:");
            if ui.button("Padrão").clicked() {
                self.muda_perfil(self.perfil.controle_de(&Perfil::padrao()));
            }
            if ui.button("Faces trocadas").clicked() {
                self.muda_perfil(self.perfil.controle_de(&Perfil::faces_trocadas()));
            }
            ui.separator();
            if ui.button("Restaurar padrão").clicked() {
                self.muda_perfil(Perfil::padrao());
            }
            if ui.button("Voltar").clicked() {
                guarda_captura(ui.ctx(), None);
                self.volta_do_menu();
            }
        });
    }

    fn aviso_de_conflito(&self, ui: &mut egui::Ui) {
        let teclado = self.perfil.teclado();
        for tecla in teclado.conflitos() {
            let alvos: Vec<String> = teclado.alvos_da(&tecla).iter().map(Alvo::rotulo).collect();
            ui.colored_label(
                egui::Color32::from_rgb(230, 160, 40),
                format!(
                    "Atenção: a tecla {} está em mais de um botão ({}).",
                    nome_da_tecla(&tecla),
                    alvos.join(", ")
                ),
            );
        }
    }

    /// Altura fixa: se a faixa aparecesse so durante a captura, a tabela desceria bem
    /// na hora do clique e o proximo clique cairia na linha errada.
    fn faixa_da_captura(&mut self, ui: &mut egui::Ui) {
        let tamanho = egui::vec2(ui.available_width(), ALTURA_DA_FAIXA);
        let layout = egui::Layout::left_to_right(egui::Align::Center);
        ui.allocate_ui_with_layout(tamanho, layout, |ui| {
            ui.set_min_height(ALTURA_DA_FAIXA);
            match captura(ui.ctx()) {
                Some(c) => self.pedido_da_captura(ui, c),
                None => {
                    ui.label(
                        "Clique numa célula e aperte a tecla ou o botão. As mudanças são gravadas na hora.",
                    );
                }
            }
        });
    }

    fn pedido_da_captura(&mut self, ui: &mut egui::Ui, c: Captura) {
        let pedido = match c.coluna {
            Coluna::Teclado => "Aperte uma tecla",
            Coluna::Controle => "Aperte um botão ou mova um analógico do controle",
        };
        ui.colored_label(
            egui::Color32::LIGHT_BLUE,
            format!("{pedido} para {}… (Esc cancela)", c.alvo.rotulo()),
        );
        if ui.button("Limpar").clicked() {
            let novo = match c.coluna {
                Coluna::Teclado => self.perfil.com_teclado(self.perfil.teclado().limpa(c.alvo)),
                Coluna::Controle => self.perfil.limpa_controle(c.alvo),
            };
            encerra_captura(ui.ctx());
            self.muda_perfil(novo);
        }
        if ui.button("Cancelar").clicked() {
            encerra_captura(ui.ctx());
        }
    }

    fn linha(&mut self, ui: &mut egui::Ui, alvo: Alvo, estilo: Estilo) {
        ui.label(alvo.rotulo());
        let atual = captura(ui.ctx());
        let em = |coluna| {
            atual
                .as_ref()
                .is_some_and(|c| c.alvo == alvo && c.coluna == coluna)
        };

        let teclado = self.perfil.teclado();
        let tecla = teclado.tecla_de(alvo);
        let texto = if em(Coluna::Teclado) {
            egui::RichText::new("Aperte uma tecla…").color(egui::Color32::LIGHT_BLUE)
        } else {
            let t = egui::RichText::new(tecla.map(nome_da_tecla).unwrap_or_else(|| "—".into()));
            if tecla.is_some_and(|t| teclado.alvos_da(t).len() > 1) {
                t.color(egui::Color32::from_rgb(230, 160, 40))
            } else {
                t
            }
        };
        let celula = botao_da_celula(ui, alvo, Coluna::Teclado, texto, 140.0);
        if celula.clicked() {
            self.comeca_captura(ui.ctx(), alvo, Coluna::Teclado, celula.id);
        }

        if matches!(alvo, Alvo::Analogico { .. }) {
            ui.weak(format!("{} (fixo)", alvo.rotulo()));
            return;
        }
        let texto = if em(Coluna::Controle) {
            egui::RichText::new("Aperte um botão…").color(egui::Color32::LIGHT_BLUE)
        } else {
            let entradas: Vec<String> = self
                .perfil
                .entradas_de(alvo)
                .iter()
                .map(|e| e.rotulo(estilo))
                .collect();
            egui::RichText::new(if entradas.is_empty() {
                "—".to_string()
            } else {
                entradas.join(" / ")
            })
        };
        let celula = botao_da_celula(ui, alvo, Coluna::Controle, texto, 200.0);
        if celula.clicked() {
            self.comeca_captura(ui.ctx(), alvo, Coluna::Controle, celula.id);
        }
    }

    fn comeca_captura(
        &mut self,
        ctx: &egui::Context,
        alvo: Alvo,
        coluna: Coluna,
        celula: egui::Id,
    ) {
        let antes = self.gamepads.le().entradas;
        guarda_captura(
            ctx,
            Some(Captura {
                alvo,
                coluna,
                antes,
                celula,
            }),
        );
        ctx.memory_mut(|m| {
            if let Some(id) = m.focused() {
                m.surrender_focus(id);
            }
        });
        self.recado = None;
    }

    fn processa_captura(&mut self, ctx: &egui::Context) {
        let Some(c) = captura(ctx) else {
            return;
        };
        let tecla = tecla_apertada(ctx);
        if tecla == Some(egui::Key::Escape) {
            ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
            encerra_captura(ctx);
            return;
        }
        match c.coluna {
            Coluna::Teclado => self.captura_tecla(ctx, &c, tecla),
            Coluna::Controle => self.captura_controle(ctx, c),
        }
    }

    fn captura_tecla(&mut self, ctx: &egui::Context, c: &Captura, tecla: Option<egui::Key>) {
        let Some(tecla) = tecla else {
            return;
        };
        ctx.input_mut(|i| i.consume_key(i.modifiers, tecla));
        if RESERVADAS.contains(&tecla) {
            self.recado = Some(format!(
                "{} é atalho do emulador; escolha outra tecla.",
                tecla.name()
            ));
            return;
        }
        encerra_captura(ctx);
        let teclado = self.perfil.teclado().associa(c.alvo, tecla.name());
        self.muda_perfil(self.perfil.com_teclado(teclado));
    }

    fn captura_controle(&mut self, ctx: &egui::Context, c: Captura) {
        let agora = self.gamepads.le().entradas;
        match primeira_nova(&c.antes, &agora) {
            Some(entrada) => {
                encerra_captura(ctx);
                self.muda_perfil(self.perfil.associa_controle(c.alvo, entrada));
            }
            None => guarda_captura(ctx, Some(Captura { antes: agora, ..c })),
        }
    }

    fn muda_perfil(&mut self, novo: Perfil) {
        self.perfil = novo;
        self.grava_perfil();
    }
}
