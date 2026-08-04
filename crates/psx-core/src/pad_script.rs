pub const RELEASED: u16 = 0xFFFF;
pub const DEFAULT_PRESS_STEPS: u64 = 2_000_000;

const BUTTONS: [&str; 16] = [
    "select", "l3", "r3", "start", "up", "right", "down", "left", "l2", "r2", "l1", "r1",
    "triangle", "circle", "cross", "square",
];

pub fn button_name(bit: u8) -> Option<&'static str> {
    BUTTONS.get(bit as usize).copied()
}

pub fn button_bit(name: &str) -> Option<u8> {
    let lower = name.to_ascii_lowercase();
    BUTTONS.iter().position(|b| *b == lower).map(|i| i as u8)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Press {
    start: u64,
    end: u64,
    bit: u8,
}

#[derive(Debug, Clone, Default)]
pub struct PadScript {
    presses: Vec<Press>,
}

impl PadScript {
    pub fn parse(specs: &[String]) -> Result<Self, String> {
        let mut presses = Vec::with_capacity(specs.len());
        for spec in specs {
            presses.push(parse_one(spec)?);
        }
        Ok(Self { presses })
    }

    pub fn is_empty(&self) -> bool {
        self.presses.is_empty()
    }

    pub fn buttons_at(&self, step: u64) -> u16 {
        let mut state = RELEASED;
        for press in &self.presses {
            if step >= press.start && step < press.end {
                state &= !(1u16 << press.bit);
            }
        }
        state
    }
}

fn parse_one(spec: &str) -> Result<Press, String> {
    let erro = |motivo: &str| format!("'{spec}': {motivo}");

    let (nome, resto) = spec
        .split_once('@')
        .ok_or_else(|| erro("falta '@passo'; use BOTAO@PASSO[:DURACAO]"))?;
    let bit = button_bit(nome).ok_or_else(|| erro("botao desconhecido"))?;

    let (passo_txt, duracao_txt) = match resto.split_once(':') {
        Some((p, d)) => (p, Some(d)),
        None => (resto, None),
    };

    let start: u64 = passo_txt
        .parse()
        .map_err(|_| erro("o passo tem de ser um inteiro decimal"))?;

    let duracao = match duracao_txt {
        None => DEFAULT_PRESS_STEPS,
        Some(d) => d
            .parse::<u64>()
            .map_err(|_| erro("a duracao tem de ser um inteiro decimal"))?,
    };
    if duracao == 0 {
        return Err(erro("a duracao tem de ser pelo menos 1 passo"));
    }

    Ok(Press {
        start,
        end: start.saturating_add(duracao),
        bit,
    })
}
