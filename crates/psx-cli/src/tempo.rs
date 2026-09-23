use psx_core::dualshock::Sticks;
use psx_core::pad_script::{ANALOG_BUTTON, PadScript};

pub const CICLOS_POR_SEGUNDO: u64 = 33_868_800;
pub const DURACAO_PADRAO_S: &str = "0.1s";

/// Numero `N` (passos) ou `Ns` (segundos emulados, convertidos em ciclos do barramento).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instante {
    Passo(u64),
    Ciclo(u64),
}

pub fn segundos_em_ciclos(texto: &str) -> Result<u64, String> {
    let numero = texto
        .strip_suffix('s')
        .ok_or_else(|| format!("'{texto}': falta o sufixo 's'"))?;
    match numero.parse::<f64>() {
        Ok(s) if s.is_finite() && s >= 0.0 => Ok((s * CICLOS_POR_SEGUNDO as f64).round() as u64),
        _ => Err(format!(
            "'{texto}': segundos tem de ser um numero decimal >= 0 (ex.: 12.5s)"
        )),
    }
}

pub fn instante_de(texto: &str) -> Result<Instante, String> {
    if texto.ends_with('s') {
        return segundos_em_ciclos(texto).map(Instante::Ciclo);
    }
    texto
        .parse::<u64>()
        .map(Instante::Passo)
        .map_err(|e| format!("'{texto}': espera um passo decimal ou segundos com 's': {e}"))
}

pub fn em_segundos(spec: &str) -> bool {
    spec.rsplit_once('@')
        .map(|(_, quando)| quando.split(':').next().unwrap_or("").ends_with('s'))
        .unwrap_or(false)
}

/// `ALVO@12.5s[:0.1s]` vira `ALVO@CICLO:DURACAO` em ciclos, para o `PadScript` avaliar
/// contra o contador de ciclos do barramento.
pub fn spec_em_ciclos(spec: &str) -> Result<String, String> {
    let (alvo, quando) = spec
        .rsplit_once('@')
        .ok_or_else(|| format!("'{spec}': falta '@'"))?;
    let (inicio, duracao) = quando.split_once(':').unwrap_or((quando, DURACAO_PADRAO_S));
    if !duracao.ends_with('s') {
        return Err(format!(
            "'{spec}': inicio em segundos exige duracao em segundos (ex.: @12.5s:0.1s)"
        ));
    }
    let inicio = segundos_em_ciclos(inicio).map_err(|e| format!("'{spec}': {e}"))?;
    let duracao = segundos_em_ciclos(duracao).map_err(|e| format!("'{spec}': {e}"))?;
    if duracao == 0 {
        return Err(format!("'{spec}': a duracao tem de ser maior que zero"));
    }
    Ok(format!("{alvo}@{inicio}:{duracao}"))
}

/// Dois roteiros de controle: um medido em passos (forma antiga) e outro em ciclos
/// (forma com segundos). O botao Analog em segundos dispara no primeiro passo que cruza o ciclo.
#[derive(Debug, Default)]
pub struct Roteiro {
    pub passos: PadScript,
    pub ciclos: PadScript,
    analog_ciclos: Vec<u64>,
}

impl Roteiro {
    pub fn monta(presses: &[String], sticks: &[String]) -> Result<Self, String> {
        let mut press_passos = Vec::new();
        let mut press_ciclos = Vec::new();
        let mut analog_ciclos = Vec::new();
        for spec in presses {
            if !em_segundos(spec) {
                press_passos.push(spec.clone());
                continue;
            }
            let convertido = spec_em_ciclos(spec)?;
            match convertido.split_once('@') {
                Some((nome, resto)) if nome.eq_ignore_ascii_case(ANALOG_BUTTON) => {
                    let ciclo = resto.split(':').next().unwrap_or("0");
                    analog_ciclos.push(ciclo.parse::<u64>().map_err(|e| e.to_string())?);
                }
                _ => press_ciclos.push(convertido),
            }
        }
        let (stick_ciclos, stick_passos): (Vec<String>, Vec<String>) =
            sticks.iter().cloned().partition(|s| em_segundos(s));
        let stick_ciclos = stick_ciclos
            .iter()
            .map(|s| spec_em_ciclos(s))
            .collect::<Result<Vec<_>, _>>()?;
        analog_ciclos.sort_unstable();
        Ok(Self {
            passos: PadScript::parse(&press_passos)?.with_sticks(&stick_passos)?,
            ciclos: PadScript::parse(&press_ciclos)?.with_sticks(&stick_ciclos)?,
            analog_ciclos,
        })
    }

    pub fn is_empty(&self) -> bool {
        self.passos.is_empty() && self.ciclos.is_empty() && self.analog_ciclos.is_empty()
    }

    pub fn uses_dualshock(&self) -> bool {
        self.passos.uses_dualshock()
            || self.ciclos.uses_dualshock()
            || !self.analog_ciclos.is_empty()
    }

    pub fn buttons_at(&self, passo: u64, ciclo: u64) -> u16 {
        self.passos.buttons_at(passo) & self.ciclos.buttons_at(ciclo)
    }

    pub fn sticks_at(&self, passo: u64, ciclo: u64) -> Sticks {
        let a = self.passos.sticks_at(passo);
        let b = self.ciclos.sticks_at(ciclo);
        let c = Sticks::CENTERED;
        let esquerdo_a = (a.left_x, a.left_y) != (c.left_x, c.left_y);
        let direito_a = (a.right_x, a.right_y) != (c.right_x, c.right_y);
        Sticks {
            left_x: if esquerdo_a { a.left_x } else { b.left_x },
            left_y: if esquerdo_a { a.left_y } else { b.left_y },
            right_x: if direito_a { a.right_x } else { b.right_x },
            right_y: if direito_a { a.right_y } else { b.right_y },
        }
    }

