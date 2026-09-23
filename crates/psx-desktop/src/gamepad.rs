use gilrs::ff::{BaseEffect, BaseEffectType, Effect, EffectBuilder};
use gilrs::{Axis, Button, GamepadId, Gilrs};
use psx_core::app::input_map::{Eixos, Entrada};
use psx_core::app::troca::forca_de_vibracao;
use psx_core::dualshock::Rumble;

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

/// O que os controles fisicos dizem num quadro: botoes (e eixos como direcional, para o
/// modo digital) e os dois analogicos crus, para o modo analogico do DualShock.
#[derive(Debug, Clone, Default)]
pub struct Leitura {
    pub entradas: Vec<Entrada>,
    pub eixos: Eixos,
}

pub struct Gamepads {
    gilrs: Option<Gilrs>,
    vibracao: Option<Effect>,
    ultima_vibracao: Rumble,
}

impl Gamepads {
    /// Falta de subsistema de joystick nao derruba o app: o teclado continua valendo.
    pub fn novo() -> Self {
        match Gilrs::new() {
            Ok(g) => Gamepads {
                gilrs: Some(g),
                vibracao: None,
                ultima_vibracao: Rumble::default(),
            },
            Err(e) => {
                eprintln!("controles desligados: {e}");
                Gamepads {
                    gilrs: None,
                    vibracao: None,
                    ultima_vibracao: Rumble::default(),
                }
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
    pub fn le(&mut self) -> Leitura {
        let Some(gilrs) = self.gilrs.as_mut() else {
            return Leitura::default();
        };
        while gilrs.next_event().is_some() {}

        let mut fora = Leitura::default();
        for (_, pad) in gilrs.gamepads() {
            for (botao, entrada) in BOTOES {
                if pad.is_pressed(botao) && !fora.entradas.contains(&entrada) {
                    fora.entradas.push(entrada);
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
                if !fora.entradas.contains(&entrada) {
                    fora.entradas.push(entrada);
                }
            }
            let deste = Eixos::do_controle(
                (pad.value(Axis::LeftStickX), pad.value(Axis::LeftStickY)),
                (pad.value(Axis::RightStickX), pad.value(Axis::RightStickY)),
            );
            fora.eixos = fora.eixos.une(&deste);
        }
        fora
    }

    /// Recria o efeito so quando o jogo muda os motores: o gilrs nao ajusta a magnitude
    /// de um efeito em andamento, e soltar o `Effect` para a vibracao.
    pub fn vibra(&mut self, rumble: Rumble) {
        if rumble == self.ultima_vibracao {
            return;
        }
        self.ultima_vibracao = rumble;
        self.vibracao = None;
        let Some(gilrs) = self.gilrs.as_mut() else {
            return;
        };
        let (forte, fraco) = forca_de_vibracao(rumble);
        if forte == 0 && fraco == 0 {
            return;
        }
        let ids: Vec<GamepadId> = gilrs
            .gamepads()
            .filter(|(_, pad)| pad.is_ff_supported())
            .map(|(id, _)| id)
            .collect();
        if ids.is_empty() {
            return;
        }
        let efeito = EffectBuilder::new()
            .add_effect(BaseEffect {
                kind: BaseEffectType::Strong { magnitude: forte },
                ..Default::default()
            })
            .add_effect(BaseEffect {
                kind: BaseEffectType::Weak { magnitude: fraco },
                ..Default::default()
            })
            .gamepads(&ids)
            .finish(gilrs);
        match efeito {
            Ok(e) => {
                if e.play().is_ok() {
                    self.vibracao = Some(e);
                }
            }
            Err(e) => eprintln!("vibracao indisponivel: {e}"),
        }
    }
}

impl Default for Gamepads {
    fn default() -> Self {
        Self::novo()
    }
}
