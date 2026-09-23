use serde::{Deserialize, Serialize};

pub const ADDRESS: u8 = 0x01;
pub const ID_DIGITAL: u8 = 0x41;
pub const ID_ANALOG: u8 = 0x73;
pub const ID_CONFIG: u8 = 0xF3;
pub const STICK_CENTER: u8 = 0x80;

const HIGH_Z: u8 = 0xFF;
const ID_HIGH: u8 = 0x5A;
const DIGITAL_LEN: u8 = 5;
const FULL_LEN: u8 = 9;
const FIRST_PAYLOAD: u8 = 3;
const RUMBLE_SLOTS: usize = 6;
const UNMAPPED: u8 = 0xFF;
const MAP_SMALL: u8 = 0x00;
const MAP_LARGE: u8 = 0x01;
const STICK_BUTTONS: u16 = (1 << 1) | (1 << 2);
const LOCK_KEY: u8 = 0x03;
const TYPE_ANALOG_PAD: u8 = 0x01;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sticks {
    pub right_x: u8,
    pub right_y: u8,
    pub left_x: u8,
    pub left_y: u8,
}

impl Sticks {
    pub const CENTERED: Sticks = Sticks {
        right_x: STICK_CENTER,
        right_y: STICK_CENTER,
        left_x: STICK_CENTER,
        left_y: STICK_CENTER,
    };

    fn byte(&self, index: u8) -> u8 {
        match index {
            0 => self.right_x,
            1 => self.right_y,
            2 => self.left_x,
            _ => self.left_y,
        }
    }
}