    /// Algum aperto de Analog em segundos caiu entre `ciclo_antes` (exclusivo) e `ciclo` (inclusivo)?
    pub fn analog_press_at(&self, passo: u64, ciclo_antes: u64, ciclo: u64) -> bool {
        self.passos.analog_press_at(passo)
            || self
                .analog_ciclos
                .iter()
                .any(|&c| (c > ciclo_antes || (c == 0 && ciclo_antes == 0)) && c <= ciclo)
    }
}

/// Cadencia de algo periodico (dump de VRAM, amostra de PC): a cada N passos ou a cada N ciclos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cadencia {
    Passos(u64),
    Ciclos(u64),
}

impl Cadencia {
    pub fn de(texto: &str) -> Result<Self, String> {
        match instante_de(texto)? {
            Instante::Passo(0) | Instante::Ciclo(0) => {
                Err(format!("'{texto}': o intervalo tem de ser maior que zero"))
            }
            Instante::Passo(n) => Ok(Self::Passos(n)),
            Instante::Ciclo(n) => Ok(Self::Ciclos(n)),
        }
    }

    /// Indice do periodo que acabou de fechar neste passo, se algum fechou.
    pub fn marco(self, passo: u64, ciclo_antes: u64, ciclo: u64) -> Option<u64> {
        match self {
            Self::Passos(n) => (passo % n == 0).then_some(passo / n),
            Self::Ciclos(n) => (ciclo / n > ciclo_antes / n).then_some(ciclo / n),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segundos_viram_ciclos_do_barramento() {
        assert_eq!(segundos_em_ciclos("1s"), Ok(CICLOS_POR_SEGUNDO));
        assert_eq!(segundos_em_ciclos("12.5s"), Ok(423_360_000));
        assert_eq!(segundos_em_ciclos("0s"), Ok(0));
        assert!(segundos_em_ciclos("-1s").is_err());
        assert!(segundos_em_ciclos("abcs").is_err());
        assert!(segundos_em_ciclos("12").is_err());
    }

    #[test]
    fn instante_distingue_passo_de_segundo() {
        assert_eq!(instante_de("1000"), Ok(Instante::Passo(1000)));
        assert_eq!(
            instante_de("2s"),
            Ok(Instante::Ciclo(2 * CICLOS_POR_SEGUNDO))
        );
        assert!(instante_de("2.5").is_err());
    }

    #[test]
    fn spec_em_segundos_vira_ciclos_com_duracao_padrao() {
        assert_eq!(
            spec_em_ciclos("start@1s"),
            Ok(format!("start@{}:{}", CICLOS_POR_SEGUNDO, 3_386_880))
        );
        assert_eq!(
            spec_em_ciclos("cross@2s:0.5s"),
            Ok(format!("cross@{}:{}", 2 * CICLOS_POR_SEGUNDO, 16_934_400))
        );
        assert!(spec_em_ciclos("cross@2s:100").is_err());
        assert!(spec_em_ciclos("cross@2s:0s").is_err());
    }

    #[test]
    fn roteiro_misto_combina_passos_e_ciclos() {
        let presses = vec!["start@100:10".to_string(), "cross@1s:1s".to_string()];
        let r = Roteiro::monta(&presses, &[]).expect("roteiro valido");
        let solto = psx_core::pad_script::RELEASED;
        assert_eq!(r.buttons_at(0, 0), solto);
        assert_ne!(r.buttons_at(105, 0), solto, "start por passo");
        assert_eq!(r.buttons_at(0, CICLOS_POR_SEGUNDO - 1), solto);
        assert_ne!(
            r.buttons_at(0, CICLOS_POR_SEGUNDO),
            solto,
            "cross por tempo"
        );
        assert_eq!(r.buttons_at(0, 2 * CICLOS_POR_SEGUNDO), solto);
    }

    #[test]
    fn analog_em_segundos_dispara_uma_vez_ao_cruzar() {
        let r = Roteiro::monta(&["analog@1s".to_string()], &[]).expect("roteiro valido");
        assert!(r.uses_dualshock());
        let c = CICLOS_POR_SEGUNDO;
        assert!(!r.analog_press_at(5, c - 10, c - 3));
        assert!(r.analog_press_at(6, c - 3, c + 2));
        assert!(!r.analog_press_at(7, c + 2, c + 5));
    }

    #[test]
    fn stick_em_segundos_usa_ciclos() {
        let r = Roteiro::monta(&[], &["left:0,128@1s:1s".to_string()]).expect("roteiro valido");
        assert_eq!(r.sticks_at(0, 0), Sticks::CENTERED);
        assert_eq!(r.sticks_at(0, CICLOS_POR_SEGUNDO).left_x, 0);
    }

    #[test]
    fn cadencia_em_ciclos_marca_quando_cruza_o_periodo() {
        let c = Cadencia::de("0.5s").expect("cadencia valida");
        let meio = CICLOS_POR_SEGUNDO / 2;
        assert_eq!(c.marco(1, meio - 5, meio - 1), None);
        assert_eq!(c.marco(2, meio - 1, meio + 3), Some(1));
        assert_eq!(c.marco(3, meio + 3, meio + 6), None);
        assert_eq!(Cadencia::de("10").map(|c| c.marco(20, 0, 0)), Ok(Some(2)));
        assert!(Cadencia::de("0s").is_err());
    }
}
