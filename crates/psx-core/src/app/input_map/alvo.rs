use serde::{Deserialize, Serialize};

use crate::pad_script::{button_bit, button_name};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sentido {
    Esquerda,
    Direita,
    Cima,
    Baixo,
}

impl Sentido {
    pub fn seta(&self) -> &'static str {
        match self {
            Sentido::Esquerda => "⬅",
            Sentido::Direita => "➡",
            Sentido::Cima => "⬆",
            Sentido::Baixo => "⬇",
        }
    }

    fn chave(&self) -> &'static str {
        match self {
            Sentido::Esquerda => "esquerda",
            Sentido::Direita => "direita",
            Sentido::Cima => "cima",
            Sentido::Baixo => "baixo",
        }
    }
}

/// O que uma tecla ou botao fisico aciona no DualShock emulado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Alvo {
    Botao(u8),
    Analogico { direito: bool, sentido: Sentido },
    Analog,
}

const fn esq(sentido: Sentido) -> Alvo {
    Alvo::Analogico {
        direito: false,
        sentido,
    }
}

const fn dir(sentido: Sentido) -> Alvo {
    Alvo::Analogico {
        direito: true,
        sentido,
    }
}

pub const LINHAS: [Alvo; 25] = [
    Alvo::Botao(14),
    Alvo::Botao(13),
    Alvo::Botao(15),
    Alvo::Botao(12),
    Alvo::Botao(10),
    Alvo::Botao(8),
    Alvo::Botao(11),
    Alvo::Botao(9),
    Alvo::Botao(1),
    Alvo::Botao(2),
    Alvo::Botao(0),
    Alvo::Botao(3),
    Alvo::Botao(4),
    Alvo::Botao(6),
    Alvo::Botao(7),
    Alvo::Botao(5),
    esq(Sentido::Esquerda),
    esq(Sentido::Direita),
    esq(Sentido::Cima),
    esq(Sentido::Baixo),
    dir(Sentido::Esquerda),
    dir(Sentido::Direita),
    dir(Sentido::Cima),
    dir(Sentido::Baixo),
    Alvo::Analog,
];

impl Alvo {
    pub fn botao(nome: &str) -> Option<Alvo> {
        button_bit(nome).map(Alvo::Botao)
    }

    pub fn ordem(&self) -> usize {
        LINHAS
            .iter()
            .position(|a| a == self)
            .unwrap_or(LINHAS.len())
    }

    pub fn chave(&self) -> String {
        match self {
            Alvo::Botao(bit) => button_name(*bit).unwrap_or("?").to_string(),
            Alvo::Analogico { direito, sentido } => {
                let lado = if *direito { "direito" } else { "esquerdo" };
                format!("analogico-{lado}-{}", sentido.chave())
            }
            Alvo::Analog => crate::pad_script::ANALOG_BUTTON.to_string(),
        }
    }

    pub fn de_chave(texto: &str) -> Option<Alvo> {
        LINHAS.into_iter().find(|a| a.chave() == texto)
    }

    pub fn rotulo(&self) -> String {
        match self {
            Alvo::Botao(bit) => rotulo_do_botao(*bit),
            Alvo::Analogico { direito, sentido } => {
                let lado = if *direito { "direito" } else { "esquerdo" };
                format!("Analógico {lado} {}", sentido.seta())
            }
            Alvo::Analog => "Botão Analog".to_string(),
        }
    }

    pub fn sentido_do_direcional(&self) -> Option<Sentido> {
        match self {
            Alvo::Botao(4) => Some(Sentido::Cima),
            Alvo::Botao(5) => Some(Sentido::Direita),
            Alvo::Botao(6) => Some(Sentido::Baixo),
            Alvo::Botao(7) => Some(Sentido::Esquerda),
            _ => None,
        }
    }
}

fn rotulo_do_botao(bit: u8) -> String {
    let fixo = match button_name(bit).unwrap_or("") {
        "cross" => "✖ Xis",
        "circle" => "○ Bola",
        "square" => "◻ Quadrado",
        "triangle" => "⏶ Triângulo",
        "select" => "Select",
        "start" => "Start",
        "up" => "Direcional ⬆",
        "down" => "Direcional ⬇",
        "left" => "Direcional ⬅",
        "right" => "Direcional ➡",
        outro => return outro.to_ascii_uppercase(),
    };
    fixo.to_string()
}
