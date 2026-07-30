use psx_core::sio::Sio;

#[test]
fn sio_new_retorna_stat_tx_ready() {
    let sio = Sio::new();
    let stat = sio.read_stat();
    assert!(
        stat & 0x1 != 0,
        "STAT.0 (TX FIFO Not Full) deve ser 1 apos new"
    );
    assert!(
        stat & 0x2 == 0,
        "STAT.1 (RX FIFO Not Empty) deve ser 0 apos new"
    );
    assert!(stat & 0x4 != 0, "STAT.2 (TX Idle) deve ser 1 apos new");
}

#[test]
fn pad_digital_responde_5a41_ffff_ao_comando_42h() {
    let sio = Sio::new();
    sio.connect_digital_pad(true);

    sio.write_ctrl(0x0002);
    assert_eq!(sio.read_ctrl() & 0x0002, 0x0002);

    sio.write_tx(0x01);
    assert!(
        sio.read_stat() & 0x80 != 0,
        "STAT.7 (DSR) deve ser 1 apos envio de byte"
    );
    let _ = sio.read_rx();

    sio.write_tx(0x42);
    assert!(
        sio.read_stat() & 0x02 != 0,
        "STAT.1 (RX FIFO Not Empty) deve ser 1"
    );
    assert_eq!(sio.read_rx(), 0x41, "ID low deve ser 0x41 (digital pad)");

    sio.write_tx(0x00);
    assert_eq!(sio.read_rx(), 0x5A, "ID high deve ser 0x5A");

    sio.write_tx(0x00);
    assert_eq!(sio.read_rx(), 0xFF, "buttons high = 0xFF (todos soltos)");

    sio.write_tx(0x00);
    assert_eq!(sio.read_rx(), 0xFF, "buttons low = 0xFF (todos soltos)");

    sio.write_ctrl(0x0000);
}

#[test]
fn sem_pad_digital_rx_fifo_retorna_ff() {
    let sio = Sio::new();
    sio.connect_digital_pad(false);

    sio.write_ctrl(0x0002);
    sio.write_tx(0x01);
    let _ = sio.read_rx();

    sio.write_tx(0x42);
    assert_eq!(sio.read_rx(), 0xFF, "sem pad, resposta deve ser 0xFF (HiZ)");

    sio.write_tx(0x00);
    assert_eq!(sio.read_rx(), 0xFF);

    sio.write_ctrl(0x0000);
}

#[test]
fn stat_bit1_reflete_rx_fifo() {
    let sio = Sio::new();
    sio.connect_digital_pad(true);

    assert!(
        sio.read_stat() & 0x02 == 0,
        "STAT.1 deve ser 0 com FIFO vazia"
    );

    sio.write_ctrl(0x0002);
    sio.write_tx(0x01);
    let _ = sio.read_rx();

    sio.write_tx(0x42);
    assert!(
        sio.read_stat() & 0x02 != 0,
        "STAT.1 deve ser 1 com dado no FIFO"
    );

    sio.read_rx();
    assert!(
        sio.read_stat() & 0x02 == 0,
        "STAT.1 deve ser 0 apos ler FIFO"
    );

    sio.write_ctrl(0x0000);
}

#[test]
fn ctrl_bit4_ack_limpa_stat_bit9() {
    let sio = Sio::new();
    sio.connect_digital_pad(true);

    sio.write_ctrl(0x0002 | (1 << 12));
    sio.write_tx(0x01);
    let _ = sio.read_rx();
    sio.write_tx(0x42);

    assert!(sio.read_stat() & 0x02 != 0);

    sio.write_ctrl(0x0002 | (1 << 12) | (1 << 4));
    assert!(
        sio.read_stat() & (1 << 9) == 0,
        "STAT.9 (IRQ) deve ser limpo por CTRL.4 (ack)"
    );

    sio.write_ctrl(0x0000);
}

#[test]
fn cs_desassertado_nao_transfere() {
    let sio = Sio::new();
    sio.connect_digital_pad(true);

    sio.write_tx(0x01);

    assert!(
        sio.read_stat() & 0x02 == 0,
        "sem /CS, STAT.1 deve permanecer 0 (sem transferencia)"
    );
    assert!(
        sio.read_stat() & 0x80 == 0,
        "sem /CS, STAT.7 (DSR) deve permanecer 0"
    );
}

#[test]
fn dtr_transicao_reseta_contagem_de_bytes() {
    let sio = Sio::new();
    sio.connect_digital_pad(true);

    sio.write_ctrl(0x0002);
    sio.write_tx(0x01);
    let _ = sio.read_rx();
    sio.write_tx(0x42);
    let _ = sio.read_rx();

    sio.write_ctrl(0x0000);
    sio.write_ctrl(0x0002);

    sio.write_tx(0x01);
    let _ = sio.read_rx();
    sio.write_tx(0x42);
    assert_eq!(
        sio.read_rx(),
        0x41,
        "apos reassert de /CS, byte 1 deve ser ID low 0x41"
    );

    sio.write_ctrl(0x0000);
}

#[test]
fn botoes_pressionados_aparecem_na_resposta_42h() {
    let sio = Sio::new();
    sio.connect_digital_pad(true);
    sio.set_buttons(!((1u16 << 3) | (1u16 << 14)));

    sio.write_ctrl(0x0002);
    sio.write_tx(0x01);
    let _ = sio.read_rx();

    sio.write_tx(0x42);
    assert_eq!(sio.read_rx(), 0x41, "ID low = 0x41");

    sio.write_tx(0x00);
    assert_eq!(sio.read_rx(), 0x5A, "ID high = 0x5A");

    sio.write_tx(0x00);
    assert_eq!(
        sio.read_rx(),
        0xBF,
        "buttons high: Start(bit3)=0, Cross(bit14)=0 → bit15-8 = 10111111 = 0xBF"
    );

    sio.write_tx(0x00);
    assert_eq!(
        sio.read_rx(),
        0xF7,
        "buttons low: Cross(bit14) no low, Start(bit3)=0 → bit7-0 = 11110111 = 0xF7"
    );

    sio.write_ctrl(0x0000);
}

#[test]
fn botoes_soltos_retornam_ff() {
    let sio = Sio::new();
    sio.connect_digital_pad(true);
    sio.set_buttons(0xFFFF);

    sio.write_ctrl(0x0002);
    sio.write_tx(0x01);
    let _ = sio.read_rx();

    sio.write_tx(0x42);
    assert_eq!(sio.read_rx(), 0x41);

    sio.write_tx(0x00);
    assert_eq!(sio.read_rx(), 0x5A);

    sio.write_tx(0x00);
    assert_eq!(sio.read_rx(), 0xFF, "todos soltos -> high = 0xFF");

    sio.write_tx(0x00);
    assert_eq!(sio.read_rx(), 0xFF, "todos soltos -> low = 0xFF");

    sio.write_ctrl(0x0000);
}
