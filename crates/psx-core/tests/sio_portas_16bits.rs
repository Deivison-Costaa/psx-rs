mod support;

use psx_core::bus::{Bus, BusRead, BusWrite};
use support::asm;

const JOY_DATA: u32 = 0x1F80_1040;
const JOY_STAT: u32 = 0x1F80_1044;
const JOY_MODE: u32 = 0x1F80_1048;
const JOY_CTRL: u32 = 0x1F80_104A;
const JOY_BAUD: u32 = 0x1F80_104E;

// Sequencia que a BIOS SCPH1001 executa em 0x0000454C: JOY_CTRL = 1003h por `sh`
// (TXEN | DTR=/CS | DSR-IRQ-Enable), depois `sb` do byte de endereco 01h.
const CTRL_BIOS: u16 = 0x1003;

fn bus() -> Bus {
    asm::bus_with_bios_empty()
}

fn ctrl16(bus: &mut Bus, val: u16) {
    bus.write16::<BusWrite>(JOY_CTRL, val);
}

fn stat16(bus: &Bus) -> u16 {
    bus.read16::<BusRead>(JOY_STAT)
}

#[test]
fn write16_em_joy_ctrl_entrega_o_byte_alto_em_0x1f80104b() {
    let mut bus = bus();

    ctrl16(&mut bus, CTRL_BIOS);

    assert_eq!(
        bus.sio_mut().read_ctrl(),
        CTRL_BIOS,
        "JOY_CTRL e um registrador de 2 bytes em 1F80104Ah: o byte alto de um `sh` tem de \
         cair em 1F80104Bh, nao ser reescrito por cima do byte baixo"
    );
}

#[test]
fn ctrl_1003h_por_write16_mantem_cs_e_o_byte_enviado_volta_no_rx_fifo() {
    let mut bus = bus();

    ctrl16(&mut bus, CTRL_BIOS);
    bus.write8::<BusWrite>(JOY_DATA, 0x01);

    assert_ne!(
        stat16(&bus) & 0x0002,
        0,
        "com DTR (/CS, bit 1 do JOY_CTRL) baixo, escrever em JOY_TX_DATA inicia a \
         transferencia e o byte recebido acende JOY_STAT.1 (RX FIFO Not Empty) — e o bit que \
         o laco da BIOS em 0x000045C4 espera"
    );
}

#[test]
fn read16_de_joy_stat_traz_o_bit9_de_interrupcao_do_byte_alto() {
    let mut bus = bus();

    ctrl16(&mut bus, CTRL_BIOS);
    bus.write8::<BusWrite>(JOY_DATA, 0x01);

    assert_ne!(
        stat16(&bus) & 0x0200,
        0,
        "JOY_STAT.9 (Interrupt Request) mora no byte alto; um `lhu` em 1F801044h tem de ler \
         1F801045h no byte alto para o driver enxergar o pedido"
    );
}

#[test]
fn write16_em_joy_txdata_envia_um_unico_byte() {
    let mut bus = bus();

    ctrl16(&mut bus, CTRL_BIOS);
    bus.write16::<BusWrite>(JOY_DATA, 0x0001);

    bus.read8::<BusRead>(JOY_DATA);
    assert_eq!(
        stat16(&bus) & 0x0002,
        0,
        "JOY_TX_DATA usa so os bits 0-7 (bits 8-31 nao usados): um `sh` envia UM byte, entao \
         depois de tirar uma resposta do FIFO ele fica vazio"
    );
}

#[test]
fn write16_em_joy_mode_entrega_o_byte_alto() {
    let mut bus = bus();

    bus.write16::<BusWrite>(JOY_MODE, 0x010D);

    assert_eq!(
        bus.read16::<BusRead>(JOY_MODE),
        0x010D,
        "JOY_MODE tem 2 bytes: o bit 8 (clock polarity) so existe se o byte alto for escrito \
         em 1F801049h"
    );
}

#[test]
fn write16_em_joy_baud_entrega_o_byte_alto() {
    let mut bus = bus();

    bus.write16::<BusWrite>(JOY_BAUD, 0x0188);

    assert_eq!(
        bus.read16::<BusRead>(JOY_BAUD),
        0x0188,
        "JOY_BAUD e o valor de recarga de 16 bits do temporizador de baud: escrever so o byte \
         baixo trocaria a taxa do barramento do controle"
    );
}

#[test]
fn ack_por_write16_limpa_o_bit9_sem_soltar_o_cs() {
    let mut bus = bus();
    ctrl16(&mut bus, CTRL_BIOS);
    bus.write8::<BusWrite>(JOY_DATA, 0x01);
    assert_ne!(stat16(&bus) & 0x0200, 0, "pre-condicao: IRQ pedida");

    // JOY_CTRL.bit4 = Acknowledge; o driver escreve 1013h, mantendo TXEN/DTR/DSR-IRQ.
    ctrl16(&mut bus, CTRL_BIOS | 0x0010);

    assert_eq!(
        stat16(&bus) & 0x0200,
        0,
        "bit 4 do JOY_CTRL reseta JOY_STAT.9"
    );
    assert_eq!(
        bus.sio_mut().read_ctrl() & 0x0002,
        0x0002,
        "o ack nao pode derrubar o DTR (/CS): soltar o /CS no meio do pacote aborta a leitura \
         do controle"
    );
}

#[test]
fn read16_de_joy_ctrl_traz_o_byte_alto_certo() {
    let mut bus = bus();
    ctrl16(&mut bus, CTRL_BIOS);

    assert_eq!(
        bus.read16::<BusRead>(JOY_CTRL),
        CTRL_BIOS,
        "o driver le JOY_CTRL em 0x00004590 e reescreve com o bit de ack; se a leitura perder \
         o byte alto, a escrita seguinte solta o /CS"
    );
}
