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
}

fn param_write(bus: &mut Bus, val: u8) {
    set_bank(bus, 0);
    cd_write(bus, 2, val);
}

fn result_read(bus: &mut Bus) -> u8 {
    set_bank(bus, 0);
    cd_read(bus, 1)
}

fn rddata_read(bus: &mut Bus) -> u8 {
    set_bank(bus, 0);
    cd_read(bus, 2)
}

fn insert_stub_disc(bus: &mut Bus) {
    bus.cdrom_mut().insert_disc();
}

fn bcd_minute(m: u8) -> u8 {
    m
}

fn bcd_second(s: u8) -> u8 {
    s
}

fn bcd_sector(f: u8) -> u8 {
    f
}

#[test]
fn read_n_retorna_int3_depois_int1_com_dados() {
    let mut bus = bus();
    insert_stub_disc(&mut bus);
    param_write(&mut bus, bcd_minute(0x02));
    param_write(&mut bus, bcd_second(0x10));
    param_write(&mut bus, bcd_sector(0x00));
    send_command(&mut bus, 0x02);
    let _ = result_read(&mut bus);

    send_command(&mut bus, 0x06);
    let hintsts = hintsts_read_bank1(&mut bus);
    assert_eq!(hintsts & 0x7, 3, "INT3 apos ReadN (06h)");

    let stat = result_read(&mut bus);
    assert_ne!(stat & (1 << 5), 0, "stat bit5=1 — reading");

    hclrctl_write(&mut bus, 0x07);

    let hintsts2 = hintsts_read_bank1(&mut bus);
    assert_eq!(hintsts2 & 0x7, 1, "INT1 apos acknowledge do INT3 do ReadN");

    let stat2 = result_read(&mut bus);
    assert_ne!(stat2 & (1 << 5), 0, "stat bit5=1 — reading no INT1");

    let data0 = rddata_read(&mut bus);
    let data1 = rddata_read(&mut bus);
    assert_ne!(data0, 0, "RDDATA retorna dados validos apos INT1");
    assert_ne!(data1, 0, "RDDATA retorna dados validos no segundo byte");
}

#[test]
fn read_s_retorna_int3_depois_int1_com_dados() {
    let mut bus = bus();
    insert_stub_disc(&mut bus);
    param_write(&mut bus, bcd_minute(0x02));
    param_write(&mut bus, bcd_second(0x10));
    param_write(&mut bus, bcd_sector(0x00));
    send_command(&mut bus, 0x02);
    let _ = result_read(&mut bus);

    send_command(&mut bus, 0x1B);
    let hintsts = hintsts_read_bank1(&mut bus);
    assert_eq!(hintsts & 0x7, 3, "INT3 apos ReadS (1Bh)");

    let stat = result_read(&mut bus);
    assert_ne!(stat & (1 << 5), 0, "stat bit5=1 — reading");

    hclrctl_write(&mut bus, 0x07);

    let hintsts2 = hintsts_read_bank1(&mut bus);
    assert_eq!(hintsts2 & 0x7, 1, "INT1 apos acknowledge do INT3 do ReadS");

    let stat2 = result_read(&mut bus);
    assert_ne!(stat2 & (1 << 5), 0, "stat bit5=1 — reading no INT1");

    let data0 = rddata_read(&mut bus);
    assert_ne!(data0, 0, "RDDATA retorna dados validos apos INT1");
}

#[test]
fn read_n_continua_apos_primeiro_acknowledge() {
    let mut bus = bus();
    insert_stub_disc(&mut bus);
    param_write(&mut bus, bcd_minute(0x02));
    param_write(&mut bus, bcd_second(0x10));
    param_write(&mut bus, bcd_sector(0x00));
    send_command(&mut bus, 0x02);
    let _ = result_read(&mut bus);

    send_command(&mut bus, 0x06);
    let _ = result_read(&mut bus);
    hclrctl_write(&mut bus, 0x07);
    let _stat = result_read(&mut bus);

    hclrctl_write(&mut bus, 0x07);

    let hintsts3 = hintsts_read_bank1(&mut bus);
    assert_eq!(
        hintsts3 & 0x7,
        1,
        "INT1 apos segundo acknowledge — ReadN continua"
    );
}

#[test]
fn read_s_para_apos_primeiro_acknowledge() {
    let mut bus = bus();
    insert_stub_disc(&mut bus);
    param_write(&mut bus, bcd_minute(0x02));
    param_write(&mut bus, bcd_second(0x10));
    param_write(&mut bus, bcd_sector(0x00));
    send_command(&mut bus, 0x02);
    let _ = result_read(&mut bus);

    send_command(&mut bus, 0x1B);
    let _ = result_read(&mut bus);
    hclrctl_write(&mut bus, 0x07);
    let _stat = result_read(&mut bus);

    hclrctl_write(&mut bus, 0x07);

    let hintsts3 = hintsts_read_bank1(&mut bus);
    assert_eq!(hintsts3 & 0x7, 0, "INT0 — ReadS para apos primeiro setor");
}

