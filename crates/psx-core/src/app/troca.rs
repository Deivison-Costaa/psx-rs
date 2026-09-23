use crate::dualshock::Rumble;

/// Um segundo emulado: porta que fecha no mesmo instante o jogo nem percebe.
pub const CICLOS_DE_PORTA_ABERTA: u64 = 33_868_800;

const MARCAS_DE_DISCO: [&str; 4] = ["(disc", "(disk", "(cd", "[disc"];

/// "Jogo (USA) (Disc 2) (Rev 1)" vira "Jogo (USA)".
pub fn titulo_base(nome: &str) -> String {
    let minusculo = nome.to_lowercase();
    let corte = MARCAS_DE_DISCO
        .iter()
        .filter_map(|marca| minusculo.find(marca))
        .min()
        .unwrap_or(nome.len());
    let base = nome.get(..corte).unwrap_or(nome);
    base.trim().to_string()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidato {
    pub indice: usize,
    pub mesmo_jogo: bool,
    pub atual: bool,
}

/// Outros discos do mesmo titulo primeiro, depois o resto, cada grupo por nome.
pub fn ordena_candidatos(atual: &str, nomes: &[String]) -> Vec<Candidato> {
    let base = titulo_base(atual).to_lowercase();
    let mut candidatos: Vec<Candidato> = nomes
        .iter()
        .enumerate()
        .map(|(indice, nome)| Candidato {
            indice,
            mesmo_jogo: !base.is_empty() && titulo_base(nome).to_lowercase() == base,
            atual: nome == atual,
        })
        .collect();
    candidatos.sort_by_key(|c| (!c.mesmo_jogo, nomes.get(c.indice).map(|n| n.to_lowercase())));
    candidatos
}

/// Vive no frontend, fora do save state: e a mao do jogador, nao o console.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortaAberta {
    fecha_em: u64,
}

impl PortaAberta {
    pub fn desde(ciclo: u64) -> Self {
        PortaAberta {
            fecha_em: ciclo.saturating_add(CICLOS_DE_PORTA_ABERTA),
        }
    }

    pub fn deve_fechar(&self, ciclo: u64) -> bool {
        ciclo >= self.fecha_em
    }
}

pub fn forca_de_vibracao(rumble: Rumble) -> (u16, u16) {
    let forte = u16::from(rumble.large) * 0x0101;
    let fraco = if rumble.small { u16::MAX } else { 0 };
    (forte, fraco)
}

pub fn apertou_agora(antes: bool, agora: bool) -> bool {
    agora && !antes
}
