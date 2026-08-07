mod support;

use psx_core::bus::{Bus, BusRead};
use support::asm;

const CD_BASE: u32 = 0x1F80_1800;
const CADENCIA_VELOCIDADE_NORMAL: u32 = 451_584;

fn bus() -> Bus {
    asm::bus_with_bios_empty()
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

fn hintsts_read_bank1(bus: &mut Bus) -> u8 {
    set_bank(bus, 1);
    let val = cd_read(bus, 3);
    set_bank(bus, 0);
    val
}

fn hclrctl_write(bus: &mut Bus, val: u8) {
    set_bank(bus, 1);
    cd_write(bus, 3, val);
    set_bank(bus, 0);
}

fn send_command(bus: &mut Bus, cmd: u8) {
    set_bank(bus, 0);
    cd_write(bus, 1, cmd);
    bus.tick_timers(0x1_4000);
}

fn param_write(bus: &mut Bus, val: u8) {
    set_bank(bus, 0);
    cd_write(bus, 2, val);
}

fn result_read(bus: &mut Bus) -> u8 {
    set_bank(bus, 0);
    cd_read(bus, 1)
}

// § "cancela resposta armada ao aceitar comando" (commit 7b57967) descarta um 2o
// response OBSOLETO quando um comando novo e' aceito enquanto ocioso — mas
// int1_pending vira false assim que o driver da' ACK do INT1 do ReadN, ANTES do
// CDROM_SECOND (entrega do setor, ja agendado nesse ACK) disparar de fato. Se o
// jogo emite outro comando rapido (ex.: Nop, GT2 faz isso no loop de audio de
// fundo) enquanto reading=true, esse comando nao pode cancelar a entrega do
// setor que ainda esta a caminho.
#[test]
fn readn_seguido_de_nop_antes_da_entrega_nao_cancela_o_setor() {
    let mut bus = bus();
    bus.cdrom_mut().insert_disc();

    param_write(&mut bus, 0x02);
    param_write(&mut bus, 0x10);
    param_write(&mut bus, 0x00);
    send_command(&mut bus, 0x02);
    let _ = result_read(&mut bus);
    hclrctl_write(&mut bus, 0x07);

    send_command(&mut bus, 0x06);
    let _ = result_read(&mut bus);
    hclrctl_write(&mut bus, 0x07);

    // Nop chega logo apos o ACK do INT1 do ReadN, bem antes da cadencia do
    // primeiro setor (0x4A00 ciclos) — exatamente a janela de corrida do jogo.
    send_command(&mut bus, 0x01);
    let _ = result_read(&mut bus);
    hclrctl_write(&mut bus, 0x07);

    bus.tick_timers(CADENCIA_VELOCIDADE_NORMAL);

    assert_eq!(
        hintsts_read_bank1(&mut bus) & 0x7,
        1,
        "INT1 do setor de dado do ReadN tem que chegar mesmo com um Nop latched \
         entre o ACK do primeiro response e a entrega do setor — a leitura nao \
         pode ficar presa num retry infinito so' porque a CPU emitiu outro \
         comando enquanto esperava"
    );
}
