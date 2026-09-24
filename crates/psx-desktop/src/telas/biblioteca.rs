use std::collections::HashMap;

use psx_core::app::library::{self, Ordem, agrupa, casa_busca, ordena, rotulo_do_disco};
use psx_core::app::sessao::formata_tempo;

use crate::{App, Tela};

#[derive(Default)]
pub(crate) struct Painel {
    busca: String,
    ordem: Ordem,
    selecionado: usize,
    disco: HashMap<String, usize>,
    rolar: bool,
}

/// Um jogo na tela: um ou mais discos (`membros` são índices em `App::jogos`).
struct Entrada {
    titulo: String,
    membros: Vec<usize>,
    segundos: u64,
    ultima_vez: Option<u64>,
}

enum Acao {
    Joga(usize),
    Estados(usize),
    Cartao(usize),
    Seleciona(usize),
    Disco(String, usize),
}

impl Entrada {
    fn disco(&self, painel: &Painel) -> usize {
        let pos = painel.disco.get(&self.titulo).copied().unwrap_or(0);
        self.membros
            .get(pos)
            .or(self.membros.first())
            .copied()
            .unwrap_or(0)
    }
}

impl App {
    fn entradas(&self) -> Vec<Entrada> {
        let nomes: Vec<String> = self.jogos.iter().map(|j| j.titulo.clone()).collect();
        let recentes = self.recentes();
        let todas: Vec<Entrada> = agrupa(&nomes)
            .into_iter()
            .filter(|g| {
                let mut campos: Vec<&str> = vec![g.titulo.as_str()];
                campos.extend(g.membros.iter().map(|i| self.jogos[*i].serial()));
                casa_busca(&self.paineis.biblioteca.busca, &campos)
            })
            .map(|g| {
                let seriais = g.membros.iter().map(|i| self.jogos[*i].serial());
                let segundos = seriais.clone().map(|s| recentes.tempo_de(s)).sum();
                let ultima_vez = seriais.filter_map(|s| recentes.ultima_vez_de(s)).max();
                Entrada {
                    titulo: g.titulo,
                    membros: g.membros,
                    segundos,
                    ultima_vez,
                }
            })
            .collect();
        let chaves: Vec<(String, Option<u64>)> = todas
            .iter()
            .map(|e| (e.titulo.clone(), e.ultima_vez))
            .collect();
        let ordem = ordena(&chaves, self.paineis.biblioteca.ordem);
        let mut por_indice: Vec<Option<Entrada>> = todas.into_iter().map(Some).collect();
        ordem
            .into_iter()
            .filter_map(|i| por_indice.get_mut(i).and_then(Option::take))
            .collect()
    }

    fn continuar(&self) -> Option<(usize, String)> {
        let topo = self.recentes().itens().first()?;
        let indice = self.jogos.iter().position(|j| j.serial() == topo.serial)?;
        let titulo = library::agrupa(std::slice::from_ref(&self.jogos[indice].titulo))
            .first()
            .map(|g| g.titulo.clone())
            .unwrap_or_else(|| topo.titulo.clone());
        Some((
            indice,
            format!(
                "▶  Continuar: {titulo} · jogado {}",
                formata_tempo(topo.segundos)
            ),
        ))
    }

