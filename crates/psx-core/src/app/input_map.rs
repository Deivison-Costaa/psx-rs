use serde::{Deserialize, Serialize};

use crate::dualshock::{STICK_CENTER, Sticks};
use crate::pad_script::{RELEASED, button_bit};

mod alvo;
mod estilo;
mod navegacao;
mod teclado;

pub use alvo::{Alvo, LINHAS, Sentido};
pub use estilo::Estilo;
pub use navegacao::{Comando, Navegador, REPETE_APOS, REPETE_CADA, primeira_nova};
pub use teclado::Teclado;

pub const SOLTO: u16 = RELEASED;
pub const ZONA_MORTA_ANALOGICA: f32 = 0.08;

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
    analog: Option<Entrada>,
    teclado: Teclado,
}

const FORMATO_ATUAL: &str = "formato = 2";
const PREFIXO_TECLADO: &str = "teclado.";

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
            analog: None,
            teclado: Teclado::vazio(),
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
            analog: Some(Entrada::Modo),
            teclado: Teclado::padrao(),
        }
    }

    pub fn padrao() -> Self {
        Self::com_faces("Padrão (PlayStation/Xbox)", "cross", "circle")
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
            ligacoes,
            ..self.clone()
        })
    }

    pub fn desliga(&self, entrada: Entrada) -> Perfil {
        Perfil {
            ligacoes: self
                .ligacoes
                .iter()
                .copied()
                .filter(|(e, _)| *e != entrada)
                .collect(),
            ..self.clone()
        }
    }

    pub fn palavra(&self, pressionados: &[Entrada]) -> u16 {
        self.palavra_no_modo(pressionados, false)
    }

    /// No modo analogico o analogico esquerdo vai para os eixos do DualShock; se tambem
    /// apertasse o direcional, o jogo receberia o mesmo movimento duas vezes.
    pub fn palavra_no_modo(&self, pressionados: &[Entrada], analogico: bool) -> u16 {
        let mut palavra = SOLTO;
        for entrada in pressionados {
            if analogico && entrada.e_eixo() {
                continue;
            }
            if let Some(bit) = self.botao_de(*entrada) {
                palavra &= !(1u16 << bit);
            }
        }
        palavra
    }
}

/// Os dois analogicos em [-1, 1] na convencao do PS1: X cresce para a direita e Y para
/// BAIXO (00h = cima). gilrs entrega Y crescendo para cima; quem traduz inverte.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Eixos {
    pub esquerdo_x: f32,
    pub esquerdo_y: f32,
    pub direito_x: f32,
    pub direito_y: f32,
}

impl Eixos {
    /// Valores crus do gilrs (Y cresce para cima) na convencao do PS1 (Y cresce para baixo).
    pub fn do_controle(esquerdo: (f32, f32), direito: (f32, f32)) -> Eixos {
        Eixos {
            esquerdo_x: esquerdo.0,
            esquerdo_y: -esquerdo.1,
            direito_x: direito.0,
            direito_y: -direito.1,
        }
    }

    /// Por eixo, vale quem esta mais longe do centro: teclado e controle juntos.
    pub fn une(&self, outro: &Eixos) -> Eixos {
        let maior = |a: f32, b: f32| if b.abs() > a.abs() { b } else { a };
        Eixos {
            esquerdo_x: maior(self.esquerdo_x, outro.esquerdo_x),
            esquerdo_y: maior(self.esquerdo_y, outro.esquerdo_y),
            direito_x: maior(self.direito_x, outro.direito_x),
            direito_y: maior(self.direito_y, outro.direito_y),
        }
    }

    pub fn sticks(&self) -> Sticks {
        Sticks {
            right_x: eixo_para_byte(self.direito_x),
            right_y: eixo_para_byte(self.direito_y),
            left_x: eixo_para_byte(self.esquerdo_x),
            left_y: eixo_para_byte(self.esquerdo_y),
        }
    }
}

pub fn eixo_para_byte(valor: f32) -> u8 {
    if !valor.is_finite() || valor.abs() < ZONA_MORTA_ANALOGICA {
        return STICK_CENTER;
    }
    ((valor.clamp(-1.0, 1.0) + 1.0) * 127.5).round() as u8
}

/// Tecla de direcao como eixo digital: as duas juntas se anulam, como no direcional.
pub fn direcao(negativo: bool, positivo: bool) -> f32 {
    match (negativo, positivo) {
        (true, false) => -1.0,
        (false, true) => 1.0,
        _ => 0.0,
    }
}

impl Default for Perfil {
    fn default() -> Self {
        Self::padrao()
    }
}

