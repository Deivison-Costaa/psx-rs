mod support;

use psx_core::bus::{Bus, BusRead};
use support::asm;

const CD_BASE: u32 = 0x1F80_1800;
const ESPERA_PRIMEIRA_RESPOSTA: u32 = 0x1_4000;
const D3_MADR: u32 = 0x1F80_10B0;
const D3_BCR: u32 = 0x1F80_10B4;
const D3_CHCR: u32 = 0x1F80_10B8;
const DPCR: u32 = 0x1F80_10F0;

fn bus() -> Bus {
    asm::bus_with_bios_empty()
}

fn cd_write(bus: &mut Bus, offset: u32, val: u8) {
    bus.write8::<BusRead>(CD_BASE + offset, val);
}

fn cd_read(bus: &Bus, offset: u32) -> u8 {
    bus.read8::<BusRead>(CD_BASE + offset)
}

fn set_bank(bus: &mut Bus, b: u8) {
    cd_write(bus, 0, b);
}

fn hsts_read(bus: &Bus) -> u8 {
    cd_read(bus, 0)
}

fn hchpctl_write(bus: &mut Bus, val: u8) {
    set_bank(bus, 0);
    cd_write(bus, 3, val);
    set_bank(bus, 0);
}

fn hclrctl_write(bus: &mut Bus, val: u8) {
    set_bank(bus, 1);
    cd_write(bus, 3, val);
    set_bank(bus, 0);
}

fn send_command(bus: &mut Bus, cmd: u8) {
    set_bank(bus, 0);
    cd_write(bus, 1, cmd);
    bus.tick_timers(ESPERA_PRIMEIRA_RESPOSTA);
}

fn param_write(bus: &mut Bus, val: u8) {
    set_bank(bus, 0);
    cd_write(bus, 2, val);
}

fn result_read(bus: &mut Bus) -> u8 {
    set_bank(bus, 0);
    cd_read(bus, 1)
}

fn insert_stub_disc(bus: &mut Bus) {
    bus.cdrom_mut().insert_disc();
}

fn setloc(bus: &mut Bus, mm: u8, ss: u8, ff: u8) {
    param_write(bus, mm);
    param_write(bus, ss);
    param_write(bus, ff);
    send_command(bus, 0x02);
    let _ = result_read(bus);
}

fn read_n_and_int1(bus: &mut Bus) {
    send_command(bus, 0x06);
    let _ = result_read(bus);
    hclrctl_write(bus, 0x07);
    bus.tick_timers(0x6000);
    set_bank(bus, 1);
    let hintsts = cd_read(bus, 3) & 0x7;
    set_bank(bus, 0);
    assert_eq!(
        hintsts, 1,
        "pre-condicao: o setor tem de ter chegado (INT1). Sem isto os testes negativos \
         abaixo ficam verdes com o drive morto, e nao por causa do que eles afirmam medir"
    );
    let _ = result_read(bus);
}

#[test]
fn dma3_madr_gravavel_e_legivel() {
    let mut bus = bus();
    bus.write32::<BusRead>(D3_MADR, 0x0012_3456);
    let val = bus.read32::<BusRead>(D3_MADR);
    assert_eq!(val & 0x00FF_FFFF, 0x0012_3456);
}

#[test]
fn dma3_bcr_gravavel_e_legivel() {
    let mut bus = bus();
    bus.write32::<BusRead>(D3_BCR, 0x0000_0042);
    let val = bus.read32::<BusRead>(D3_BCR);
    assert_eq!(val & 0xFFFF, 0x0042);
}

#[test]
fn dma3_chcr_gravavel_e_legivel() {
    let mut bus = bus();
    bus.write32::<BusRead>(D3_CHCR, 0x1100_0000);
    let val = bus.read32::<BusRead>(D3_CHCR);
    assert_eq!(val, 0x1100_0000);
}

#[test]
fn dpcr_dma3_priority_master_enable_gravavel_e_legivel() {
    let mut bus = bus();
    bus.write32::<BusRead>(DPCR, 0x1234_5678);
    let val = bus.read32::<BusRead>(DPCR);
    assert_eq!(val, 0x1234_5678, "DPCR gravavel e legivel");
    assert_eq!(
        (val >> 12) & 0xF,
        0x5,
        "bits 12-15 refletem priority e master enable do DMA3"
    );
}

#[test]
fn drqsts_baixo_quando_bfrd_nao_setado_apos_int1() {
    let mut bus = bus();
    insert_stub_disc(&mut bus);
    setloc(&mut bus, 0x02, 0x10, 0x00);
    read_n_and_int1(&mut bus);

    let hsts = hsts_read(&bus);
    assert_eq!(
        hsts & (1 << 6),
        0,
        "DRQSTS=0 quando BFRD nao foi setado, mesmo com dados disponiveis"
    );
}

#[test]
fn drqsts_alto_quando_bfrd_setado_apos_int1() {
    let mut bus = bus();
    insert_stub_disc(&mut bus);
    setloc(&mut bus, 0x02, 0x10, 0x00);
    read_n_and_int1(&mut bus);

    hchpctl_write(&mut bus, 0x80);

    let hsts = hsts_read(&bus);
    assert_ne!(
        hsts & (1 << 6),
        0,
        "DRQSTS=1 quando BFRD=1 e dados disponiveis"
    );
}