    fn teclado_da_biblioteca(&mut self, ui: &egui::Ui, entradas: &[Entrada]) -> Option<Acao> {
        let (cima, baixo, enter, esq, dir, esc) = ui.ctx().input_mut(|i| {
            (
                i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp),
                i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown),
                i.consume_key(egui::Modifiers::NONE, egui::Key::Enter),
                self.paineis.biblioteca.busca.is_empty()
                    && i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowLeft),
                self.paineis.biblioteca.busca.is_empty()
                    && i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowRight),
                i.key_pressed(egui::Key::Escape),
            )
        });
        let painel = &mut self.paineis.biblioteca;
        if esc {
            painel.busca.clear();
        }
        let ultimo = entradas.len().saturating_sub(1);
        if cima {
            painel.selecionado = painel.selecionado.saturating_sub(1);
            painel.rolar = true;
        }
        if baixo {
            painel.selecionado = (painel.selecionado + 1).min(ultimo);
            painel.rolar = true;
        }
        let atual = entradas.get(painel.selecionado)?;
        if esq || dir {
            let pos = painel.disco.get(&atual.titulo).copied().unwrap_or(0);
            let nova = if dir {
                (pos + 1).min(atual.membros.len() - 1)
            } else {
                pos.saturating_sub(1)
            };
            return Some(Acao::Disco(atual.titulo.clone(), nova));
        }
        enter.then(|| Acao::Joga(atual.disco(painel)))
    }

    pub(crate) fn tela_biblioteca(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Biblioteca");
            ui.add_space(12.0);
            if ui.button("Atualizar").clicked() {
                self.revarre();
            }
            if ui.button("Memory cards").clicked() {
                self.tela = Tela::Saves;
            }
            if ui.button("Ajustes").clicked() {
                self.tela = Tela::Ajustes;
            }
            if ui.button("Controles").clicked() {
                self.tela = Tela::Controles;
            }
        });
        self.avisos(ui);
        if self.jogos.is_empty() {
            ui.separator();
            ui.label(format!(
                "Nenhum .cue encontrado em {}. Aponte a pasta de jogos em Ajustes.",
                self.efetiva().pasta_de_jogos
            ));
            return;
        }

        let mut acao = None;
        if let Some((indice, texto)) = self.continuar() {
            let botao = egui::Button::new(egui::RichText::new(texto).size(16.0).strong())
                .min_size(egui::vec2(ui.available_width(), 34.0));
            if ui.add(botao).clicked() {
                acao = Some(Acao::Joga(indice));
            }
        }
        let entradas = self.entradas();
        let painel = &mut self.paineis.biblioteca;
        painel.selecionado = painel.selecionado.min(entradas.len().saturating_sub(1));
        ui.horizontal(|ui| {
            ui.label("Buscar");
            let busca = ui.add(
                egui::TextEdit::singleline(&mut painel.busca)
                    .hint_text("nome ou serial")
                    .desired_width(260.0),
            );
            if busca.changed() {
                painel.selecionado = 0;
            }
            if ui.ctx().memory(|m| m.focused().is_none()) {
                busca.request_focus();
            }
            ui.label("Ordem");
            ui.selectable_value(&mut painel.ordem, Ordem::Nome, "Nome");
            ui.selectable_value(&mut painel.ordem, Ordem::Recentes, "Recentes");
            ui.label(format!(
                "{} jogo(s) na lista · {} disco(s) na pasta",
                entradas.len(),
                self.jogos.len()
            ));
        });
        ui.small(
            "Setas para cima/baixo: escolher · Enter ou duplo clique: jogar · setas laterais: disco · Esc: limpar a busca",
        );
        ui.separator();
        if let Some(a) = self.teclado_da_biblioteca(ui, &entradas) {
            acao = Some(a);
        }
        if entradas.is_empty() {
            ui.label("Nenhum jogo casa com a busca.");
        }
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (pos, entrada) in entradas.iter().enumerate() {
                if let Some(a) = self.linha_do_jogo(ui, pos, entrada) {
                    acao = Some(a);
                }
            }
        });
        self.paineis.biblioteca.rolar = false;
        match acao {
            Some(Acao::Joga(i)) => self.inicia(i),
            Some(Acao::Estados(i)) => self.abre_estados_do_jogo(i),
            Some(Acao::Cartao(i)) => self.abre_cartao_do_jogo(i),
            Some(Acao::Seleciona(pos)) => self.paineis.biblioteca.selecionado = pos,
            Some(Acao::Disco(titulo, d)) => {
                self.paineis.biblioteca.disco.insert(titulo, d);
            }
            None => {}
        }
    }

    fn linha_do_jogo(&self, ui: &mut egui::Ui, pos: usize, entrada: &Entrada) -> Option<Acao> {
        let painel = &self.paineis.biblioteca;
        let marcado = painel.selecionado == pos;
        let jogo_idx = entrada.disco(painel);
        let jogo = &self.jogos[jogo_idx];
        let mut acao = None;
        let texto = egui::RichText::new(&entrada.titulo).size(15.0).strong();
        let largura = ui.available_width();
        let resp =
            ui.add(egui::Button::selectable(marcado, texto).min_size(egui::vec2(largura, 26.0)));
        if marcado && painel.rolar {
            resp.scroll_to_me(Some(egui::Align::Center));
        }
        if resp.double_clicked() {
            acao = Some(Acao::Joga(jogo_idx));
        } else if resp.clicked() {
            acao = Some(Acao::Seleciona(pos));
        }
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.small(jogo.detalhe(entrada.segundos));
            if entrada.membros.len() > 1 {
                for (n, membro) in entrada.membros.iter().enumerate() {
                    let nome = rotulo_do_disco(&self.jogos[*membro].titulo)
                        .unwrap_or_else(|| format!("Disco {}", n + 1));
                    if ui.selectable_label(*membro == jogo_idx, nome).clicked() {
                        acao = Some(Acao::Disco(entrada.titulo.clone(), n));
                    }
                }
            }
            if marcado {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Memory card").clicked() {
                        acao = Some(Acao::Cartao(jogo_idx));
                    }
                    if ui.button("Estados salvos").clicked() {
                        acao = Some(Acao::Estados(jogo_idx));
                    }
                    if ui.button("Jogar").clicked() {
                        acao = Some(Acao::Joga(jogo_idx));
                    }
                });
            }
        });
        ui.add_space(4.0);
        acao
    }
}
