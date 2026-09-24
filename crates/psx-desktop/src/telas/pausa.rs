use psx_core::app::pausa::{
    Acao, Comando, ITENS_DE_PAUSA, ItemDePausa, MenuDePausa, comando_do_controle, pede_pausa,
    slot_vizinho,
};

use crate::emulador;
use crate::telas::jogando::painel;
use crate::{App, Tela};

const TECLAS_DO_MENU: [(egui::Key, Comando); 7] = [
    (egui::Key::ArrowUp, Comando::Cima),
    (egui::Key::ArrowDown, Comando::Baixo),
    (egui::Key::ArrowLeft, Comando::Esquerda),
    (egui::Key::ArrowRight, Comando::Direita),
    (egui::Key::Enter, Comando::Confirma),
    (egui::Key::Space, Comando::Confirma),
    (egui::Key::Escape, Comando::Volta),
];
const LARGURA_DO_MENU: f32 = 280.0;
const PERGUNTA_DE_SAIDA: &str = "Sair? O progresso não salvo no cartão será perdido.";

impl App {
    pub(crate) fn abre_pausa(&mut self) {
        self.menu_de_pausa = Some(MenuDePausa::default());
        self.ajuda_na_pausa = false;
        self.tela = Tela::Pausa;
        self.medida = None;
        self.gamepads.vibra(psx_core::dualshock::Rumble::default());
        if let Some(emu) = self.emulador.as_mut() {
            emu.pausa();
        }
    }

    fn continua(&mut self) {
        self.menu_de_pausa = None;
        self.tela = Tela::Jogando;
        self.segura_entrada = true;
        if let Some(emu) = self.emulador.as_mut() {
            emu.retoma();
        }
    }

    pub(crate) fn tela_pausa(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        if self.emulador.is_none() {
            self.tela = Tela::Biblioteca;
            return;
        }
        self.desenha_imagem(ctx, ui, true);
        let comandos = self.comandos_da_pausa(ctx);
        let menu = self.menu_de_pausa.unwrap_or_default();
        if self.ajuda_na_pausa {
            Self::desenha_ajuda(ctx);
            if !comandos.is_empty() || ctx.input(|i| i.pointer.any_click()) {
                self.ajuda_na_pausa = false;
            }
            return;
        }
        let (menu, acao) = comandos
            .iter()
            .fold((menu, None), |(m, acao), c| match acao {
                Some(_) => (m, acao),
                None => m.aplica(*c),
            });
        let (menu, clique) = self.desenha_menu(ctx, menu);
        self.menu_de_pausa = Some(menu);
        if let Some(acao) = acao.or(clique) {
            self.executa(acao);
        }
    }

