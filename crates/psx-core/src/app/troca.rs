use crate::dualshock::Rumble;

/// Um segundo emulado: a BIOS e os jogos multidisco so percebem a troca se a porta ficar
/// aberta por tempo que o drive note (INT5 + parar o motor).
pub const CICLOS_DE_PORTA_ABERTA: u64 = 33_868_800;

const MARCAS_DE_DISCO: [&str; 4] = ["(disc", "(disk", "(cd", "[disc"];

/// Titulo sem o sufixo de disco: "Final Fantasy IX (USA) (Disc 2)" vira
/// "Final Fantasy IX (USA)". O que vem depois da marca (ex.: "(Rev 1)") tambem sai.
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

/// Ordena os discos para a troca: os do mesmo jogo primeiro (outros discos do mesmo
/// titulo), depois o resto; dentro de cada grupo, por nome sem diferenciar caixa.
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
    candidatos.sort_by_key(|c| {
        (
            !c.mesmo_jogo,
            nomes.get(c.indice).map(|n| n.to_lowercase()),
        )
    });
    candidatos
}

/// Porta aberta aguardando fechar. Vive no frontend, fora do save state: e a mao do
/// jogador, nao o console.
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

/// Motor pequeno do DualShock e liga/desliga; o grande tem 256 niveis. gilrs pede u16.
pub fn forca_de_vibracao(rumble: Rumble) -> (u16, u16) {
    let forte = u16::from(rumble.large) * 0x0101;
    let fraco = if rumble.small { u16::MAX } else { 0 };
    (forte, fraco)
}

/// Borda de subida: segurar o botao Analog nao pode alternar o modo a cada quadro.
pub fn apertou_agora(antes: bool, agora: bool) -> bool {
    agora && !antes
}
