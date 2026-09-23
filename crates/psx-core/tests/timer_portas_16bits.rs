mod support;

use psx_core::bus::{Bus, BusRead, BusWrite};
use support::asm;

const T2_COUNT: u32 = 0x1F80_1120;
const T2_MODE: u32 = 0x1F80_1124;
const T2_TARGET: u32 = 0x1F80_1128;
const T1_COUNT: u32 = 0x1F80_1110;

fn bus() -> Bus {
    asm::bus_with_bios_empty()
}

#[test]
fn lhu_do_contador_devolve_o_valor_e_nao_zero() {
    let mut bus = bus();
    bus.write32::<BusWrite>(T2_COUNT, 0x0000_1234);

    assert_eq!(
        bus.read16::<BusRead>(T2_COUNT),
        0x1234,
        "o contador e um registrador de 16 bits em 1F801100h+N*10h; o `lhu` que o kernel usa \
         para esperar por ele tem de ler o valor, nao zero"
    );
}

#[test]
fn lbu_do_contador_devolve_o_byte_certo_nos_dois_lados() {
    let mut bus = bus();
    bus.write32::<BusWrite>(T2_COUNT, 0x0000_1234);

    assert_eq!(bus.read8::<BusRead>(T2_COUNT), 0x34, "byte baixo");
    assert_eq!(bus.read8::<BusRead>(T2_COUNT + 1), 0x12, "byte alto");
}

#[test]
fn lhu_do_alvo_e_do_modo_tambem_saem_do_registrador() {
    let mut bus = bus();
    bus.write32::<BusWrite>(T2_TARGET, 0x0000_16B0);

    assert_eq!(
        bus.read16::<BusRead>(T2_TARGET),
        0x16B0,
        "o alvo (1F801108h+N*10h) tem 16 bits uteis e e lido por meia palavra"
    );
    bus.write32::<BusWrite>(T2_MODE, 0x0148);
    assert_eq!(
        bus.read16::<BusRead>(T2_MODE) & 0x3FF,
        0x148,
        "o modo tambem: os bits 0-9 escritos voltam num `lhu`"
    );
}

#[test]
fn sh_no_contador_grava_as_duas_metades() {
    let mut bus = bus();

    bus.write16::<BusWrite>(T2_COUNT, 0xBEEF);

    assert_eq!(
        bus.read32::<BusRead>(T2_COUNT) & 0xFFFF,
        0xBEEF,
        "o contador e gravavel; um `sh` tem de entregar os dois bytes, nao so o baixo"
    );
}

#[test]
fn sh_no_alvo_grava_as_duas_metades() {
    let mut bus = bus();

    bus.write16::<BusWrite>(T2_TARGET, 0x16B0);

    assert_eq!(
        bus.read32::<BusRead>(T2_TARGET) & 0xFFFF,
        0x16B0,
        "o kernel arma o alvo por `sh`; perder o byte alto mudaria o periodo do timer"
    );
}

#[test]
fn sh_no_modo_zera_o_contador_como_manda_a_spec() {
    let mut bus = bus();
    bus.write32::<BusWrite>(T2_COUNT, 0x0000_4321);

    bus.write16::<BusWrite>(T2_MODE, 0x0148);

    assert_eq!(
        bus.read32::<BusRead>(T2_COUNT) & 0xFFFF,
        0,
        "o contador e zerado a forca em qualquer escrita no registrador de modo"
    );
}

#[test]
fn contador_que_anda_e_visto_por_lhu() {
    let mut bus = bus();
    // Modo 0 = fonte de clock do sistema, sem sincronizacao: o contador anda com os ciclos.
    bus.write32::<BusWrite>(T2_MODE, 0x0000);

    let antes = bus.read16::<BusRead>(T2_COUNT);
    bus.tick_timers(4_000);
    let depois = bus.read16::<BusRead>(T2_COUNT);

    assert!(
        depois > antes,
        "o laco do kernel le o contador por `lhu` ate passar de um alvo; se a meia palavra nao \
         acompanha o contador, o laco gira para sempre (antes={antes}, depois={depois})"
    );
}

#[test]
fn os_tres_timers_respondem_a_meia_palavra() {
    let mut bus = bus();
    bus.write32::<BusWrite>(T1_COUNT, 0x0000_00A5);

    assert_eq!(
        bus.read16::<BusRead>(T1_COUNT),
        0x00A5,
        "a faixa 1F801100h..1F80112Fh inteira e de registradores, nao so o timer 2"
    );
}
