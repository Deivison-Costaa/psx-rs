use serde::{Deserialize, Serialize};

pub const LIMITE_DE_RECENTES: usize = 10;
pub const VELOCIDADES: [u32; 4] = [1, 2, 4, 8];
pub const VELOCIDADE_MAX: u32 = 8;

const SEGUNDOS_POR_MINUTO: u64 = 60;
const SEGUNDOS_POR_HORA: u64 = 3600;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recente {
    pub serial: String,
    pub titulo: String,
    pub segundos: u64,
    pub ultima_vez: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recentes {
    itens: Vec<Recente>,
}

impl Recentes {
    pub fn itens(&self) -> &[Recente] {
        &self.itens
    }

    pub fn tempo_de(&self, serial: &str) -> u64 {
        self.itens
            .iter()
            .find(|i| i.serial == serial)
            .map(|i| i.segundos)
            .unwrap_or(0)
    }

    /// Devolve uma lista NOVA com o jogo no topo. `agora` vem de fora: o `psx-core` nao
    /// le relogio (R3). Sem serial nao ha como acumular tempo, entao o jogo nao entra.
    pub fn registra(&self, serial: &str, titulo: &str, segundos: u64, agora: u64) -> Recentes {
        if serial.is_empty() {
            return self.clone();
        }
        let acumulado = self.tempo_de(serial) + segundos;
        let mut itens = vec![Recente {
            serial: serial.to_string(),
            titulo: titulo.to_string(),
            segundos: acumulado,
            ultima_vez: agora,
        }];
        itens.extend(
            self.itens
                .iter()
                .filter(|i| i.serial != serial)
                .take(LIMITE_DE_RECENTES - 1)
                .cloned(),
        );
        Recentes { itens }
    }
}

pub fn passos_por_quadro(base: usize, multiplicador: u32) -> usize {
    base * multiplicador.max(1) as usize
}

pub fn proxima_velocidade(atual: u32) -> u32 {
    let Some(i) = VELOCIDADES.iter().position(|v| *v == atual) else {
        return VELOCIDADES[0];
    };
    VELOCIDADES[(i + 1) % VELOCIDADES.len()]
}

pub fn formata_tempo(segundos: u64) -> String {
    if segundos < SEGUNDOS_POR_MINUTO {
        return format!("{segundos} s");
    }
    if segundos < SEGUNDOS_POR_HORA {
        return format!("{} min", segundos / SEGUNDOS_POR_MINUTO);
    }
    let horas = segundos / SEGUNDOS_POR_HORA;
    let minutos = (segundos % SEGUNDOS_POR_HORA) / SEGUNDOS_POR_MINUTO;
    format!("{horas} h {minutos:02} min")
}
