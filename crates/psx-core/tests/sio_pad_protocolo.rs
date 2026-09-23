use psx_core::sio::Sio;

const CTRL_PORTA_1: u16 = 0x1003;
const CTRL_PORTA_2: u16 = 0x3003;

fn porta(ctrl: u16) -> Sio {
    let sio = Sio::new();
    sio.connect_digital_pad(true);
    sio.connect_memory_card(true);
    sio.write_ctrl(ctrl);
    sio
}

fn troca(sio: &Sio, byte: u8) -> (u8, bool) {
    sio.write_tx(byte);
    sio.deliver_ack();
    let ack = sio.take_irq7();
    sio.end_ack_pulse();
    sio.write_ctrl(sio.read_ctrl() | (1 << 4));
    (sio.read_rx(), ack)
}

// 10-controllers-memcards.md, "Address byte (01h) being sent": "Once the last byte of the
// packet is transferred, the device shall no longer pulse /ACK". O pad digital para no 5o byte.
#[test]
fn pad_digital_nao_pulsa_ack_depois_do_ultimo_byte() {
    let sio = porta(CTRL_PORTA_1);
    sio.set_buttons(!(1u16 << 14));

    assert_eq!(troca(&sio, 0x01), (0xFF, true));
    assert_eq!(troca(&sio, 0x42), (0x41, true));
    assert_eq!(troca(&sio, 0x00), (0x5A, true));
    assert_eq!(troca(&sio, 0x00), (0xFF, true), "swlo, e ainda ha swhi");
    assert_eq!(
        troca(&sio, 0x00),
        (0xBF, false),
        "swhi e o ultimo byte do pad digital: sem /ACK depois dele"
    );
}

// "Controllers - Configuration Commands": 43h e comando de pad com modo de configuracao
// (analogico); o pad digital so conhece a leitura 42h e nao reconhece outro comando.
#[test]
fn pad_digital_nao_reconhece_comando_de_configuracao() {
    let sio = porta(CTRL_PORTA_1);

    assert_eq!(troca(&sio, 0x01), (0xFF, true));
    let (rx, ack) = troca(&sio, 0x43);
    assert!(
        !ack,
        "sem /ACK o driver conclui que nao ha modo de configuracao"
    );
    assert_eq!(rx, 0xFF, "a linha de dados fica ociosa");
}

// 17-sio.md SIO_CTRL.13: seleciona qual /CS e baixado. So ha controle e cartao na porta 1.
#[test]
fn porta_2_sem_controle_nao_responde() {
    let sio = porta(CTRL_PORTA_2);

    assert_eq!(
        troca(&sio, 0x01),
        (0xFF, false),
        "com CTRL.13=1 o /CS da porta 2 e o selecionado, e ela esta vazia"
    );
    assert_eq!(troca(&sio, 0x42), (0xFF, false));
}

#[test]
fn porta_2_sem_memory_card_nao_responde() {
    let sio = porta(CTRL_PORTA_2);

    assert_eq!(
        troca(&sio, 0x81),
        (0xFF, false),
        "o cartao montado esta no slot 1, nao no 2"
    );
}

#[test]
fn porta_1_continua_com_memory_card() {
    let sio = porta(CTRL_PORTA_1);

    assert!(troca(&sio, 0x81).1, "slot 1 reconhece o endereco 81h");
}
