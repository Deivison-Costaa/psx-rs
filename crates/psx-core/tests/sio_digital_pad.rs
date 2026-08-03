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
    sio.deliver_ack();
    assert!(
        sio.read_stat() & 0x80 != 0,
        "STAT.7 (DSR) deve ser 1 quando o /ACK do periferico chega"
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
    sio.deliver_ack();
    let _ = sio.read_rx();
    sio.write_tx(0x42);
    sio.deliver_ack();

    assert!(sio.read_stat() & 0x02 != 0);
    assert!(
        sio.read_stat() & (1 << 9) != 0,
        "pre-condicao: o /ACK entregue acende STAT.9, senao o ack de CTRL.4 nao mede nada"
    );

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

    // § Controller Transfer (L546-549): a ordem e "swlo  Receive Digital Switches
    // bit0..7" e SO DEPOIS "swhi  ... bit8..15". § Standard Controllers (L618-625):
    // bit3 = Start, bit14 = Cross. Trocar os dois bytes entrega Start ao jogo na
    // posicao de R1 — foi o que impediu o Crash de sair do menu (achado 0186.1).
    sio.write_tx(0x00);
    assert_eq!(
        sio.read_rx(),
        0xF7,
        "swlo (bit0..7) vem primeiro: Start(bit3)=0 -> 11110111 = 0xF7"
    );

    sio.write_tx(0x00);
    assert_eq!(
        sio.read_rx(),
        0xBF,
        "swhi (bit8..15) vem depois: Cross(bit14)=0 -> 10111111 = 0xBF"
    );

    sio.write_ctrl(0x0000);
}

#[test]
fn start_sozinho_sai_no_primeiro_byte_de_switches() {
    let sio = Sio::new();
    sio.connect_digital_pad(true);
    sio.set_buttons(!(1u16 << 3));

    sio.write_ctrl(0x0002);
    sio.write_tx(0x01);
    let _ = sio.read_rx();
    sio.write_tx(0x42);
    let _ = sio.read_rx();
    sio.write_tx(0x00);
    let _ = sio.read_rx();

    sio.write_tx(0x00);
    assert_eq!(
        sio.read_rx(),
        0xF7,
        "Start e o bit3 do swlo, que e o PRIMEIRO byte de switches"
    );

    sio.write_tx(0x00);
    assert_eq!(
        sio.read_rx(),
        0xFF,
        "nenhum botao do swhi (L2/R2/L1/R1/triangulo/circulo/cross/quadrado) foi apertado"
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

#[test]
fn leitura_32bit_do_sio_data_consome_byte_do_fifo() {
    let sio = Sio::new();
    sio.connect_digital_pad(true);

    sio.write_ctrl(0x0002);
    sio.write_tx(0x01);
    let _ = sio.read_rx();

    sio.write_tx(0x42);
    let primeira = sio.read_data();
    assert_eq!(
        primeira & 0xFF,
        0x41,
        "leitura 32bit deve retornar ID low 0x41"
    );

    let segunda = sio.read_data();
    assert_eq!(
        segunda & 0xFF,
        0xFF,
        "segunda leitura 32bit deve retornar 0xFF (FIFO vazio = HiZ)"
    );

    sio.write_ctrl(0x0000);
}
