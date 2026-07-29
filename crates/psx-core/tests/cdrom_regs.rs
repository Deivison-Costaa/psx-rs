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

fn intmsk_write(bus: &mut Bus, val: u8) {
    set_bank(bus, 1);
    cd_write(bus, 2, val);
    set_bank(bus, 0);
}

#[test]
fn index0_read_retorna_status_inicial() {
    let bus = bus();
    let hsts = cd_read(&bus, 0);
    assert_eq!(hsts & 0x3, 0, "bank inicial deve ser 0");
    assert_ne!(
        hsts & (1 << 3),
        0,
        "PRMEMPT: parameter FIFO vazia no inicio"
    );
    assert_ne!(
        hsts & (1 << 4),
        0,
        "PRMWRDY: parameter FIFO nao cheia no inicio"
    );
    assert_eq!(hsts & (1 << 5), 0, "RSLRRDY: result FIFO vazia no inicio");
    assert_eq!(hsts & (1 << 7), 0, "BUSYSTS: nao ocupado no inicio");
}

#[test]
fn index0_write_troca_de_bank() {
    let mut bus = bus();
    cd_write(&mut bus, 0, 2);
    let hsts = cd_read(&bus, 0);
    assert_eq!(
        hsts & 0x3,
        2,
        "bank deve ser 2 apos escrever 0x02 no INDEX0"
    );
    cd_write(&mut bus, 0, 3);
    let hsts2 = cd_read(&bus, 0);
    assert_eq!(hsts2 & 0x3, 3, "bank deve ser 3 apos escrever 0x03");
    assert_eq!(hsts2 & 0x3, 3, "bits de bank sao 2 bits (0-3)");
}

#[test]
fn init_command_dispara_int3_e_depois_int2() {
    let mut bus = bus();
    set_bank(&mut bus, 0);
    cd_write(&mut bus, 1, 0x0A);
    let hintsts = hintsts_read_bank1(&mut bus);
    assert_eq!(hintsts & 0x7, 3, "INT3 apos comando Init (0Ah)");
    set_bank(&mut bus, 0);
    let hsts = cd_read(&bus, 0);
    assert_ne!(hsts & (1 << 7), 0, "BUSYSTS=1 apos comando");
    cd_read(&bus, 1);
    hclrctl_write(&mut bus, 0x07);
    let hintsts2 = hintsts_read_bank1(&mut bus);
    assert_eq!(hintsts2 & 0x7, 2, "INT2 apos acknowledge do INT3 do Init");
}

#[test]
fn getstat_20h_retorna_data_e_versao() {
    let mut bus = bus();
    set_bank(&mut bus, 0);
    cd_write(&mut bus, 2, 0x20);
    cd_write(&mut bus, 1, 0x19);
    let yy = cd_read(&bus, 1);
    let mm = cd_read(&bus, 1);
    let dd = cd_read(&bus, 1);
    let ver = cd_read(&bus, 1);
    assert_eq!(yy, 0x97, "ano BCD (97h = 1997)");
    assert_eq!(mm, 0x01, "mes BCD (01h = Janeiro)");
    assert_eq!(dd, 0x10, "dia BCD (10h = dia 10)");
    assert_eq!(ver, 0xC2, "versao (C2h = vC2)");
}

#[test]
fn getstat_21h_retorna_flags_dos_switches() {
    let mut bus = bus();
    set_bank(&mut bus, 0);
    cd_write(&mut bus, 2, 0x21);
    cd_write(&mut bus, 1, 0x19);
    let flags = cd_read(&bus, 1);
    assert_eq!(flags & 0x1, 1, "bit0: HeadIsAtPos0=1 (assumido)");
    assert_eq!(flags & 0x2, 0, "bit1: DoorIsOpen=0 (assumido)");
}

