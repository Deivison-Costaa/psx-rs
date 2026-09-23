use crate::dualshock::Sticks;

pub const RELEASED: u16 = 0xFFFF;
pub const DEFAULT_PRESS_STEPS: u64 = 2_000_000;
pub const ANALOG_BUTTON: &str = "analog";

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Tilt {
    start: u64,
    end: u64,
    right: bool,
    x: u8,
    y: u8,
}

#[derive(Debug, Clone, Default)]
pub struct PadScript {
    presses: Vec<Press>,
    tilts: Vec<Tilt>,
    analog_presses: Vec<u64>,
}

impl PadScript {
    /// `analog@PASSO` aperta o botao Analog (alterna o modo) no passo dado.
    pub fn parse(specs: &[String]) -> Result<Self, String> {
        let mut presses = Vec::with_capacity(specs.len());
        let mut analog_presses = Vec::new();
        for spec in specs {
            match spec.split_once('@') {
                Some((nome, resto)) if nome.eq_ignore_ascii_case(ANALOG_BUTTON) => {
                    analog_presses.push(parse_window(spec, resto)?.0);
                }
                _ => presses.push(parse_one(spec)?),
            }
        }
        Ok(Self {
            presses,
            tilts: Vec::new(),
            analog_presses,
        })
    }

    /// Especificacoes `left|right:X,Y@PASSO[:DURACAO]`, com X e Y de 0 a 255 (80h = centro).
    pub fn with_sticks(self, specs: &[String]) -> Result<Self, String> {
        let mut tilts = self.tilts;
        for spec in specs {
            tilts.push(parse_tilt(spec)?);
        }
        Ok(Self { tilts, ..self })
    }

    pub fn is_empty(&self) -> bool {
        self.presses.is_empty() && self.tilts.is_empty() && self.analog_presses.is_empty()
    }

    pub fn sticks_at(&self, step: u64) -> Sticks {
        let mut sticks = Sticks::CENTERED;
        for tilt in &self.tilts {
            if step < tilt.start || step >= tilt.end {
                continue;
            }
            if tilt.right {
                sticks.right_x = tilt.x;
                sticks.right_y = tilt.y;
            } else {
                sticks.left_x = tilt.x;
                sticks.left_y = tilt.y;
            }
        }
        sticks
    }

    pub fn analog_press_at(&self, step: u64) -> bool {
        self.analog_presses.contains(&step)
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
    let (start, end) = parse_window(spec, resto)?;
    Ok(Press { start, end, bit })
}

fn parse_tilt(spec: &str) -> Result<Tilt, String> {
    let erro = |motivo: &str| format!("'{spec}': {motivo}");
    let formato = "use left|right:X,Y@PASSO[:DURACAO]";

    let (alvo, resto) = spec
        .split_once('@')
        .ok_or_else(|| erro(&format!("falta '@passo'; {formato}")))?;
    let (lado, eixos) = alvo
        .split_once(':')
        .ok_or_else(|| erro(&format!("falta ':X,Y'; {formato}")))?;
    let right = match lado.to_ascii_lowercase().as_str() {
        "left" => false,
        "right" => true,
        _ => return Err(erro("o analogico e 'left' ou 'right'")),
    };
    let (x_txt, y_txt) = eixos
        .split_once(',')
        .ok_or_else(|| erro(&format!("falta ',' entre X e Y; {formato}")))?;
    let x = parse_axis(x_txt).ok_or_else(|| erro("X tem de ser 0..255 (decimal ou 0x..)"))?;
    let y = parse_axis(y_txt).ok_or_else(|| erro("Y tem de ser 0..255 (decimal ou 0x..)"))?;
    let (start, end) = parse_window(spec, resto)?;
    Ok(Tilt {
        start,
        end,
        right,
        x,
        y,
    })
}

fn parse_axis(texto: &str) -> Option<u8> {
    let texto = texto.trim();
    match texto.strip_prefix("0x").or_else(|| texto.strip_prefix("0X")) {
        Some(hex) => u8::from_str_radix(hex, 16).ok(),
        None => texto.parse().ok(),
    }
}

fn parse_window(spec: &str, resto: &str) -> Result<(u64, u64), String> {
    let erro = |motivo: &str| format!("'{spec}': {motivo}");

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

    Ok((start, start.saturating_add(duracao)))
}