#[test]
fn dma3_transfere_dados_do_setor_para_ram() {
    let mut bus = bus();
    insert_stub_disc(&mut bus);
    setloc(&mut bus, 0x02, 0x10, 0x00);
    read_n_and_int1(&mut bus);
    hchpctl_write(&mut bus, 0x80);

    let dest: u32 = 0x0001_0000;
    bus.write32::<BusRead>(D3_MADR, dest);
    bus.write32::<BusRead>(D3_BCR, 4);
    bus.write32::<BusRead>(DPCR, 0x0765_4321 | (1 << 15));
    bus.write32::<BusRead>(D3_CHCR, 0x1100_0000);

    let w0 = bus.read32::<BusRead>(dest);
    let w1 = bus.read32::<BusRead>(dest + 4);
    let w2 = bus.read32::<BusRead>(dest + 8);
    let w3 = bus.read32::<BusRead>(dest + 12);
    assert_eq!(w0, 0x0403_0201, "primeiro word: bytes 1,2,3,4 do stub");
    assert_eq!(w1, 0x0807_0605, "segundo word: bytes 5,6,7,8 do stub");
    assert_eq!(w2, 0x0C0B_0A09, "terceiro word: bytes 9,10,11,12 do stub");
    assert_eq!(w3, 0x100F_0E0D, "quarto word: bytes 13,14,15,16 do stub");
}

#[test]
fn dma3_bit24_e_bit28_limpos_apos_transferencia() {
    let mut bus = bus();
    insert_stub_disc(&mut bus);
    setloc(&mut bus, 0x02, 0x10, 0x00);
    read_n_and_int1(&mut bus);
    hchpctl_write(&mut bus, 0x80);

    bus.write32::<BusRead>(D3_MADR, 0x0001_0000);
    bus.write32::<BusRead>(D3_BCR, 1);
    bus.write32::<BusRead>(DPCR, 0x0765_4321 | (1 << 15));
    bus.write32::<BusRead>(D3_CHCR, 0x1100_0000);

    let chcr = bus.read32::<BusRead>(D3_CHCR);
    assert_eq!(chcr & (1 << 24), 0, "bit 24 limpo apos transferencia");
    assert_eq!(chcr & (1 << 28), 0, "bit 28 limpo apos transferencia");
}

#[test]
fn dma3_nao_dispara_sem_bfrd() {
    let mut bus = bus();
    insert_stub_disc(&mut bus);
    setloc(&mut bus, 0x02, 0x10, 0x00);
    read_n_and_int1(&mut bus);

    let dest: u32 = 0x0001_0000;
    bus.write32::<BusRead>(dest, 0xCAFE_BABE);
    bus.write32::<BusRead>(D3_MADR, dest);
    bus.write32::<BusRead>(D3_BCR, 1);
    bus.write32::<BusRead>(DPCR, 0x0765_4321 | (1 << 15));
    bus.write32::<BusRead>(D3_CHCR, 0x1100_0000);

    let kept = bus.read32::<BusRead>(dest);
    assert_eq!(
        kept, 0xCAFE_BABE,
        "RAM inalterada: DMA3 nao disparou sem BFRD"
    );
}

#[test]
fn dma3_nao_dispara_sem_bit24() {
    let mut bus = bus();
    insert_stub_disc(&mut bus);
    setloc(&mut bus, 0x02, 0x10, 0x00);
    read_n_and_int1(&mut bus);
    hchpctl_write(&mut bus, 0x80);

    let dest: u32 = 0x0001_0000;
    bus.write32::<BusRead>(dest, 0xDEAD_BEEF);
    bus.write32::<BusRead>(D3_MADR, dest);
    bus.write32::<BusRead>(D3_BCR, 1);
    bus.write32::<BusRead>(DPCR, 0x0765_4321 | (1 << 15));
    bus.write32::<BusRead>(D3_CHCR, 0x1000_0000);

    let kept = bus.read32::<BusRead>(dest);
    assert_eq!(
        kept, 0xDEAD_BEEF,
        "RAM inalterada: DMA3 nao disparou sem bit 24"
    );
}

#[test]
fn dma3_nao_dispara_sem_bit28() {
    let mut bus = bus();
    insert_stub_disc(&mut bus);
    setloc(&mut bus, 0x02, 0x10, 0x00);
    read_n_and_int1(&mut bus);
    hchpctl_write(&mut bus, 0x80);

    let dest: u32 = 0x0001_0000;
    bus.write32::<BusRead>(dest, 0xBAAD_F00D);
    bus.write32::<BusRead>(D3_MADR, dest);
    bus.write32::<BusRead>(D3_BCR, 1);
    bus.write32::<BusRead>(DPCR, 0x0765_4321 | (1 << 15));
    bus.write32::<BusRead>(D3_CHCR, 0x0100_0000);

    let kept = bus.read32::<BusRead>(dest);
    assert_eq!(
        kept, 0xBAAD_F00D,
        "RAM inalterada: DMA3 nao disparou com so bit 24, sem bit 28"
    );
}

#[test]
fn dma3_bcr_zero_equivale_a_10000h_words() {
    let mut bus = bus();
    insert_stub_disc(&mut bus);
    setloc(&mut bus, 0x02, 0x10, 0x00);
    read_n_and_int1(&mut bus);
    hchpctl_write(&mut bus, 0x80);

    let dest: u32 = 0x0001_0000;
    bus.write32::<BusRead>(D3_MADR, dest);
    bus.write32::<BusRead>(D3_BCR, 0);
    bus.write32::<BusRead>(DPCR, 0x0765_4321 | (1 << 15));
    bus.write32::<BusRead>(D3_CHCR, 0x1100_0000);

    let w0 = bus.read32::<BusRead>(dest);
    assert_eq!(w0, 0x0403_0201, "primeiro word transferido com BCR=0");

    let chcr = bus.read32::<BusRead>(D3_CHCR);
    assert_eq!(
        chcr & (1 << 24),
        0,
        "bit 24 limpo — transferencia executou com BCR=0"
    );
}
