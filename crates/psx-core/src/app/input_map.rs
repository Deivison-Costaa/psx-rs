use serde::{Deserialize, Serialize};

use crate::pad_script::{RELEASED, button_bit};

pub const SOLTO: u16 = RELEASED;

/// Vocabulario de entrada fisica, independente de biblioteca de gamepad. Quem traduz o
/// `gilrs::Button` para ca e o frontend — o `psx-core` nao conhece gilrs (R3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Entrada {
    Sul,
    Leste,
    Norte,
    Oeste,
    L1,
    L2,
    R1,
    R2,
    L3,
    R3,
    Select,
    Start,
    Modo,
    DpadCima,
    DpadBaixo,
    DpadEsquerda,
    DpadDireita,
    EixoNegativo(u8),
    EixoPositivo(u8),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Perfil {
    pub nome: String,
    ligacoes: Vec<(Entrada, u8)>,
}

const COMUNS: [(Entrada, &str); 14] = [
    (Entrada::Norte, "triangle"),
    (Entrada::Oeste, "square"),
    (Entrada::L1, "l1"),
    (Entrada::L2, "l2"),
    (Entrada::R1, "r1"),
    (Entrada::R2, "r2"),
    (Entrada::L3, "l3"),
    (Entrada::R3, "r3"),
    (Entrada::Select, "select"),
    (Entrada::Start, "start"),
    (Entrada::DpadCima, "up"),
    (Entrada::DpadBaixo, "down"),
    (Entrada::DpadEsquerda, "left"),
    (Entrada::DpadDireita, "right"),
];

const ANALOGICO: [(Entrada, &str); 4] = [
    (Entrada::EixoNegativo(0), "left"),
    (Entrada::EixoPositivo(0), "right"),
    (Entrada::EixoNegativo(1), "down"),
    (Entrada::EixoPositivo(1), "up"),
];

impl Perfil {
    pub fn vazio(nome: &str) -> Self {
        Perfil {
            nome: nome.to_string(),
            ligacoes: Vec::new(),
        }
    }

    fn com_faces(nome: &str, sul: &str, leste: &str) -> Self {
        let mut ligacoes = vec![(Entrada::Sul, sul), (Entrada::Leste, leste)];
        ligacoes.extend_from_slice(&COMUNS);
        ligacoes.extend_from_slice(&ANALOGICO);
        Perfil {
            nome: nome.to_string(),
            ligacoes: ligacoes
                .into_iter()
                .filter_map(|(e, b)| button_bit(b).map(|bit| (e, bit)))
                .collect(),
        }
    }

    pub fn padrao() -> Self {
        Self::com_faces("Padrao (PlayStation/Xbox)", "cross", "circle")
    }

    /// Controle cujo A/B fisico e invertido em relacao ao PlayStation (estilo Nintendo):
    /// o botao de baixo confirma, e quem confirma no PS1 e o X.
    pub fn faces_trocadas() -> Self {
        Self::com_faces("Faces trocadas (estilo Nintendo)", "circle", "cross")
    }

    pub fn ligacoes(&self) -> &[(Entrada, u8)] {
        &self.ligacoes
    }

    pub fn botao_de(&self, entrada: Entrada) -> Option<u8> {
        self.ligacoes
            .iter()
            .find(|(e, _)| *e == entrada)
            .map(|(_, b)| *b)
    }

    pub fn nome_do_botao(&self, entrada: Entrada) -> Option<&'static str> {
        let bit = self.botao_de(entrada)?;
        crate::pad_script::button_name(bit)
    }

    /// Devolve um perfil NOVO: remapear durante o jogo nao pode alterar o perfil que o
    /// usuario ainda nao confirmou. Uma entrada fisica so aciona um botao, entao religar
    /// substitui a ligacao anterior em vez de somar outra.
    pub fn liga(&self, entrada: Entrada, botao: &str) -> Result<Perfil, String> {
        let bit = button_bit(botao).ok_or_else(|| format!("botao desconhecido: '{botao}'"))?;
        let mut ligacoes: Vec<(Entrada, u8)> = self
            .ligacoes
            .iter()
            .copied()
            .filter(|(e, _)| *e != entrada)
            .collect();
        ligacoes.push((entrada, bit));
        Ok(Perfil {
            nome: self.nome.clone(),
            ligacoes,
        })
    }

    pub fn desliga(&self, entrada: Entrada) -> Perfil {
        Perfil {
            nome: self.nome.clone(),
            ligacoes: self
                .ligacoes
                .iter()
                .copied()
                .filter(|(e, _)| *e != entrada)
                .collect(),
        }
    }

    pub fn palavra(&self, pressionados: &[Entrada]) -> u16 {
        let mut palavra = SOLTO;
        for entrada in pressionados {
            if let Some(bit) = self.botao_de(*entrada) {
                palavra &= !(1u16 << bit);
            }
        }
        palavra
    }
}

