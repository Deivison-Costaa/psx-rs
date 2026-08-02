use psx_core::sio::Sio;

const CTRL_BIOS: u16 = 0x1003;
const ENDERECO_CONTROLE: u8 = 0x01;
const ENDERECO_MEMORY_CARD: u8 = 0x81;
const ENDERECO_YAROZE: u8 = 0x21;
const STAT_IRQ: u32 = 1 << 9;
const STAT_ACK: u32 = 0x80;

fn porta(pad: bool) -> Sio {
    let sio = Sio::new();
    sio.connect_digital_pad(pad);
    sio.write_ctrl(CTRL_BIOS);
    sio
}

#[test]
fn sem_periferico_o_endereco_01h_nao_produz_ack() {
    let sio = porta(false);

    sio.write_tx(ENDERECO_CONTROLE);

    assert!(
        !sio.take_irq7(),
        "sem periferico conectado nao ha /ACK, logo nao ha IRQ7"
    );
    assert_eq!(
        sio.read_stat() & STAT_IRQ,
        0,
        "JOY_STAT.9 so acende quando o periferico enderecado responde"
    );
    assert_eq!(
        sio.read_stat() & STAT_ACK,
        0,
        "JOY_STAT.7 espelha a linha /ACK, que ninguem puxa quando o slot esta vazio"
    );
    assert_eq!(sio.read_rx(), 0xFF, "linha de dados ociosa devolve 0xFF");
}

#[test]
fn pad_conectado_responde_ao_endereco_01h() {
    let sio = porta(true);

    sio.write_tx(ENDERECO_CONTROLE);

    assert!(sio.take_irq7(), "o controle enderecado puxa /ACK");
    assert_eq!(
        sio.read_stat() & STAT_IRQ,
        STAT_IRQ,
        "JOY_STAT.9 acende no byte reconhecido"
    );
}

#[test]
fn endereco_81h_nao_produz_ack_sem_memory_card() {
    let sio = porta(true);

    sio.write_tx(ENDERECO_MEMORY_CARD);

    assert!(
        !sio.take_irq7(),
        "0x81 endereca o memory card; o controle nao responde por ele"
    );
    assert_eq!(
        sio.read_rx(),
        0xFF,
        "slot de card vazio deixa a linha ociosa"
    );

    sio.write_tx(0x52);

    assert!(
        !sio.take_irq7(),
        "sem card, nenhum byte seguinte da transferencia e reconhecido"
    );
    assert_eq!(sio.read_rx(), 0xFF);
}

#[test]
fn endereco_desconhecido_nao_produz_ack() {
    let sio = porta(true);

    sio.write_tx(ENDERECO_YAROZE);

    assert!(
        !sio.take_irq7(),
        "so o dispositivo enderecado responde; 0x21 nao e o controle digital"
    );
    assert_eq!(sio.read_rx(), 0xFF);
}

#[test]
fn soltar_o_cs_relatcha_o_endereco_da_transferencia_seguinte() {
    let sio = porta(true);

    sio.write_tx(ENDERECO_MEMORY_CARD);
    assert!(!sio.take_irq7());
    let _ = sio.read_rx();

    sio.write_ctrl(0x0000);
    sio.write_ctrl(CTRL_BIOS);
    sio.write_tx(ENDERECO_CONTROLE);

    assert!(
        sio.take_irq7(),
        "cada assercao de /CS comeca uma transferencia nova, com endereco novo"
    );
}

#[test]
fn o_pad_so_responde_id_e_botoes_depois_do_endereco_01h() {
    let sio = porta(true);

    sio.write_tx(ENDERECO_MEMORY_CARD);
    let _ = sio.read_rx();
    sio.write_tx(0x42);

    assert_eq!(
        sio.read_rx(),
        0xFF,
        "o controle nao pode devolver 0x41 numa transferencia enderecada ao card"
    );
}
