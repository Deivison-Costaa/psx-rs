use serde::{Deserialize, Serialize};

use super::alvo::{Alvo, Sentido};
use super::{Eixos, SOLTO, direcao};

/// Teclas guardadas pelo nome do `egui::Key` ("Z", "Space", "Up"): o `psx-core` nao
/// conhece egui (R3), e nome e o que o usuario le e edita no arquivo.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Teclado {
    ligacoes: Vec<(Alvo, String)>,
}

const PADRAO: [(&str, &str); 19] = [
    ("up", "Up"),
    ("down", "Down"),
    ("left", "Left"),
    ("right", "Right"),
    ("cross", "Z"),
    ("circle", "Space"),
    ("square", "A"),
    ("triangle", "S"),
    ("start", "Enter"),
    ("select", "Tab"),
    ("l1", "D"),
    ("r1", "F"),
    ("l2", "E"),
    ("r2", "R"),
    ("analogico-esquerdo-esquerda", "J"),
    ("analogico-esquerdo-direita", "L"),
    ("analogico-esquerdo-cima", "I"),
    ("analogico-esquerdo-baixo", "K"),
    ("analog", "F3"),
];

impl Teclado {
    pub fn vazio() -> Self {
        Teclado::default()
    }

    pub fn padrao() -> Self {
        PADRAO
            .iter()
            .filter_map(|(alvo, tecla)| Alvo::de_chave(alvo).map(|a| (a, *tecla)))
            .fold(Teclado::vazio(), |t, (alvo, tecla)| t.associa(alvo, tecla))
    }

    pub fn ligacoes(&self) -> &[(Alvo, String)] {
        &self.ligacoes
    }

    pub fn tecla_de(&self, alvo: Alvo) -> Option<&str> {
        self.ligacoes
            .iter()
            .find(|(a, _)| *a == alvo)
            .map(|(_, t)| t.as_str())
    }

    pub fn alvos_da(&self, tecla: &str) -> Vec<Alvo> {
        self.ligacoes
            .iter()
            .filter(|(_, t)| t == tecla)
            .map(|(a, _)| *a)
            .collect()
    }

    /// Uma tecla por alvo; a mesma tecla em dois alvos e permitida (aciona os dois) e
    /// aparece em `conflitos` para a tela avisar.
    pub fn associa(&self, alvo: Alvo, tecla: &str) -> Teclado {
        let mut ligacoes = self.limpa(alvo).ligacoes;
        ligacoes.push((alvo, tecla.to_string()));
        ligacoes.sort_by_key(|(a, _)| a.ordem());
        Teclado { ligacoes }
    }

    pub fn limpa(&self, alvo: Alvo) -> Teclado {
        Teclado {
            ligacoes: self
                .ligacoes
                .iter()
                .filter(|(a, _)| *a != alvo)
                .cloned()
                .collect(),
        }
    }

    pub fn conflitos(&self) -> Vec<String> {
        let mut repetidas: Vec<String> = self
            .ligacoes
            .iter()
            .map(|(_, t)| t.clone())
            .filter(|t| self.alvos_da(t).len() > 1)
            .collect();
        repetidas.sort();
        repetidas.dedup();
        repetidas
    }

    pub fn palavra(&self, apertadas: &[&str]) -> u16 {
        self.ligacoes
            .iter()
            .filter(|(_, t)| apertadas.contains(&t.as_str()))
            .fold(SOLTO, |palavra, (alvo, _)| match alvo {
                Alvo::Botao(bit) => palavra & !(1u16 << bit),
                _ => palavra,
            })
    }

    pub fn eixos(&self, apertadas: &[&str]) -> Eixos {
        let segura = |direito: bool, sentido: Sentido| {
            self.tecla_de(Alvo::Analogico { direito, sentido })
                .is_some_and(|t| apertadas.contains(&t))
        };
        let eixo = |direito: bool, menos: Sentido, mais: Sentido| {
            direcao(segura(direito, menos), segura(direito, mais))
        };
        Eixos {
            esquerdo_x: eixo(false, Sentido::Esquerda, Sentido::Direita),
            esquerdo_y: eixo(false, Sentido::Cima, Sentido::Baixo),
            direito_x: eixo(true, Sentido::Esquerda, Sentido::Direita),
            direito_y: eixo(true, Sentido::Cima, Sentido::Baixo),
        }
    }
}