    /// Todas as teclas do quadro, em ordem, e consumidas: sem isso o botao focado do egui
    /// tambem reage ao Enter e o item roda duas vezes.
    fn comandos_da_pausa(&mut self, ctx: &egui::Context) -> Vec<Comando> {
        let mut comandos: Vec<Comando> = ctx.input_mut(|i| {
            let lidos = i
                .events
                .iter()
                .filter_map(|e| match e {
                    egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } if modifiers.is_none() => TECLAS_DO_MENU
                        .iter()
                        .find(|(tecla, _)| tecla == key)
                        .map(|(_, c)| *c),
                    _ => None,
                })
                .collect();
            for (tecla, _) in TECLAS_DO_MENU {
                i.consume_key(egui::Modifiers::NONE, tecla);
            }
            lidos
        });
        let controle = self.gamepads.le();
        let do_controle = if pede_pausa(&self.controle_antes, &controle.entradas) {
            Some(Comando::Volta)
        } else {
            comando_do_controle(&self.controle_antes, &controle.entradas)
        };
        self.controle_antes = controle.entradas;
        comandos.extend(do_controle);
        comandos
    }

    fn desenha_menu(&self, ctx: &egui::Context, menu: MenuDePausa) -> (MenuDePausa, Option<Acao>) {
        let mut novo = menu;
        let mut acao = None;
        let slot = self.emulador.as_ref().map_or(0, |e| e.slot);
        let slot_salvo = self.emulador.as_ref().is_some_and(|e| e.slot_existe(slot));
        let mexeu = ctx.input(|i| i.pointer.delta() != egui::Vec2::ZERO);
        egui::Area::new(egui::Id::new("pausa"))
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                painel().show(ui, |ui| {
                    ui.set_width(LARGURA_DO_MENU);
                    ui.vertical_centered(|ui| ui.heading("Pausa"));
                    if let Some(titulo) = &self.em_execucao {
                        ui.vertical_centered(|ui| ui.small(titulo));
                    }
                    ui.add_space(6.0);
                    if let Some(sim) = menu.confirmacao {
                        (novo, acao) = Self::pergunta_de_saida(ui, menu, sim);
                        return;
                    }
                    for (i, item) in ITENS_DE_PAUSA.iter().enumerate() {
                        let texto = match item {
                            ItemDePausa::Slot => {
                                let estado = if slot_salvo { "salvo" } else { "vazio" };
                                format!("◀  Slot {slot} ({estado})  ▶")
                            }
                            outro => outro.rotulo().to_string(),
                        };
                        let botao = egui::Button::new(texto)
                            .selected(i == menu.selecionado)
                            .min_size(egui::vec2(LARGURA_DO_MENU, 28.0));
                        let r = ui.add(botao);
                        if r.hovered() && mexeu {
                            novo = novo.seleciona(i);
                        }
                        if r.clicked() {
                            let (m, a) = menu
                                .seleciona(i)
                                .aplica(Self::lado_do_clique(ui, &r, *item));
                            (novo, acao) = (m, a);
                        }
                    }
                });
            });
        (novo, acao)
    }

    /// No slot, clicar na metade esquerda volta um e na direita avança um.
    fn lado_do_clique(ui: &egui::Ui, r: &egui::Response, item: ItemDePausa) -> Comando {
        let x = ui.ctx().input(|i| i.pointer.interact_pos()).map(|p| p.x);
        match (item, x) {
            (ItemDePausa::Slot, Some(x)) if x < r.rect.center().x => Comando::Esquerda,
            (ItemDePausa::Slot, Some(_)) => Comando::Direita,
            _ => Comando::Confirma,
        }
    }

    fn pergunta_de_saida(
        ui: &mut egui::Ui,
        menu: MenuDePausa,
        sim: bool,
    ) -> (MenuDePausa, Option<Acao>) {
        ui.label(PERGUNTA_DE_SAIDA);
        ui.add_space(6.0);
        let mut saida = (menu, None);
        ui.horizontal(|ui| {
            let largura = egui::vec2(LARGURA_DO_MENU / 2.0 - 4.0, 28.0);
            if ui
                .add(
                    egui::Button::new("Sim, sair")
                        .selected(sim)
                        .min_size(largura),
                )
                .clicked()
            {
                saida = MenuDePausa {
                    confirmacao: Some(true),
                    ..menu
                }
                .aplica(Comando::Confirma);
            }
            if ui
                .add(egui::Button::new("Não").selected(!sim).min_size(largura))
                .clicked()
            {
                saida = menu.aplica(Comando::Volta);
            }
        });
        saida
    }

    fn executa(&mut self, acao: Acao) {
        match acao {
            Acao::Continua => self.continua(),
            Acao::SaiDoJogo => {
                self.encerra_partida();
                self.tela = Tela::Biblioteca;
            }
            Acao::MudaSlot(delta) => {
                if let Some(emu) = self.emulador.as_mut() {
                    emu.slot = slot_vizinho(emu.slot, delta, emulador::SLOTS);
                }
            }
            Acao::Executa(item) => self.executa_item(item),
        }
    }

    fn executa_item(&mut self, item: ItemDePausa) {
        match item {
            ItemDePausa::SalvarEstado => {
                if let Some(emu) = self.emulador.as_mut() {
                    emu.salva_estado();
                }
            }
            ItemDePausa::CarregarEstado => {
                let carregou = self.emulador.as_mut().is_some_and(|emu| {
                    let existe = emu.slot_existe(emu.slot);
                    emu.carrega_estado();
                    existe
                });
                if carregou {
                    self.continua();
                }
            }
            ItemDePausa::MemoryCard => self.tela = Tela::Saves,
            ItemDePausa::TrocarDisco => self.abre_troca_de_disco(),
            ItemDePausa::Controles => self.tela = Tela::Controles,
            ItemDePausa::Ajustes => self.tela = Tela::Ajustes,
            ItemDePausa::Atalhos => self.ajuda_na_pausa = true,
            ItemDePausa::Continuar | ItemDePausa::Slot | ItemDePausa::SairDoJogo => {}
        }
    }
}
