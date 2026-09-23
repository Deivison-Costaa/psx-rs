use psx_core::dualshock::Sticks;
use psx_core::sio::Sio;

const CTRL_PORTA_1: u16 = 0x1003;
const HIZ: u8 = 0xFF;

fn pad() -> Sio {
    let sio = Sio::new();
    sio.connect_digital_pad(true);
    sio
}

fn transfere(sio: &Sio, envio: &[u8]) -> (Vec<u8>, Vec<bool>) {
    sio.write_ctrl(CTRL_PORTA_1);
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
    transfere(sio, envio).0
}

fn entra_config(sio: &Sio) {
    respostas(sio, &[0x01, 0x43, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00]);
}

fn sai_config(sio: &Sio) {
    respostas(sio, &[0x01, 0x43, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
}

const LEITURA_ANALOGICA: [u8; 9] = [0x01, 0x42, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

#[test]
fn liga_em_modo_digital_com_id_5a41_e_cinco_bytes() {
    let sio = pad();
    let (r, acks) = transfere(&sio, &LEITURA_ANALOGICA);

    assert_eq!(&r[..5], &[HIZ, 0x41, 0x5A, 0xFF, 0xFF]);
    assert_eq!(
        &acks[..5],
        &[true, true, true, true, false],
        "o ultimo byte do pacote (5o no modo digital) nao pulsa /ACK"
    );
    assert_eq!(&r[5..], &[HIZ; 4], "depois do pacote a linha fica em HiZ");
    assert!(acks[5..].iter().all(|a| !a));
}

#[test]
fn modo_analogico_responde_5a73_botoes_e_quatro_eixos() {
    let sio = pad();
    sio.set_analog_mode(true);
    sio.set_buttons(!(1u16 << 14));
    sio.set_sticks(Sticks {
        right_x: 0x11,
        right_y: 0x22,
        left_x: 0x33,
        left_y: 0x44,
    });

    let (r, acks) = transfere(&sio, &LEITURA_ANALOGICA);

    assert_eq!(
        r,
        vec![HIZ, 0x73, 0x5A, 0xFF, 0xBF, 0x11, 0x22, 0x33, 0x44],
        "ordem do fio: RightX, RightY, LeftX, LeftY"
    );
    assert_eq!(acks, vec![true, true, true, true, true, true, true, true, false]);
}

#[test]
fn eixos_centrados_em_80h_por_padrao() {
    let sio = pad();
    sio.set_analog_mode(true);

    let r = respostas(&sio, &LEITURA_ANALOGICA);

    assert_eq!(&r[5..], &[0x80; 4]);
}

#[test]
fn l3_e_r3_so_aparecem_no_modo_analogico() {
    let sio = pad();
    sio.set_buttons(!((1u16 << 1) | (1u16 << 2)));

    let digital = respostas(&sio, &LEITURA_ANALOGICA);
    assert_eq!(digital[3], 0xFF, "no digital os bits 1-2 ficam em 1");

    sio.set_analog_mode(true);
    let analogico = respostas(&sio, &LEITURA_ANALOGICA);
    assert_eq!(analogico[3], 0xF9, "no analogico L3 (bit1) e R3 (bit2) valem");
}

#[test]
fn botao_analog_alterna_o_modo_quando_destravado() {
    let sio = pad();

    assert!(sio.press_analog_button());
    assert!(sio.analog_mode());
    assert_eq!(respostas(&sio, &LEITURA_ANALOGICA)[1], 0x73);

    assert!(sio.press_analog_button());
    assert!(!sio.analog_mode());
    assert_eq!(respostas(&sio, &LEITURA_ANALOGICA)[1], 0x41);
}

#[test]
fn comando_43h_no_modo_normal_responde_como_42h_e_entra_em_config() {
    let sio = pad();
    sio.set_buttons(!(1u16 << 3));

    let (r, acks) = transfere(&sio, &[0x01, 0x43, 0x00, 0x01, 0x00]);
    assert_eq!(r, vec![HIZ, 0x41, 0x5A, 0xF7, 0xFF]);
    assert_eq!(acks, vec![true, true, true, true, false]);

    let (r, acks) = transfere(&sio, &LEITURA_ANALOGICA);
    assert_eq!(
        r,
        vec![HIZ, 0xF3, 0x5A, 0xF7, 0xFF, 0x80, 0x80, 0x80, 0x80],
        "42h em config forca a resposta analogica mesmo com LED apagado"
    );
    assert_eq!(acks.iter().filter(|a| **a).count(), 8, "config: 9 bytes");
}

#[test]
fn comando_43h_com_xx_zero_nao_entra_em_config() {
    let sio = pad();

    respostas(&sio, &[0x01, 0x43, 0x00, 0x00, 0x00]);

    assert_eq!(respostas(&sio, &LEITURA_ANALOGICA)[1], 0x41);
}

#[test]
fn comando_43h_em_config_devolve_zeros_e_sai() {
    let sio = pad();
    entra_config(&sio);

    let (r, acks) = transfere(&sio, &[0x01, 0x43, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

    assert_eq!(r, vec![HIZ, 0xF3, 0x5A, 0, 0, 0, 0, 0, 0]);
    assert!(!acks[8]);
    assert_eq!(respostas(&sio, &LEITURA_ANALOGICA)[1], 0x41, "de volta ao normal");
}

#[test]
fn comando_44h_liga_o_analogico_e_trava_o_botao() {
    let sio = pad();
    entra_config(&sio);

    let r = respostas(&sio, &[0x01, 0x44, 0x00, 0x01, 0x03, 0x00, 0x00, 0x00, 0x00]);
    assert_eq!(r, vec![HIZ, 0xF3, 0x5A, 0, 0, 0, 0, 0, 0]);
    sai_config(&sio);

    assert!(sio.analog_mode());
    assert!(sio.analog_locked());
    assert_eq!(respostas(&sio, &LEITURA_ANALOGICA)[1], 0x73);
    assert!(!sio.press_analog_button(), "Key=03h trava o botao Analog");
    assert!(sio.analog_mode());
}

#[test]
fn comando_44h_key_usa_so_os_dois_bits_de_baixo() {
    let sio = pad();
    entra_config(&sio);
    respostas(&sio, &[0x01, 0x44, 0x00, 0x01, 0x07, 0x00, 0x00, 0x00, 0x00]);
    assert!(sio.analog_locked(), "07h AND 03h = 03h: trava");

    respostas(&sio, &[0x01, 0x44, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00]);
    assert!(!sio.analog_locked(), "02h destrava");
    assert!(!sio.analog_mode(), "Led=00h volta ao digital");
}

#[test]
fn comando_44h_ignora_led_fora_de_0_e_1() {
    let sio = pad();
    entra_config(&sio);

    respostas(&sio, &[0x01, 0x44, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00]);

    assert!(!sio.analog_mode());
}

#[test]
fn comando_45h_devolve_tipo_e_led() {
    let sio = pad();
    entra_config(&sio);

    let apagado = respostas(&sio, &[0x01, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    assert_eq!(apagado, vec![HIZ, 0xF3, 0x5A, 0x01, 0x02, 0x00, 0x02, 0x01, 0x00]);

    respostas(&sio, &[0x01, 0x44, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00]);
    let aceso = respostas(&sio, &[0x01, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    assert_eq!(aceso[5], 0x01, "Led=01h depois de 44h com Led=01h");
}

#[test]
fn comando_46h_tabela_por_atuador() {
    let sio = pad();
    entra_config(&sio);

    let zero = respostas(&sio, &[0x01, 0x46, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    assert_eq!(zero, vec![HIZ, 0xF3, 0x5A, 0x00, 0x00, 0x01, 0x02, 0x00, 0x0A]);
    let um = respostas(&sio, &[0x01, 0x46, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00]);
    assert_eq!(um, vec![HIZ, 0xF3, 0x5A, 0x00, 0x00, 0x01, 0x01, 0x01, 0x14]);
    let outro = respostas(&sio, &[0x01, 0x46, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00]);
    assert_eq!(outro, vec![HIZ, 0xF3, 0x5A, 0, 0, 0, 0, 0, 0]);
}

#[test]
fn comando_47h_constantes() {
    let sio = pad();
    entra_config(&sio);

    let r = respostas(&sio, &[0x01, 0x47, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

    assert_eq!(r, vec![HIZ, 0xF3, 0x5A, 0x00, 0x00, 0x02, 0x00, 0x01, 0x00]);
}

#[test]
fn comando_48h_ee_so_para_ii_0_e_1() {
    let sio = pad();
    entra_config(&sio);

    let um = respostas(&sio, &[0x01, 0x48, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00]);
    assert_eq!(um, vec![HIZ, 0xF3, 0x5A, 0, 0, 0, 0, 0x01, 0]);
    let dois = respostas(&sio, &[0x01, 0x48, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00]);
    assert_eq!(dois[7], 0x00);
}

#[test]
fn comando_4ch_tabela_b() {
    let sio = pad();
    entra_config(&sio);

    let zero = respostas(&sio, &[0x01, 0x4C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    assert_eq!(zero, vec![HIZ, 0xF3, 0x5A, 0, 0, 0, 0x04, 0, 0]);
    let um = respostas(&sio, &[0x01, 0x4C, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00]);
    assert_eq!(um[6], 0x07);
    let outro = respostas(&sio, &[0x01, 0x4C, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00]);
    assert_eq!(outro[6], 0x00);
}

#[test]
fn comandos_sem_uso_em_config_devolvem_zeros_com_ack() {
    let sio = pad();
    entra_config(&sio);

    for comando in [0x40, 0x41, 0x49, 0x4A, 0x4B, 0x4E, 0x4F] {
        let (r, acks) = transfere(&sio, &[0x01, comando, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(r, vec![HIZ, 0xF3, 0x5A, 0, 0, 0, 0, 0, 0], "comando {comando:02X}h");
        assert_eq!(acks.iter().filter(|a| **a).count(), 8);
    }
}

#[test]
fn comando_de_config_fora_do_modo_config_nao_e_reconhecido() {
    let sio = pad();

    let (r, acks) = transfere(&sio, &[0x01, 0x45, 0x00, 0x00, 0x00]);

    assert_eq!(r[1], 0x41, "o ID sai junto com o comando, antes de o pad decodifica-lo");
    assert!(!acks[1], "comando invalido: sem /ACK, a transferencia acaba");
    assert_eq!(&r[2..], &[HIZ; 3]);
}

#[test]
fn modo_analogico_sobrevive_a_troca_de_transferencia() {
    let sio = pad();
    sio.set_analog_mode(true);
    respostas(&sio, &[0x01, 0x42]);

    let r = respostas(&sio, &LEITURA_ANALOGICA);

    assert_eq!(r[1], 0x73, "/CS soltar no meio nao reseta o modo do controle");
}
