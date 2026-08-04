use serde::{Deserialize, Serialize};

use crate::app::sessao::Recentes;

pub const ESCALA_MIN: u32 = 1;
pub const ESCALA_MAX: u32 = 6;
pub const VOLUME_MAX: u8 = 100;
pub const SLOT_MAX: u8 = 9;

const PASTA_JOGOS: &str = ".";
const PASTA_CARTOES: &str = "cartoes";
const PASTA_SAVES: &str = "saves";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub bios: String,
    pub pasta_de_jogos: String,
    pub pasta_de_cartoes: String,
    pub pasta_de_saves: String,
    pub escala: u32,
    pub filtro_linear: bool,
    pub volume: u8,
    pub audio_ligado: bool,
    pub slot_inicial: u8,
    #[serde(default)]
    pub recentes: Recentes,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            bios: String::new(),
            pasta_de_jogos: PASTA_JOGOS.to_string(),
            pasta_de_cartoes: PASTA_CARTOES.to_string(),
            pasta_de_saves: PASTA_SAVES.to_string(),
            escala: 2,
            filtro_linear: false,
            volume: VOLUME_MAX,
            audio_ligado: true,
            slot_inicial: 0,
            recentes: Recentes::default(),
        }
    }
}

fn ou_padrao(valor: &str, padrao: &str) -> String {
    if valor.trim().is_empty() {
        padrao.to_string()
    } else {
        valor.to_string()
    }
}

impl Config {
    /// Devolve uma copia dentro das faixas. Grampear em vez de recusar: arquivo de
    /// configuracao editado a mao com um numero absurdo tem de abrir o app assim mesmo.
    pub fn ajustada(&self) -> Config {
        Config {
            bios: self.bios.clone(),
            pasta_de_jogos: ou_padrao(&self.pasta_de_jogos, PASTA_JOGOS),
            pasta_de_cartoes: ou_padrao(&self.pasta_de_cartoes, PASTA_CARTOES),
            pasta_de_saves: ou_padrao(&self.pasta_de_saves, PASTA_SAVES),
            escala: self.escala.clamp(ESCALA_MIN, ESCALA_MAX),
            filtro_linear: self.filtro_linear,
            volume: self.volume.min(VOLUME_MAX),
            audio_ligado: self.audio_ligado,
            slot_inicial: self.slot_inicial.min(SLOT_MAX),
            recentes: self.recentes.clone(),
        }
    }

    /// O que o usuario precisa consertar, em texto. Roda ANTES do grampeamento: o valor
    /// fora de faixa vira aviso na tela, nao correcao silenciosa.
    pub fn valida(&self) -> Vec<String> {
        let mut problemas = Vec::new();
        if self.bios.trim().is_empty() {
            problemas.push("caminho da BIOS vazio: aponte para a BIOS do seu console".into());
        }
        if !(ESCALA_MIN..=ESCALA_MAX).contains(&self.escala) {
            problemas.push(format!(
                "escala {} fora de {ESCALA_MIN}..{ESCALA_MAX}",
                self.escala
            ));
        }
        if self.volume > VOLUME_MAX {
            problemas.push(format!("volume {} acima de {VOLUME_MAX}", self.volume));
        }
        if self.slot_inicial > SLOT_MAX {
            problemas.push(format!("slot {} acima de {SLOT_MAX}", self.slot_inicial));
        }
        problemas
    }

    pub fn ganho(&self) -> f32 {
        if !self.audio_ligado {
            return 0.0;
        }
        f32::from(self.volume.min(VOLUME_MAX)) / f32::from(VOLUME_MAX)
    }
}