impl Default for Sticks {
    fn default() -> Self {
        Self::CENTERED
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rumble {
    pub small: bool,
    pub large: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum Command {
    Read,
    Config,
    SetMode,
    Status,
    ActuatorInfo,
    Constants,
    Unknown48,
    ModeInfo,
    RumbleMap,
    Zeros,
}

impl Command {
    fn decode(byte: u8, config_mode: bool) -> Option<Command> {
        match (byte, config_mode) {
            (0x42, _) => Some(Command::Read),
            (0x43, _) => Some(Command::Config),
            (0x44, true) => Some(Command::SetMode),
            (0x45, true) => Some(Command::Status),
            (0x46, true) => Some(Command::ActuatorInfo),
            (0x47, true) => Some(Command::Constants),
            (0x48, true) => Some(Command::Unknown48),
            (0x4C, true) => Some(Command::ModeInfo),
            (0x4D, true) => Some(Command::RumbleMap),
            (0x40..=0x4F, true) => Some(Command::Zeros),
            _ => None,
        }
    }
}

/// SCPH-1200 (DualShock): digital (41h) ao ligar, analogico (73h) pelo botao Analog ou pelo
/// comando 44h, e o modo de configuracao (F3h) que destrava os dois motores.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualShock {
    buttons: u16,
    sticks: Sticks,
    analog: bool,
    locked: bool,
    config_mode: bool,
    configured: bool,
    id_high_cleared: bool,
    rumble_map: [u8; RUMBLE_SLOTS],
    rumble: Rumble,
    step: u8,
    command: Option<Command>,
    in_config: bool,
    in_analog: bool,
    param: u8,
    legacy_xx: u8,
}

impl Default for DualShock {
    fn default() -> Self {
        Self::new()
    }
}

impl DualShock {
    pub fn new() -> Self {
        DualShock {
            buttons: 0xFFFF,
            sticks: Sticks::CENTERED,
            analog: false,
            locked: false,
            config_mode: false,
            configured: false,
            id_high_cleared: false,
            rumble_map: [UNMAPPED; RUMBLE_SLOTS],
            rumble: Rumble::default(),
            step: 0,
            command: None,
            in_config: false,
            in_analog: false,
            param: 0,
            legacy_xx: 0,
        }
    }

    pub fn buttons(&self) -> u16 {
        self.buttons
    }

    pub fn set_buttons(&mut self, buttons: u16) {
        self.buttons = buttons;
    }

    pub fn sticks(&self) -> Sticks {
        self.sticks
    }

    pub fn set_sticks(&mut self, sticks: Sticks) {
        self.sticks = sticks;
    }

    pub fn analog(&self) -> bool {
        self.analog
    }

    pub fn locked(&self) -> bool {
        self.locked
    }

    pub fn config_mode(&self) -> bool {
        self.config_mode
    }

    pub fn rumble(&self) -> Rumble {
        self.rumble
    }

    pub fn set_analog(&mut self, analog: bool) {
        self.analog = analog;
    }

    /// Botao Analog fisico. Travado pelo jogo (44h com Key=3) nao faz nada; solto, troca o
    /// modo, para e trava os motores, e — se o jogo ja usou comandos de configuracao — troca
    /// o 5Ah da resposta por 00h para o jogo perceber a troca.
    pub fn press_analog_button(&mut self) -> bool {
        if self.locked {
            return false;
        }
        self.analog = !self.analog;
        self.rumble_map = [UNMAPPED; RUMBLE_SLOTS];
        self.rumble = Rumble::default();
        if self.configured {
            self.id_high_cleared = true;
        }
        true
    }

    pub fn begin(&mut self) {
        self.step = 0;
        self.command = None;
        self.param = 0;
        self.legacy_xx = 0;
    }

    pub fn exchange(&mut self, rx: u8) -> (u8, bool) {
        let step = self.step;
        self.step = self.step.saturating_add(1);
        match step {
            0 => (HIGH_Z, rx == ADDRESS),
            1 => self.command_byte(rx),
            _ => self.payload_byte(step, rx),
        }
    }

    fn id_low(&self) -> u8 {
        if self.config_mode {
            ID_CONFIG
        } else if self.analog {
            ID_ANALOG
        } else {
            ID_DIGITAL
        }
    }

    fn command_byte(&mut self, rx: u8) -> (u8, bool) {
        let id = self.id_low();
        self.in_config = self.config_mode;
        self.in_analog = self.analog;
        self.command = Command::decode(rx, self.config_mode);
        (id, self.command.is_some())
    }

    fn transfer_len(&self) -> u8 {
        if self.in_config || self.in_analog {
            FULL_LEN
        } else {
            DIGITAL_LEN
        }
    }

    fn payload_byte(&mut self, step: u8, rx: u8) -> (u8, bool) {
        let Some(command) = self.command else {
            return (HIGH_Z, false);
        };
        let len = self.transfer_len();
        if step >= len {
            return (HIGH_Z, false);
        }
        let reply = if step == 2 {
            self.id_high()
        } else {
            self.reply(command, step - FIRST_PAYLOAD)
        };
        if step >= FIRST_PAYLOAD {
            self.receive(command, step - FIRST_PAYLOAD, rx);
        }
        (reply, step + 1 < len)
    }

    fn id_high(&self) -> u8 {
        if self.id_high_cleared && !self.in_config {
            0x00
        } else {
            ID_HIGH
        }
    }

    fn pad_data(&self, index: u8) -> u8 {
        let forced_analog = self.in_config || self.in_analog;
        let buttons = if forced_analog {
            self.buttons
        } else {
            self.buttons | STICK_BUTTONS
        };
        match index {
            0 => buttons as u8,
            1 => (buttons >> 8) as u8,
            n => self.sticks.byte(n - 2),
        }
    }

    fn reply(&self, command: Command, index: u8) -> u8 {
        match command {
            Command::Read => self.pad_data(index),
            Command::Config if !self.in_config => self.pad_data(index),
            Command::Status => {
                [TYPE_ANALOG_PAD, 0x02, self.analog as u8, 0x02, 0x01, 0x00][index as usize]
            }
            Command::ActuatorInfo => match (self.param, index) {
                (0, 2..) => [0x01, 0x02, 0x00, 0x0A][index as usize - 2],
                (1, 2..) => [0x01, 0x01, 0x01, 0x14][index as usize - 2],
                _ => 0x00,
            },
            Command::Constants => [0x00, 0x00, 0x02, 0x00, 0x01, 0x00][index as usize],
            Command::Unknown48 => u8::from(index == 4 && self.param <= 1),
            Command::ModeInfo => match (self.param, index) {
                (0, 3) => 0x04,
                (1, 3) => 0x07,
                _ => 0x00,
            },
            Command::RumbleMap => self.rumble_map[index as usize],
            _ => 0x00,
        }
    }

    fn receive(&mut self, command: Command, index: u8, rx: u8) {
        if index == 0 {
            self.param = rx;
        }
        match command {
            Command::Read => self.drive_motors(index, rx),
            Command::Config if index == 0 => self.set_config_mode(rx == 0x01),
            Command::SetMode => self.set_mode(index, rx),
            Command::RumbleMap => self.rumble_map[index as usize] = rx,
            _ => {}
        }
    }

    fn set_config_mode(&mut self, enter: bool) {
        if enter {
            self.configured = true;
            self.id_high_cleared = false;
        }
        self.config_mode = enter;
    }

    fn set_mode(&mut self, index: u8, rx: u8) {
        match (index, rx) {
            (0, 0x00) => self.analog = false,
            (0, 0x01) => self.analog = true,
            (1, key) => self.locked = key & LOCK_KEY == LOCK_KEY,
            _ => {}
        }
    }

    fn drive_motors(&mut self, index: u8, rx: u8) {
        if !self.configured {
            self.drive_legacy_motor(index, rx);
            return;
        }
        match self.rumble_map[index as usize] {
            MAP_SMALL => self.rumble.small = rx & 1 != 0,
            MAP_LARGE => self.rumble.large = rx,
            _ => {}
        }
    }

    fn drive_legacy_motor(&mut self, index: u8, rx: u8) {
        match index {
            0 => self.legacy_xx = rx,
            1 => self.rumble.small = self.legacy_xx & 0xC0 == 0x40 && rx & 1 != 0,
            _ => {}
        }
    }
}
