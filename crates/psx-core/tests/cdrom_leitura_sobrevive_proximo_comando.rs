mod support;

use psx_core::bus::{Bus, BusRead};
use support::asm;

const CD_BASE: u32 = 0x1F80_1800;

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

/// Avanca ate a entrega ja agendada. § 06-cdrom.md L2077-2078: o primeiro setor de um
/// ReadN chega depois da BUSCA, que agora custa muito mais que a resposta de um comando
/// passivo — por isso o comando intercalado responde ANTES do setor, e nao depois.
fn tick_ate_a_entrega(bus: &mut Bus) {
    let ciclos = bus.cdrom().second_response_cycles() as u32;
    bus.tick_timers(ciclos);
}

/// Prepara um ReadN em andamento e devolve com o INT3 do ReadN ja reconhecido.
fn read_n_em_andamento(bus: &mut Bus) {
    bus.cdrom_mut().insert_disc();
    param_write(bus, 0x02);
    param_write(bus, 0x10);
    param_write(bus, 0x00);
    send_command(bus, 0x02);
    let _ = result_read(bus);
    hclrctl_write(bus, 0x07);
    send_command(bus, 0x06);
    let _ = result_read(bus);
    hclrctl_write(bus, 0x07);
}

// § Sending a new command while another is pending (06-cdrom.md L471-473): a spec
// mede a sequencia literal "ReadN/ReadS -> Wait for INT3 IRQ -> clear IRQ -> SetMode/
// SetLoc/..." e conclui "Will not drop any of the two commands, thus execute
// sequentially". Setmode NAO para o drive (L564: so INT3, sem completion), entao a
// leitura em curso tem que sobreviver a ele. So os comandos que de fato mexem no
// motor/posicao (Stop, Pause, Init, Seek, Reset, novo Read/Play) abortam a entrega.
#[test]
fn setmode_durante_read_n_nao_mata_o_streaming() {
    let mut bus = bus();
    read_n_em_andamento(&mut bus);

    param_write(&mut bus, 0x80);
    send_command(&mut bus, 0x0E);

    assert_eq!(
        hintsts_read_bank1(&mut bus) & 0x7,
        3,
        "a busca do primeiro setor e' bem mais longa que a resposta de um comando \
         passivo, entao o Setmode responde INT3 ainda durante a busca"
    );
    let _ = result_read(&mut bus);
    hclrctl_write(&mut bus, 0x07);

    tick_ate_a_entrega(&mut bus);

    assert_eq!(
        hintsts_read_bank1(&mut bus) & 0x7,
        1,
        "Setmode durante ReadN nao pode cancelar a entrega de setor: a spec (L471-473) \
         mede exatamente ReadN seguido de SetMode e diz que nenhum dos dois e' \
         descartado — se o Setmode zera o CDROM_SECOND agendado e nada rearma, o \
         streaming morre pra sempre"
    );
}

// § idem L471-473, agora com Setloc, o outro comando que a spec cita nominalmente na
// mesma sequencia. Setloc so anota a posicao alvo; quem move o drive e' o Seek/Read
// seguinte, entao a leitura em curso tambem sobrevive.
#[test]
fn setloc_durante_read_n_nao_mata_o_streaming() {
    let mut bus = bus();
    read_n_em_andamento(&mut bus);

    param_write(&mut bus, 0x02);
    param_write(&mut bus, 0x20);
    param_write(&mut bus, 0x00);
    send_command(&mut bus, 0x02);

    assert_eq!(
        hintsts_read_bank1(&mut bus) & 0x7,
        3,
        "o Setloc responde INT3 ainda durante a busca do primeiro setor"
    );
    let _ = result_read(&mut bus);
    hclrctl_write(&mut bus, 0x07);

    tick_ate_a_entrega(&mut bus);

    assert_eq!(
        hintsts_read_bank1(&mut bus) & 0x7,
        1,
        "Setloc durante ReadN tambem nao pode cancelar a entrega em voo (L471-473)"
    );
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

    // Nop chega logo apos o ACK do INT3 do ReadN, enquanto o drive ainda BUSCA o
    // primeiro setor — exatamente a janela de corrida do jogo.
    send_command(&mut bus, 0x01);

    assert_eq!(
        hintsts_read_bank1(&mut bus) & 0x7,
        3,
        "o Nop responde INT3 durante a busca do primeiro setor"
    );
    let _ = result_read(&mut bus);
    hclrctl_write(&mut bus, 0x07);

    tick_ate_a_entrega(&mut bus);

    assert_eq!(
        hintsts_read_bank1(&mut bus) & 0x7,
        1,
        "INT1 do setor de dado do ReadN tem que chegar mesmo com um Nop despachado \
         entre o ACK do primeiro response e a entrega do setor — a leitura nao \
         pode ficar presa num retry infinito so' porque a CPU emitiu outro \
         comando enquanto esperava"
    );
}
