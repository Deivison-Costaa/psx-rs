use gilrs::{Axis, Button, Gilrs};
use psx_core::app::input_map::Entrada;

/// Abaixo disso o analogico e considerado centrado. Controle usado solta valor de
/// repouso longe de zero; sem zona morta o jogo anda sozinho.
const ZONA_MORTA: f32 = 0.5;

const BOTOES: [(Button, Entrada); 17] = [
    (Button::South, Entrada::Sul),
    (Button::East, Entrada::Leste),
    (Button::North, Entrada::Norte),
    (Button::West, Entrada::Oeste),
    (Button::LeftTrigger, Entrada::L1),
    (Button::LeftTrigger2, Entrada::L2),
    (Button::RightTrigger, Entrada::R1),
    (Button::RightTrigger2, Entrada::R2),
    (Button::LeftThumb, Entrada::L3),
    (Button::RightThumb, Entrada::R3),
    (Button::Select, Entrada::Select),
    (Button::Start, Entrada::Start),
    (Button::Mode, Entrada::Modo),
    (Button::DPadUp, Entrada::DpadCima),
    (Button::DPadDown, Entrada::DpadBaixo),
    (Button::DPadLeft, Entrada::DpadEsquerda),
    (Button::DPadRight, Entrada::DpadDireita),
];

const EIXOS: [(Axis, u8); 2] = [(Axis::LeftStickX, 0), (Axis::LeftStickY, 1)];

pub struct Gamepads {
    gilrs: Option<Gilrs>,
}

impl Gamepads {
    /// Falta de subsistema de joystick nao derruba o app: o teclado continua valendo.
    pub fn novo() -> Self {
        match Gilrs::new() {
            Ok(g) => Gamepads { gilrs: Some(g) },
            Err(e) => {
                eprintln!("controles desligados: {e}");
                Gamepads { gilrs: None }
            }
        }
    }

    pub fn nomes(&self) -> Vec<String> {
        let Some(gilrs) = &self.gilrs else {
            return Vec::new();
        };
        gilrs
            .gamepads()
            .map(|(_, pad)| pad.name().to_string())
            .collect()
    }

    /// Le o estado de TODOS os controles conectados de uma vez: dois controles no mesmo
    /// slot 1 e o que um jogador com um pad e um arcade stick espera.
    pub fn pressionados(&mut self) -> Vec<Entrada> {
        let Some(gilrs) = self.gilrs.as_mut() else {
            return Vec::new();
        };
        while gilrs.next_event().is_some() {}

        let mut fora = Vec::new();
        for (_, pad) in gilrs.gamepads() {
            for (botao, entrada) in BOTOES {
                if pad.is_pressed(botao) && !fora.contains(&entrada) {
                    fora.push(entrada);
                }
            }
            for (eixo, numero) in EIXOS {
                let valor = pad.value(eixo);
                let entrada = if valor <= -ZONA_MORTA {
                    Entrada::EixoNegativo(numero)
                } else if valor >= ZONA_MORTA {
                    Entrada::EixoPositivo(numero)
                } else {
                    continue;
                };
                if !fora.contains(&entrada) {
                    fora.push(entrada);
                }
            }
        }
        fora
    }
}

impl Default for Gamepads {
    fn default() -> Self {
        Self::novo()
    }
}