impl Entrada {
    pub fn e_eixo(&self) -> bool {
        matches!(self, Entrada::EixoNegativo(_) | Entrada::EixoPositivo(_))
    }

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
        let controle = self.ligacoes.iter().filter_map(|(e, bit)| {
            crate::pad_script::button_name(*bit).map(|b| format!("{} = {b}", e.nome()))
        });
        let analog = self
            .analog
            .map(|e| format!("{} = {}", e.nome(), Alvo::Analog.chave()));
        let teclado = self
            .teclado
            .ligacoes()
            .iter()
            .map(|(a, t)| format!("{PREFIXO_TECLADO}{t} = {}", a.chave()));
        let mut linhas: Vec<String> = controle.chain(analog).chain(teclado).collect();
        linhas.sort();
        linhas.insert(0, FORMATO_ATUAL.to_string());
        linhas.join("\n") + "\n"
    }

    /// Arquivo sem a linha de formato e do tempo em que o teclado era fixo e o Home
    /// sempre era o Analog: le com esses padroes em vez de deixar o usuario sem teclado.
    pub fn de_texto(nome: &str, texto: &str) -> Perfil {
        let limpas: Vec<&str> = texto
            .lines()
            .map(|l| l.split('#').next().unwrap_or("").trim())
            .collect();
        let atual = limpas
            .iter()
            .any(|l| l.split_whitespace().collect::<String>() == FORMATO_ATUAL.replace(' ', ""));
        let inicial = Perfil {
            analog: (!atual).then_some(Entrada::Modo),
            teclado: if atual {
                Teclado::vazio()
            } else {
                Teclado::padrao()
            },
            ..Perfil::vazio(nome)
        };
        limpas.iter().fold(inicial, |perfil, linha| {
            let Some((esquerda, direita)) = linha.split_once('=') else {
                return perfil;
            };
            perfil.le_linha(esquerda.trim(), direita.trim())
        })
    }

    fn le_linha(self, esquerda: &str, direita: &str) -> Perfil {
        if let Some(tecla) = esquerda.strip_prefix(PREFIXO_TECLADO) {
            return match Alvo::de_chave(direita) {
                Some(alvo) if !tecla.is_empty() => Perfil {
                    teclado: self.teclado.associa(alvo, tecla),
                    ..self
                },
                _ => self,
            };
        }
        let Some(entrada) = Entrada::de_nome(esquerda) else {
            return self;
        };
        if Alvo::de_chave(direita) == Some(Alvo::Analog) {
            return self.associa_controle(Alvo::Analog, entrada);
        }
        self.liga(entrada, direita).unwrap_or(self)
    }

    pub fn teclado(&self) -> &Teclado {
        &self.teclado
    }

    pub fn com_teclado(&self, teclado: Teclado) -> Perfil {
        Perfil {
            teclado,
            ..self.clone()
        }
    }

    /// Perfil pronto so troca a coluna do controle: quem ajustou o teclado nao perde.
    pub fn controle_de(&self, outro: &Perfil) -> Perfil {
        Perfil {
            teclado: self.teclado.clone(),
            ..outro.clone()
        }
    }

    pub fn analog(&self) -> Option<Entrada> {
        self.analog
    }

    pub fn entradas_de(&self, alvo: Alvo) -> Vec<Entrada> {
        match alvo {
            Alvo::Botao(bit) => self
                .ligacoes
                .iter()
                .filter(|(_, b)| *b == bit)
                .map(|(e, _)| *e)
                .collect(),
            Alvo::Analog => self.analog.into_iter().collect(),
            Alvo::Analogico { .. } => Vec::new(),
        }
    }

    /// Associar pela tela substitui so a entrada do mesmo tipo: trocar o botao do ↑ nao
    /// tira o analogico que tambem aponta para ele.
    pub fn associa_controle(&self, alvo: Alvo, entrada: Entrada) -> Perfil {
        let sem = self.desliga(entrada);
        let sem = Perfil {
            analog: sem.analog.filter(|e| *e != entrada),
            ..sem
        };
        match alvo {
            Alvo::Botao(bit) => {
                let mut ligacoes: Vec<(Entrada, u8)> = sem
                    .ligacoes
                    .iter()
                    .copied()
                    .filter(|(e, b)| *b != bit || e.e_eixo() != entrada.e_eixo())
                    .collect();
                ligacoes.push((entrada, bit));
                Perfil { ligacoes, ..sem }
            }
            Alvo::Analog => Perfil {
                analog: Some(entrada),
                ..sem
            },
            Alvo::Analogico { .. } => self.clone(),
        }
    }

    pub fn limpa_controle(&self, alvo: Alvo) -> Perfil {
        match alvo {
            Alvo::Botao(bit) => Perfil {
                ligacoes: self
                    .ligacoes
                    .iter()
                    .copied()
                    .filter(|(_, b)| *b != bit)
                    .collect(),
                ..self.clone()
            },
            Alvo::Analog => Perfil {
                analog: None,
                ..self.clone()
            },
            Alvo::Analogico { .. } => self.clone(),
        }
    }
}