#[test]
fn rddata_retorna_sequencia_de_bytes() {
    let mut bus = bus();
    insert_stub_disc(&mut bus);
    param_write(&mut bus, bcd_minute(0x02));
    param_write(&mut bus, bcd_second(0x10));
    param_write(&mut bus, bcd_sector(0x00));
    send_command(&mut bus, 0x02);
    let _ = result_read(&mut bus);

    send_command(&mut bus, 0x06);
    let _ = result_read(&mut bus);
    hclrctl_write(&mut bus, 0x07);
    let _ = result_read(&mut bus);

    let b0 = rddata_read(&mut bus);
    let b1 = rddata_read(&mut bus);
    let b2 = rddata_read(&mut bus);
    let b3 = rddata_read(&mut bus);
    assert_ne!(
        (b0 as u32, b1 as u32, b2 as u32, b3 as u32),
        (0, 0, 0, 0),
        "RDDATA retorna bytes variados, nao so zeros"
    );
}

#[test]
fn read_n_sem_disco_retorna_int5() {
    let mut bus = bus();
    param_write(&mut bus, bcd_minute(0x02));
    param_write(&mut bus, bcd_second(0x10));
    param_write(&mut bus, bcd_sector(0x00));
    send_command(&mut bus, 0x02);
    let _ = result_read(&mut bus);

    send_command(&mut bus, 0x06);
    let hintsts = hintsts_read_bank1(&mut bus);
    assert_eq!(hintsts & 0x7, 5, "INT5 apos ReadN sem disco");

    let stat = result_read(&mut bus);
    assert_ne!(stat & 0x01, 0, "stat bit0=1 — erro");
    let err = result_read(&mut bus);
    assert_eq!(err, 0x80, "error byte = 80h (sem disco)");
}

#[test]
fn read_s_sem_disco_retorna_int5() {
    let mut bus = bus();
    param_write(&mut bus, bcd_minute(0x02));
    param_write(&mut bus, bcd_second(0x10));
    param_write(&mut bus, bcd_sector(0x00));
    send_command(&mut bus, 0x02);
    let _ = result_read(&mut bus);

    send_command(&mut bus, 0x1B);
    let hintsts = hintsts_read_bank1(&mut bus);
    assert_eq!(hintsts & 0x7, 5, "INT5 apos ReadS sem disco");
}

#[test]
fn pause_para_read_n() {
    let mut bus = bus();
    insert_stub_disc(&mut bus);
    param_write(&mut bus, bcd_minute(0x02));
    param_write(&mut bus, bcd_second(0x10));
    param_write(&mut bus, bcd_sector(0x00));
    send_command(&mut bus, 0x02);
    let _ = result_read(&mut bus);

    send_command(&mut bus, 0x06);
    let _ = result_read(&mut bus);
    hclrctl_write(&mut bus, 0x07);
    let _ = result_read(&mut bus);

    send_command(&mut bus, 0x09);
    let hintsts_pause = hintsts_read_bank1(&mut bus);
    assert_eq!(hintsts_pause & 0x7, 3, "INT3 apos Pause durante ReadN");

    let _ = result_read(&mut bus);
    hclrctl_write(&mut bus, 0x07);

    let hintsts_after = hintsts_read_bank1(&mut bus);
    assert_eq!(hintsts_after & 0x7, 2, "INT2 — segunda resposta do Pause");

    let stat_after = result_read(&mut bus);
    assert_eq!(stat_after & (1 << 5), 0, "stat bit5=0 — parou de ler");

    hclrctl_write(&mut bus, 0x07);
    let hintsts_final = hintsts_read_bank1(&mut bus);
    assert_eq!(
        hintsts_final & 0x7,
        0,
        "INT0 — ReadN parado apos Pause completo"
    );
}

#[test]
fn hsts_drqsts_setado_quando_dados_disponiveis() {
    let mut bus = bus();
    insert_stub_disc(&mut bus);
    param_write(&mut bus, bcd_minute(0x02));
    param_write(&mut bus, bcd_second(0x10));
    param_write(&mut bus, bcd_sector(0x00));
    send_command(&mut bus, 0x02);
    let _ = result_read(&mut bus);

    send_command(&mut bus, 0x06);
    let _ = result_read(&mut bus);
    hclrctl_write(&mut bus, 0x07);
    let _ = result_read(&mut bus);

    set_bank(&mut bus, 0);
    let hsts = cd_read(&bus, 0);
    assert_ne!(
        hsts & (1 << 6),
        0,
        "DRQSTS=1 quando dados disponiveis no buffer"
    );
}

fn setloc(disc: &mut Bus, mm: u8, ss: u8, ff: u8) {
    param_write(disc, bcd_minute(mm));
    param_write(disc, bcd_second(ss));
    param_write(disc, bcd_sector(ff));
    send_command(disc, 0x02);
    let _ = result_read(disc);
}

fn read_n(disc: &mut Bus) {
    send_command(disc, 0x06);
    let _ = result_read(disc);
    hclrctl_write(disc, 0x07);
    let _ = result_read(disc);
}

#[test]
fn setloc_consumido_pelo_read_n() {
    let mut bus = bus();
    insert_stub_disc(&mut bus);
    setloc(&mut bus, 0x02, 0x10, 0x00);
    read_n(&mut bus);
    hclrctl_write(&mut bus, 0x07);
    let hintsts = hintsts_read_bank1(&mut bus);
    assert_eq!(hintsts & 0x7, 1, "ReadN continua — Setloc foi consumido");
}