impl Default for Perfil {
    fn default() -> Self {
        Self::padrao()
    }
}

impl Entrada {
    pub fn nome(&self) -> String {
        match self {
            Entrada::Sul => "sul".into(),
            Entrada::Leste => "leste".into(),
            Entrada::Norte => "norte".into(),
            Entrada::Oeste => "oeste".into(),
            Entrada::L1 => "l1".into(),
            Entrada::L2 => "l2".into(),
            Entrada::R1 => "r1".into(),
            Entrada::R2 => "r2".into(),
            Entrada::L3 => "l3".into(),
            Entrada::R3 => "r3".into(),
            Entrada::Select => "select".into(),
            Entrada::Start => "start".into(),
            Entrada::Modo => "modo".into(),
            Entrada::DpadCima => "dpad-cima".into(),
            Entrada::DpadBaixo => "dpad-baixo".into(),
            Entrada::DpadEsquerda => "dpad-esquerda".into(),
            Entrada::DpadDireita => "dpad-direita".into(),
            Entrada::EixoNegativo(n) => format!("eixo-{n}-negativo"),
            Entrada::EixoPositivo(n) => format!("eixo-{n}-positivo"),
        }
    }

    pub fn de_nome(texto: &str) -> Option<Entrada> {
        for fixa in TODAS_FIXAS {
            if fixa.nome() == texto {
                return Some(fixa);
            }
        }
        let numero = texto.strip_prefix("eixo-")?;
        let (n, sinal) = numero.split_once('-')?;
        let n: u8 = n.parse().ok()?;
        match sinal {
            "negativo" => Some(Entrada::EixoNegativo(n)),
            "positivo" => Some(Entrada::EixoPositivo(n)),
            _ => None,
        }
    }
}

pub const TODAS_FIXAS: [Entrada; 17] = [
    Entrada::Sul,
    Entrada::Leste,
    Entrada::Norte,
    Entrada::Oeste,
    Entrada::L1,
    Entrada::L2,
    Entrada::R1,
    Entrada::R2,
    Entrada::L3,
    Entrada::R3,
    Entrada::Select,
    Entrada::Start,
    Entrada::Modo,
    Entrada::DpadCima,
    Entrada::DpadBaixo,
    Entrada::DpadEsquerda,
    Entrada::DpadDireita,
];

impl Perfil {
    /// Formato de linha `entrada = botao`, nao TOML/JSON: as variantes de eixo carregam
    /// numero, e enum com payload vira tabela aninhada em TOML — ilegivel de editar a mao,
    /// que e justamente o motivo de um perfil de controle virar arquivo de texto.
    pub fn para_texto(&self) -> String {
        let mut linhas: Vec<String> = self
            .ligacoes
            .iter()
            .filter_map(|(e, bit)| {
                crate::pad_script::button_name(*bit).map(|b| format!("{} = {b}", e.nome()))
            })
            .collect();
        linhas.sort();
        linhas.join("\n") + "\n"
    }

    pub fn de_texto(nome: &str, texto: &str) -> Perfil {
        let mut perfil = Perfil::vazio(nome);
        for linha in texto.lines() {
            let sem_comentario = linha.split('#').next().unwrap_or("");
            let Some((esquerda, direita)) = sem_comentario.split_once('=') else {
                continue;
            };
            let Some(entrada) = Entrada::de_nome(esquerda.trim()) else {
                continue;
            };
            if let Ok(novo) = perfil.liga(entrada, direita.trim()) {
                perfil = novo;
            }
        }
        perfil
    }
}
