use crate::app::input_map::Entrada;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemDePausa {
    Continuar,
    SalvarEstado,
    CarregarEstado,
    Slot,
    EstadosSalvos,
    MemoryCard,
    TrocarDisco,
    Controles,
    Ajustes,
    Atalhos,
    SairDoJogo,
}

pub const ITENS_DE_PAUSA: [ItemDePausa; 11] = [
    ItemDePausa::Continuar,
    ItemDePausa::SalvarEstado,
    ItemDePausa::CarregarEstado,
    ItemDePausa::Slot,
    ItemDePausa::EstadosSalvos,
    ItemDePausa::MemoryCard,
    ItemDePausa::TrocarDisco,
    ItemDePausa::Controles,
    ItemDePausa::Ajustes,
    ItemDePausa::Atalhos,
    ItemDePausa::SairDoJogo,
];

impl ItemDePausa {
    pub fn rotulo(self) -> &'static str {
        match self {
            ItemDePausa::Continuar => "Continuar",
            ItemDePausa::SalvarEstado => "Salvar estado",
            ItemDePausa::CarregarEstado => "Carregar estado",
            ItemDePausa::Slot => "Slot",
            ItemDePausa::EstadosSalvos => "Estados salvos",
            ItemDePausa::MemoryCard => "Memory card",
            ItemDePausa::TrocarDisco => "Trocar disco",
            ItemDePausa::Controles => "Controles",
            ItemDePausa::Ajustes => "Ajustes",
            ItemDePausa::Atalhos => "Atalhos",
            ItemDePausa::SairDoJogo => "Sair do jogo",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comando {
    Cima,
    Baixo,
    Esquerda,
    Direita,
    Confirma,
    Volta,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Acao {
    Executa(ItemDePausa),
    MudaSlot(i8),
    Continua,
    SaiDoJogo,
}

/// `confirmacao` e `Some(sim?)` enquanto a pergunta de sair esta aberta; o cursor
/// comeca no "nao" para um Enter apressado nao jogar o progresso fora.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MenuDePausa {
    pub selecionado: usize,
    pub confirmacao: Option<bool>,
}

impl MenuDePausa {
    pub fn item(&self) -> ItemDePausa {
        ITENS_DE_PAUSA
            .get(self.selecionado)
            .copied()
            .unwrap_or(ItemDePausa::Continuar)
    }

    pub fn seleciona(&self, indice: usize) -> MenuDePausa {
        MenuDePausa {
            selecionado: indice.min(ITENS_DE_PAUSA.len() - 1),
            confirmacao: self.confirmacao,
        }
    }

    pub fn aplica(&self, comando: Comando) -> (MenuDePausa, Option<Acao>) {
        match self.confirmacao {
            Some(sim) => self.na_pergunta(sim, comando),
            None => self.na_lista(comando),
        }
    }

    fn na_pergunta(&self, sim: bool, comando: Comando) -> (MenuDePausa, Option<Acao>) {
        let fechada = MenuDePausa {
            confirmacao: None,
            ..*self
        };
        match comando {
            Comando::Confirma if sim => (fechada, Some(Acao::SaiDoJogo)),
            Comando::Confirma | Comando::Volta => (fechada, None),
            _ => (
                MenuDePausa {
                    confirmacao: Some(!sim),
                    ..*self
                },
                None,
            ),
        }
    }

    fn na_lista(&self, comando: Comando) -> (MenuDePausa, Option<Acao>) {
        let total = ITENS_DE_PAUSA.len();
        let slot = self.item() == ItemDePausa::Slot;
        match comando {
            Comando::Cima => (self.seleciona((self.selecionado + total - 1) % total), None),
            Comando::Baixo => (self.seleciona((self.selecionado + 1) % total), None),
            Comando::Esquerda if slot => (*self, Some(Acao::MudaSlot(-1))),
            Comando::Direita if slot => (*self, Some(Acao::MudaSlot(1))),
            Comando::Esquerda | Comando::Direita => (*self, None),
            Comando::Volta => (*self, Some(Acao::Continua)),
            Comando::Confirma => match self.item() {
                ItemDePausa::Continuar => (*self, Some(Acao::Continua)),
                ItemDePausa::Slot => (*self, Some(Acao::MudaSlot(1))),
                ItemDePausa::SairDoJogo => (
                    MenuDePausa {
                        confirmacao: Some(false),
                        ..*self
                    },
                    None,
                ),
                outro => (*self, Some(Acao::Executa(outro))),
            },
        }
    }
}

pub fn slot_vizinho(slot: u8, delta: i8, total: u8) -> u8 {
    if total == 0 {
        return 0;
    }
    let n = i16::from(total);
    (i16::from(slot) + i16::from(delta)).rem_euclid(n) as u8
}

fn apertou(antes: &[Entrada], agora: &[Entrada], entrada: Entrada) -> bool {
    agora.contains(&entrada) && !antes.contains(&entrada)
}

/// Start+Select juntos, na borda: segurar os dois nao abre e fecha o menu a cada quadro.
pub fn pede_pausa(antes: &[Entrada], agora: &[Entrada]) -> bool {
    let juntos = |e: &[Entrada]| e.contains(&Entrada::Start) && e.contains(&Entrada::Select);
    juntos(agora) && !juntos(antes)
}

pub fn comando_do_controle(antes: &[Entrada], agora: &[Entrada]) -> Option<Comando> {
    const MAPA: [(Entrada, Comando); 6] = [
        (Entrada::DpadCima, Comando::Cima),
        (Entrada::DpadBaixo, Comando::Baixo),
        (Entrada::DpadEsquerda, Comando::Esquerda),
        (Entrada::DpadDireita, Comando::Direita),
        (Entrada::Sul, Comando::Confirma),
        (Entrada::Leste, Comando::Volta),
    ];
    MAPA.iter()
        .find(|(e, _)| apertou(antes, agora, *e))
        .map(|(_, c)| *c)
}
