mod support;

use psx_core::bus::{Bus, BusRead};
use support::asm;

const CD_BASE: u32 = 0x1F80_1800;
const MOTOR_ON: u8 = 1 << 1;
const ESPERA_COMANDO: u32 = 0x1_4000;

fn bus() -> Bus {
    let mut b = asm::bus_with_bios_empty();
    b.cdrom_mut().insert_disc();
    b
}

fn cd_read(bus: &Bus, offset: u32) -> u8 {
    bus.read8::<BusRead>(CD_BASE + offset)
}

fn cd_write(bus: &mut Bus, offset: u32, val: u8) {
    bus.write8::<BusRead>(CD_BASE + offset, val);
}

fn set_bank(bus: &mut Bus, b: u8) {
    cd_write(bus, 0, b);
}

fn hintsts(bus: &mut Bus) -> u8 {
    set_bank(bus, 1);
    let v = cd_read(bus, 3);
    set_bank(bus, 0);
    v & 0x7
}

fn ack(bus: &mut Bus) {
    set_bank(bus, 1);
    cd_write(bus, 3, 0x07);
    set_bank(bus, 0);
}

fn param(bus: &mut Bus, val: u8) {
    set_bank(bus, 0);
    cd_write(bus, 2, val);
}

fn manda(bus: &mut Bus, cmd: u8) {
    set_bank(bus, 0);
    cd_write(bus, 1, cmd);
    bus.tick_timers(ESPERA_COMANDO);
}

fn resultado(bus: &mut Bus) -> u8 {
    set_bank(bus, 0);
    cd_read(bus, 1)
}

/// Manda Stop e consome as duas respostas, deixando o motor parado.
fn para_o_motor(bus: &mut Bus) {
    manda(bus, 0x08);
    let _ = resultado(bus);
    ack(bus);
    bus.tick_timers(0x0E0_0000);
    let _ = resultado(bus);
    ack(bus);
}

// § MotorOn (06-cdrom.md L731-732): "Commands like Read, Seek, and Play are automatically
// starting the Motor when needed". O bit1 do stat e' Motor On; um jogo que le esse bit
// depois de um Stop seguido de Read tem de ver o motor ligado de novo.
#[test]
fn read_n_liga_o_motor_que_o_stop_desligou() {
    let mut bus = bus();
    para_o_motor(&mut bus);

    param(&mut bus, 0x02);
    param(&mut bus, 0x10);
    param(&mut bus, 0x00);
    manda(&mut bus, 0x02);
    let _ = resultado(&mut bus);
    ack(&mut bus);

    manda(&mut bus, 0x06);
    let stat = resultado(&mut bus);

    assert_eq!(
        stat & MOTOR_ON,
        MOTOR_ON,
        "ReadN depois de Stop tem de religar o motor sozinho (L731-732); sem isso o bit1 do \
         stat fica zerado pra sempre e um jogo que checa 'motor ligado' desiste"
    );
}

// § idem L731-732, agora com Seek.
#[test]
fn seek_l_liga_o_motor_que_o_stop_desligou() {
    let mut bus = bus();
    para_o_motor(&mut bus);

    param(&mut bus, 0x02);
    param(&mut bus, 0x10);
    param(&mut bus, 0x00);
    manda(&mut bus, 0x02);
    let _ = resultado(&mut bus);
    ack(&mut bus);

    manda(&mut bus, 0x15);
    let stat = resultado(&mut bus);

    assert_eq!(
        stat & MOTOR_ON,
        MOTOR_ON,
        "SeekL depois de Stop tem de religar o motor sozinho (L731-732)"
    );
}

// § MotorOn (06-cdrom.md L727-729): "Activates the drive motor, works ONLY if the motor was
// off (otherwise fails with INT5(stat,20h); that error code would normally indicate 'wrong
// number of parameters', but means 'motor already on' in this case)".
#[test]
fn motor_on_com_motor_parado_liga_e_responde_int3() {
    let mut bus = bus();
    para_o_motor(&mut bus);

    manda(&mut bus, 0x07);
    let stat = resultado(&mut bus);

    assert_eq!(
        hintsts(&mut bus),
        3,
        "MotorOn com motor parado responde INT3"
    );
    assert_eq!(
        stat & MOTOR_ON,
        MOTOR_ON,
        "e o stat ja sai com o motor ligado"
    );
}

#[test]
fn motor_on_com_motor_ligado_falha_com_int5_e_codigo_20h() {
    let mut bus = bus();

    manda(&mut bus, 0x07);
    let stat = resultado(&mut bus);
    let erro = resultado(&mut bus);

    assert_eq!(
        hintsts(&mut bus),
        5,
        "MotorOn com o motor JA ligado falha com INT5 (L727-729)"
    );
    assert_eq!(
        stat & MOTOR_ON,
        MOTOR_ON,
        "o stat do erro continua mostrando o motor ligado"
    );
    assert_eq!(
        erro, 0x20,
        "o codigo de erro e' 20h — normalmente significa 'numero errado de parametros', mas \
         aqui significa 'motor ja ligado' (L727-729)"
    );
}
