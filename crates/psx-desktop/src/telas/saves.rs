use std::path::{Path, PathBuf};

use psx_core::app::saves::{self, Save};

use crate::emulador;
use crate::{App, Tela};

const EXTENSAO: &str = "mcd";
const BLOCOS: u8 = 15;
const LARGURA_DA_LISTA: f32 = 260.0;
const LARGURA_DO_CAMPO: f32 = 380.0;
const ROTULO: f32 = 90.0;

enum Confirma {
    Apagar(u8, String),
    Formatar,
    Importar(Vec<u8>, String),
}

#[derive(Default)]
pub(crate) struct Painel {
    selecionado: Option<PathBuf>,
    confirma: Option<Confirma>,
    exportar_para: String,
    importar_de: String,
}

enum Acao {
    Seleciona(PathBuf),
    Pede(Confirma),
    Confirmado,
    Cancela,
    Exporta,
    Importa,
    Volta,
}

fn cartoes_na_pasta(pasta: &Path) -> Vec<PathBuf> {
    let Ok(entradas) = std::fs::read_dir(pasta) else {
        return Vec::new();
    };
    let mut fora: Vec<PathBuf> = entradas
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .is_some_and(|e| e.eq_ignore_ascii_case(EXTENSAO))
        })
        .collect();
    fora.sort();
    fora
}

fn expande(texto: &str, casa: Option<&Path>) -> PathBuf {
    match (texto.trim().strip_prefix("~/"), casa) {
        (Some(resto), Some(c)) => c.join(resto),
        _ => PathBuf::from(texto.trim()),
    }
}

impl App {
    /// Cartão de um jogo da biblioteca, mesmo que ainda não exista (importar cria).
    pub(crate) fn abre_cartao_do_jogo(&mut self, indice: usize) {
        let Some(jogo) = self.jogos.get(indice) else {
            return;
        };
        let pasta = PathBuf::from(self.efetiva().pasta_de_cartoes);
        self.paineis.cartoes.selecionado = Some(pasta.join(saves::nome_do_cartao(jogo.serial())));
        self.tela = Tela::Saves;
    }

    fn cartao_do_jogo_aberto(&self) -> Option<PathBuf> {
        self.emulador
            .as_ref()
            .map(|e| e.caminho_do_cartao().to_path_buf())
    }

    fn imagem(&self, cartao: &Path) -> Option<Vec<u8>> {
        if self.cartao_do_jogo_aberto().as_deref() == Some(cartao) {
            return self.emulador.as_ref().map(|e| e.imagem_do_cartao());
        }
        std::fs::read(cartao).ok()
    }

    fn grava_cartao(&mut self, cartao: &Path, imagem: &[u8]) -> Result<(), String> {
        if self.cartao_do_jogo_aberto().as_deref() == Some(cartao) {
            if let Some(emu) = self.emulador.as_mut() {
                return emu.troca_imagem_do_cartao(imagem);
            }
        }
        emulador::grava_arquivo(cartao, imagem)
    }

    fn nome_do_cartao(&self, cartao: &Path) -> String {
        let serial = cartao
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        match self.jogos.iter().find(|j| j.serial() == serial) {
            Some(jogo) => format!("{} ({serial})", jogo.titulo),
            None => serial,
        }
    }

    fn lista_de_cartoes(&self) -> Vec<PathBuf> {
        let mut todos = cartoes_na_pasta(Path::new(&self.efetiva().pasta_de_cartoes));
        for extra in [
            self.cartao_do_jogo_aberto(),
            self.paineis.cartoes.selecionado.clone(),
        ]
        .into_iter()
        .flatten()
        {
            if !todos.contains(&extra) {
                todos.push(extra);
            }
        }
        todos
    }