#[test]
fn getid_sem_disco_retorna_int5() {
    let mut bus = bus();
    set_bank(&mut bus, 0);
    cd_write(&mut bus, 1, 0x1A);
    let hintsts = hintsts_read_bank1(&mut bus);
    assert_eq!(hintsts & 0x7, 3, "INT3 na primeira resposta do GetID");
    cd_read(&bus, 1);
    hclrctl_write(&mut bus, 0x07);
    let hintsts2 = hintsts_read_bank1(&mut bus);
    assert_eq!(hintsts2 & 0x7, 5, "INT5 na segunda resposta (sem disco)");
    set_bank(&mut bus, 0);
    let stat = cd_read(&bus, 1);
    assert_eq!(stat, 0x08, "primeiro byte da resposta INT5 = 08h");
}

#[test]
fn result_fifo_leitura_retorna_padding_zero_apos_esvaziar() {
    let mut bus = bus();
    set_bank(&mut bus, 0);
    cd_write(&mut bus, 2, 0x20);
    cd_write(&mut bus, 1, 0x19);
    for _ in 0..4 {
        cd_read(&bus, 1);
    }
    let pad = cd_read(&bus, 1);
    assert_eq!(pad, 0, "leitura alem do fim do FIFO retorna 0");
}

#[test]
fn irq_pendente_quando_intsts_e_intmsk_se_sobrepoem() {
    let mut bus = bus();
    set_bank(&mut bus, 0);
    intmsk_write(&mut bus, 0x1F);
    cd_write(&mut bus, 1, 0x0A);
    assert!(
        bus.cdrom().irq_pending(),
        "IRQ pendente quando INTMSK cobre INTSTS"
    );
}

#[test]
fn irq_nao_pendente_quando_intmsk_nao_cobre_intsts() {
    let mut bus = bus();
    set_bank(&mut bus, 0);
    intmsk_write(&mut bus, 0x00);
    cd_write(&mut bus, 1, 0x0A);
    assert!(
        !bus.cdrom().irq_pending(),
        "IRQ nao pendente quando INTMSK=0 mesmo com INTSTS=3"
    );
}

#[test]
fn param_fifo_aceita_ate_16_bytes() {
    let mut bus = bus();
    set_bank(&mut bus, 0);
    for i in 0..16 {
        cd_write(&mut bus, 2, i);
    }
    let hsts = cd_read(&bus, 0);
    assert_eq!(hsts & (1 << 4), 0, "PRMWRDY=0 quando FIFO cheia (16 bytes)");
}

#[test]
fn escrever_mode_reseta_param_fifo() {
    let mut bus = bus();
    set_bank(&mut bus, 0);
    cd_write(&mut bus, 2, 0xAA);
    cd_write(&mut bus, 2, 0xBB);
    let hsts_antes = cd_read(&bus, 0);
    assert_eq!(
        hsts_antes & (1 << 3),
        0,
        "PRMEMPT=0 — tem bytes no param FIFO"
    );
    hclrctl_write(&mut bus, 0x40);
    set_bank(&mut bus, 0);
    let hsts_depois = cd_read(&bus, 0);
    assert_ne!(
        hsts_depois & (1 << 3),
        0,
        "PRMEMPT=1 — param FIFO foi limpa"
    );
}

#[test]
fn result_fifo_esvaziado_apos_acknowledge_e_leitura_da_segunda_resposta() {
    let mut bus = bus();
    set_bank(&mut bus, 0);
    cd_write(&mut bus, 1, 0x0A);
    let _stat = cd_read(&bus, 1);
    hclrctl_write(&mut bus, 0x03);
    set_bank(&mut bus, 0);
    let stat2 = cd_read(&bus, 1);
    assert_eq!(stat2, 0x02, "INT2 entrega stat=02h do Init");
    let hsts = cd_read(&bus, 0);
    assert_eq!(
        hsts & (1 << 5),
        0,
        "RSLRRDY=0 — result FIFO vazia apos ler segunda resposta"
    );
}

#[test]
fn prmwrdy_setado_quando_param_fifo_nao_cheia() {
    let mut bus = bus();
    set_bank(&mut bus, 0);
    let hsts = cd_read(&bus, 0);
    assert_ne!(
        hsts & (1 << 4),
        0,
        "PRMWRDY=1 — param FIFO pronta para receber no inicio"
    );
}
