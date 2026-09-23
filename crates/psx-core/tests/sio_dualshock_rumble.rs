use psx_core::dualshock::Rumble;
use psx_core::sio::Sio;

const CTRL_PORTA_1: u16 = 0x1003;
const HIZ: u8 = 0xFF;
const PARADO: Rumble = Rumble {
    small: false,
    large: 0,
};

fn pad() -> Sio {
    let sio = Sio::new();
    sio.connect_dualshock(true);
    sio
}

fn transfere_na(sio: &Sio, ctrl: u16, envio: &[u8]) -> (Vec<u8>, Vec<bool>) {
    sio.write_ctrl(ctrl);
    let mut respostas = Vec::new();
    let mut acks = Vec::new();
    for &byte in envio {
        sio.write_tx(byte);
        acks.push(sio.take_ack_request());
        respostas.push(sio.read_rx());
    }
    sio.write_ctrl(0x0000);
    (respostas, acks)
}

fn respostas(sio: &Sio, envio: &[u8]) -> Vec<u8> {
    transfere_na(sio, CTRL_PORTA_1, envio).0
}

fn entra_config(sio: &Sio) {
    respostas(sio, &[0x01, 0x43, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00]);
}

fn sai_config(sio: &Sio) {
    respostas(sio, &[0x01, 0x43, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
}

fn destrava_dois_motores(sio: &Sio) {
    entra_config(sio);
    respostas(sio, &[0x01, 0x4D, 0x00, 0x00, 0x01, 0xFF, 0xFF, 0xFF, 0xFF]);
    sai_config(sio);
}

#[test]
fn motores_comecam_parados() {
    assert_eq!(pad().rumble(), PARADO);
}

#[test]
fn metodo_antigo_liga_o_motor_pequeno_com_xx_40h_e_yy_impar() {
    let sio = pad();

    respostas(&sio, &[0x01, 0x42, 0x00, 0x40, 0x01]);
    assert_eq!(
        sio.rumble(),
        Rumble {
            small: true,
            large: 0
        }
    );

    respostas(&sio, &[0x01, 0x42, 0x00, 0x00, 0x00]);
    assert_eq!(sio.rumble(), PARADO, "yyxx=0000h desliga");
}

#[test]
fn metodo_antigo_exige_bit7_zero_e_bit6_um_em_xx() {
    let sio = pad();

    respostas(&sio, &[0x01, 0x42, 0x00, 0xC0, 0x01]);
    assert!(!sio.rumble().small, "xx=C0h tem bit7=1");
    respostas(&sio, &[0x01, 0x42, 0x00, 0x7F, 0x02]);
    assert!(!sio.rumble().small, "yy par nao liga");
    respostas(&sio, &[0x01, 0x42, 0x00, 0x7F, 0xFF]);
    assert!(sio.rumble().small);
}

#[test]
fn comandos_de_config_desligam_o_metodo_antigo() {
    let sio = pad();
    entra_config(&sio);
    sai_config(&sio);

    respostas(&sio, &[0x01, 0x42, 0x00, 0x40, 0x01]);

    assert_eq!(sio.rumble(), PARADO);
}

#[test]
fn comando_4dh_devolve_o_mapa_antigo_e_grava_o_novo() {
    let sio = pad();
    entra_config(&sio);

    let (r, acks) = transfere_na(
        &sio,
        CTRL_PORTA_1,
        &[0x01, 0x4D, 0x00, 0x00, 0x01, 0xFF, 0xFF, 0xFF, 0xFF],
    );
    assert_eq!(r, vec![HIZ, 0xF3, 0x5A, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
    assert_eq!(acks.iter().filter(|a| **a).count(), 8);

    let r = respostas(
        &sio,
        &[0x01, 0x4D, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
    );
    assert_eq!(r, vec![HIZ, 0xF3, 0x5A, 0x00, 0x01, 0xFF, 0xFF, 0xFF, 0xFF]);
}

#[test]
fn metodo_novo_m2_no_bit0_de_xx_e_m1_em_yy() {
    let sio = pad();
    destrava_dois_motores(&sio);

    respostas(&sio, &[0x01, 0x42, 0x00, 0x01, 0xC0]);
    assert_eq!(
        sio.rumble(),
        Rumble {
            small: true,
            large: 0xC0
        }
    );

    respostas(&sio, &[0x01, 0x42, 0x00, 0x00, 0x00]);
    assert_eq!(sio.rumble(), PARADO);
}

#[test]
fn metodo_novo_respeita_motores_trocados() {
    let sio = pad();
    entra_config(&sio);
    respostas(
        &sio,
        &[0x01, 0x4D, 0x00, 0x01, 0x00, 0xFF, 0xFF, 0xFF, 0xFF],
    );
    sai_config(&sio);

    respostas(&sio, &[0x01, 0x42, 0x00, 0x90, 0x01]);

    assert_eq!(
        sio.rumble(),
        Rumble {
            small: true,
            large: 0x90
        }
    );
}

#[test]
fn comando_42h_em_config_tambem_aciona_os_motores() {
    let sio = pad();
    entra_config(&sio);
    respostas(
        &sio,
        &[0x01, 0x4D, 0x00, 0x00, 0x01, 0xFF, 0xFF, 0xFF, 0xFF],
    );

    respostas(
        &sio,
        &[0x01, 0x42, 0x00, 0x01, 0x70, 0x00, 0x00, 0x00, 0x00],
    );

    assert_eq!(
        sio.rumble(),
        Rumble {
            small: true,
            large: 0x70
        }
    );
}

#[test]
fn comando_43h_no_modo_normal_nao_leva_parametros_de_motor() {
    let sio = pad();
    destrava_dois_motores(&sio);

    respostas(&sio, &[0x01, 0x43, 0x00, 0x00, 0xFF]);

    assert_eq!(sio.rumble(), PARADO);
}

#[test]
fn botao_analog_para_e_trava_os_motores() {
    let sio = pad();
    destrava_dois_motores(&sio);
    respostas(&sio, &[0x01, 0x42, 0x00, 0x01, 0xFF]);
    assert_ne!(sio.rumble(), PARADO, "pre-condicao: motores girando");

    assert!(sio.press_analog_button());

    assert_eq!(sio.rumble(), PARADO);
    respostas(
        &sio,
        &[0x01, 0x42, 0x00, 0x01, 0xFF, 0x00, 0x00, 0x00, 0x00],
    );
    assert_eq!(
        sio.rumble(),
        PARADO,
        "o mapa voltou a FFh: nada liga os motores"
    );
}

#[test]
fn botao_analog_depois_de_config_troca_5ah_por_00h() {
    let sio = pad();
    entra_config(&sio);
    sai_config(&sio);

    assert!(sio.press_analog_button());

    let r = respostas(
        &sio,
        &[0x01, 0x42, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    );
    assert_eq!(&r[1..3], &[0x73, 0x00]);
}

#[test]
fn botao_analog_sem_config_mantem_5ah() {
    let sio = pad();

    sio.press_analog_button();

    assert_eq!(respostas(&sio, &[0x01, 0x42, 0x00])[2], 0x5A);
}
