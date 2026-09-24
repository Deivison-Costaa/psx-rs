use serde::{Deserialize, Serialize};

pub const DURACAO_DO_TOAST: f64 = 3.0;
const ESMAECIMENTO_DO_TOAST: f64 = 0.5;
const JANELA_DO_MEDIDOR: f64 = 0.5;
const FOLGA_DE_ARREDONDAMENTO: f32 = 1e-3;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModoDeImagem {
    #[default]
    AjustarAJanela,
    EscalaInteira,
}

impl ModoDeImagem {
    pub const TODOS: [ModoDeImagem; 2] =
        [ModoDeImagem::AjustarAJanela, ModoDeImagem::EscalaInteira];

    pub fn rotulo(self) -> &'static str {
        match self {
            ModoDeImagem::AjustarAJanela => "Ajustar à janela",
            ModoDeImagem::EscalaInteira => "Escala inteira",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Retangulo {
    pub x: f32,
    pub y: f32,
    pub largura: f32,
    pub altura: f32,
}

/// `base` e a imagem ja em 4:3 na escala 1. Escala inteira que nao cabe nem em 1x cai no
/// ajuste livre: imagem encolhida e melhor que imagem cortada.
pub fn retangulo_da_imagem(janela: (f32, f32), base: (f32, f32), modo: ModoDeImagem) -> Retangulo {
    let (jl, ja) = janela;
    let (bl, ba) = base;
    if jl <= 0.0 || ja <= 0.0 || bl <= 0.0 || ba <= 0.0 {
        return Retangulo::default();
    }
    let livre = (jl / bl).min(ja / ba);
    let escala = match modo {
        ModoDeImagem::EscalaInteira if livre >= 1.0 => livre.floor(),
        _ => livre,
    };
    let largura = (bl * escala + FOLGA_DE_ARREDONDAMENTO).floor().min(jl);
    let altura = (ba * escala + FOLGA_DE_ARREDONDAMENTO).floor().min(ja);
    Retangulo {
        x: ((jl - largura) / 2.0).floor(),
        y: ((ja - altura) / 2.0).floor(),
        largura,
        altura,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Toast {
    pub texto: String,
    pub desde: f64,
}

impl Toast {
    pub fn novo(texto: &str, agora: f64) -> Toast {
        Toast {
            texto: texto.to_string(),
            desde: agora,
        }
    }

    pub fn opacidade(&self, agora: f64) -> f32 {
        let idade = (agora - self.desde).max(0.0);
        let resta = DURACAO_DO_TOAST - idade;
        if resta <= 0.0 {
            return 0.0;
        }
        (resta / ESMAECIMENTO_DO_TOAST).min(1.0) as f32
    }

    pub fn expirou(&self, agora: f64) -> bool {
        self.opacidade(agora) <= 0.0
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Sobreposicao {
    #[default]
    Nenhuma,
    Status,
    Ajuda,
}

impl Sobreposicao {
    pub fn proxima(self) -> Sobreposicao {
        match self {
            Sobreposicao::Nenhuma => Sobreposicao::Status,
            Sobreposicao::Status => Sobreposicao::Ajuda,
            Sobreposicao::Ajuda => Sobreposicao::Nenhuma,
        }
    }
}

/// Quadros EMULADOS por segundo real: cai abaixo de 60 quando a maquina nao acompanha,
/// e multiplica no fast-forward.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct MedidorDeQuadros {
    ciclos: u64,
    segundos: f64,
    fps: Option<f64>,
}

impl MedidorDeQuadros {
    pub fn acumula(&self, ciclos: u64, segundos: f64, ciclos_por_quadro: u64) -> MedidorDeQuadros {
        if ciclos_por_quadro == 0 {
            return *self;
        }
        let total_ciclos = self.ciclos + ciclos;
        let total_segundos = self.segundos + segundos;
        if total_segundos < JANELA_DO_MEDIDOR {
            return MedidorDeQuadros {
                ciclos: total_ciclos,
                segundos: total_segundos,
                fps: self.fps,
            };
        }
        let quadros = total_ciclos as f64 / ciclos_por_quadro as f64;
        MedidorDeQuadros {
            ciclos: 0,
            segundos: 0.0,
            fps: Some(quadros / total_segundos),
        }
    }

    pub fn fps(&self) -> Option<f64> {
        self.fps
    }
}
