use super::alvo::{Alvo, Sentido};
use super::{Entrada, Perfil};

pub const REPETE_APOS: f64 = 0.4;
pub const REPETE_CADA: f64 = 0.12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comando {
    Voltar,
    Ativar,
    Mover(Sentido),
}

/// Controle nos menus: ○ volta, ✖ ativa, direcional move o foco. Os papeis vem do
/// perfil, entao "Faces trocadas" tambem troca quem confirma.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Navegador {
    antes: Vec<Entrada>,
    segurando: Option<(Sentido, f64)>,
}

fn botao(nome: &str) -> Option<u8> {
    match Alvo::botao(nome) {
        Some(Alvo::Botao(bit)) => Some(bit),
        _ => None,
    }
}

impl Navegador {
    pub fn com_antes(&self, antes: &[Entrada]) -> Navegador {
        Navegador {
            antes: antes.to_vec(),
            segurando: None,
        }
    }

    pub fn passo(&self, perfil: &Perfil, agora: &[Entrada], t: f64) -> (Navegador, Vec<Comando>) {
        let novas: Vec<Entrada> = agora
            .iter()
            .copied()
            .filter(|e| !self.antes.contains(e))
            .collect();
        let aciona = |nome: &str, lista: &[Entrada]| {
            let bit = botao(nome);
            lista
                .iter()
                .any(|e| bit.is_some() && perfil.botao_de(*e) == bit)
        };
        let mut comandos = Vec::new();
        if aciona("circle", &novas) {
            comandos.push(Comando::Voltar);
        }
        if aciona("cross", &novas) {
            comandos.push(Comando::Ativar);
        }
        let sentido_de = |e: &Entrada| {
            perfil
                .botao_de(*e)
                .and_then(|b| Alvo::Botao(b).sentido_do_direcional())
        };
        let novo = novas.iter().find_map(sentido_de);
        let segurado = agora.iter().find_map(sentido_de);
        let segurando = match (novo, segurado, self.segurando) {
            (Some(s), _, _) => {
                comandos.push(Comando::Mover(s));
                Some((s, t + REPETE_APOS))
            }
            (None, Some(s), Some((antes, proximo))) if s == antes && t >= proximo => {
                comandos.push(Comando::Mover(s));
                Some((s, t + REPETE_CADA))
            }
            (None, Some(s), Some((antes, proximo))) if s == antes => Some((s, proximo)),
            (None, Some(s), _) => Some((s, t + REPETE_APOS)),
            (None, None, _) => None,
        };
        (
            Navegador {
                antes: agora.to_vec(),
                segurando,
            },
            comandos,
        )
    }
}

/// Primeira entrada apertada neste quadro e que nao estava apertada no anterior. Botao
/// tem prioridade: analogico com folga oscila perto da zona morta e roubaria a captura.
pub fn primeira_nova(antes: &[Entrada], agora: &[Entrada]) -> Option<Entrada> {
    let novas: Vec<Entrada> = agora
        .iter()
        .copied()
        .filter(|e| !antes.contains(e))
        .collect();
    novas
        .iter()
        .copied()
        .find(|e| !e.e_eixo())
        .or_else(|| novas.first().copied())
}