    pub(crate) fn tela_saves(&mut self, ui: &mut egui::Ui) {
        if self.paineis.cartoes.selecionado.is_none() {
            self.paineis.cartoes.selecionado = self.cartao_do_jogo_aberto();
        }
        if self.paineis.cartoes.exportar_para.is_empty() {
            self.paineis.cartoes.exportar_para = self
                .pastas
                .casa()
                .map(|c| c.to_string_lossy().to_string())
                .unwrap_or_default();
        }
        ui.heading("Memory cards");
        ui.small(format!(
            "Um cartão por jogo, em {}",
            self.efetiva().pasta_de_cartoes
        ));
        self.avisos(ui);
        ui.separator();

        let mut acao = None;
        let cartoes = self.lista_de_cartoes();
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_width(LARGURA_DA_LISTA);
                egui::ScrollArea::vertical()
                    .id_salt("lista-de-cartoes")
                    .show(ui, |ui| {
                        if cartoes.is_empty() {
                            ui.label("Nenhum cartão ainda: ele nasce quando um jogo grava.");
                        }
                        for cartao in &cartoes {
                            let marcado = self.paineis.cartoes.selecionado.as_ref() == Some(cartao);
                            let botao =
                                egui::Button::selectable(marcado, self.nome_do_cartao(cartao))
                                    .min_size(egui::vec2(LARGURA_DA_LISTA - 12.0, 0.0));
                            if ui.add(botao).clicked() {
                                acao = Some(Acao::Seleciona(cartao.clone()));
                            }
                        }
                    });
                ui.add_space(8.0);
                if ui.button("Voltar").clicked() {
                    acao = Some(Acao::Volta);
                }
            });
            ui.separator();
            ui.vertical(|ui| {
                if let Some(a) = self.detalhe_do_cartao(ui) {
                    acao = Some(a);
                }
            });
        });
        if ui.ctx().input(|i| i.key_pressed(egui::Key::Escape)) {
            acao = Some(Acao::Volta);
        }
        if let Some(a) = acao {
            self.aplica_no_cartao(a);
        }
    }

    fn detalhe_do_cartao(&mut self, ui: &mut egui::Ui) -> Option<Acao> {
        let Some(cartao) = self.paineis.cartoes.selecionado.clone() else {
            ui.label("Escolha um cartão na lista.");
            return None;
        };
        let existe = self.cartao_do_jogo_aberto().as_ref() == Some(&cartao) || cartao.exists();
        let imagem = self.imagem(&cartao);
        let valido = imagem.as_deref().is_some_and(saves::e_cartao);
        let lista: Vec<Save> = imagem.as_deref().map(saves::lista).unwrap_or_default();
        let livres = imagem.as_deref().map_or(BLOCOS, saves::blocos_livres);
        let mut acao = None;

        ui.strong(self.nome_do_cartao(&cartao));
        ui.small(cartao.display().to_string());
        if !existe {
            ui.label(
                "Este cartão ainda não existe: fica vazio até o jogo gravar ou você importar um.",
            );
        } else if !valido {
            ui.colored_label(
                egui::Color32::from_rgb(220, 160, 60),
                "O arquivo não é um memory card válido (sem a marca 'MC'). Formate ou importe outro.",
            );
        } else {
            ui.label(format!(
                "{} save(s), {} de {BLOCOS} blocos usados",
                lista.len(),
                BLOCOS - livres
            ));
        }
        ui.separator();
        egui::ScrollArea::vertical()
            .id_salt("saves-do-cartao")
            .max_height(260.0)
            .show(ui, |ui| {
                for save in &lista {
                    ui.horizontal(|ui| {
                        if ui.button("Apagar").clicked() {
                            acao = Some(Acao::Pede(Confirma::Apagar(save.bloco, rotulo(save))));
                        }
                        ui.vertical(|ui| {
                            ui.strong(rotulo(save));
                            ui.small(format!(
                                "{} · bloco {} · {} bloco(s)",
                                save.nome, save.bloco, save.blocos
                            ));
                        });
                    });
                    ui.separator();
                }
            });

        if let Some(a) = self.confirmacao(ui) {
            return Some(a);
        }
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.add_sized([ROTULO, 20.0], egui::Label::new("Exportar para"));
            ui.add(
                egui::TextEdit::singleline(&mut self.paineis.cartoes.exportar_para)
                    .desired_width(LARGURA_DO_CAMPO),
            );
            if ui
                .add_enabled(existe, egui::Button::new("Exportar .mcd"))
                .clicked()
            {
                acao = Some(Acao::Exporta);
            }
        });
        ui.horizontal(|ui| {
            ui.add_sized([ROTULO, 20.0], egui::Label::new("Importar de"));
            ui.add(
                egui::TextEdit::singleline(&mut self.paineis.cartoes.importar_de)
                    .hint_text("~/Downloads/cartao.mcd")
                    .desired_width(LARGURA_DO_CAMPO),
            )
            .on_hover_text(".mcd, .mcr (128 KiB crus), .gme (DexDrive) ou .mem (VGS)");
            if ui.button("Importar…").clicked() {
                acao = Some(Acao::Importa);
            }
        });
        if ui.button("Formatar cartão").clicked() {
            acao = Some(Acao::Pede(Confirma::Formatar));
        }
        acao
    }

    fn confirmacao(&mut self, ui: &mut egui::Ui) -> Option<Acao> {
        let pergunta = match self.paineis.cartoes.confirma.as_ref()? {
            Confirma::Apagar(_, nome) => format!("Apagar o save \"{nome}\" deste cartão?"),
            Confirma::Formatar => "Formatar o cartão? Todos os saves dele somem.".to_string(),
            Confirma::Importar(_, de) => {
                format!("Substituir TODO o conteúdo deste cartão pelo de {de}?")
            }
        };
        let mut acao = None;
        ui.colored_label(egui::Color32::from_rgb(220, 160, 60), pergunta);
        ui.horizontal(|ui| {
            if ui.button("Confirmar").clicked() {
                acao = Some(Acao::Confirmado);
            }
            if ui.button("Cancelar").clicked() {
                acao = Some(Acao::Cancela);
            }
        });
        acao
    }

    fn aplica_no_cartao(&mut self, acao: Acao) {
        match acao {
            Acao::Seleciona(c) => {
                self.paineis.cartoes.selecionado = Some(c);
                self.paineis.cartoes.confirma = None;
            }
            Acao::Pede(c) => self.paineis.cartoes.confirma = Some(c),
            Acao::Cancela => self.paineis.cartoes.confirma = None,
            Acao::Confirmado => self.executa_confirmado(),
            Acao::Exporta => self.exporta(),
            Acao::Importa => self.prepara_importacao(),
            Acao::Volta => {
                self.recado = None;
                self.erro = None;
                self.paineis.cartoes.selecionado = None;
                self.paineis.cartoes.confirma = None;
                self.volta_do_menu();
            }
        }
    }

    fn executa_confirmado(&mut self) {
        let (Some(confirma), Some(cartao)) = (
            self.paineis.cartoes.confirma.take(),
            self.paineis.cartoes.selecionado.clone(),
        ) else {
            return;
        };
        let atual = self.imagem(&cartao).unwrap_or_else(saves::formatado);
        let (nova, feito) = match confirma {
            Confirma::Apagar(bloco, nome) => match saves::apaga(&atual, bloco) {
                Ok(n) => (n, format!("save \"{nome}\" apagado")),
                Err(e) => {
                    self.erro = Some(e.to_string());
                    return;
                }
            },
            Confirma::Formatar => (saves::formatado(), "cartão formatado".to_string()),
            Confirma::Importar(bytes, de) => (bytes, format!("cartão importado de {de}")),
        };
        match self.grava_cartao(&cartao, &nova) {
            Ok(()) => {
                self.erro = None;
                self.recado = Some(feito);
            }
            Err(e) => self.erro = Some(e),
        }
    }

    fn exporta(&mut self) {
        let Some(cartao) = self.paineis.cartoes.selecionado.clone() else {
            return;
        };
        let destino = expande(&self.paineis.cartoes.exportar_para, self.pastas.casa());
        let arquivo = if destino.is_dir() {
            destino.join(cartao.file_name().unwrap_or_default())
        } else {
            destino
        };
        let resultado = self
            .imagem(&cartao)
            .ok_or_else(|| "o cartão não pôde ser lido".to_string())
            .and_then(|img| emulador::grava_arquivo(&arquivo, &img));
        match resultado {
            Ok(()) => self.recado = Some(format!("exportado para {}", arquivo.display())),
            Err(e) => self.erro = Some(e),
        }
    }

    fn prepara_importacao(&mut self) {
        let origem = expande(&self.paineis.cartoes.importar_de, self.pastas.casa());
        let validado = std::fs::read(&origem)
            .map_err(|e| format!("lendo '{}': {e}", origem.display()))
            .and_then(|b| saves::importa(&b).map_err(|e| e.to_string()));
        match validado {
            Ok(img) => {
                self.erro = None;
                self.paineis.cartoes.confirma = Some(Confirma::Importar(img, nome_de(&origem)));
            }
            Err(e) => self.erro = Some(e),
        }
    }
}

fn rotulo(save: &Save) -> String {
    if save.titulo.is_empty() {
        "(sem título)".to_string()
    } else {
        save.titulo.clone()
    }
}

fn nome_de(caminho: &Path) -> String {
    caminho
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| caminho.display().to_string())
}
