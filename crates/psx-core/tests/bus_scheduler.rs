use psx_core::bus::{Bios, Bus, BusRead, Ram};
use psx_core::scheduler::{EventId, ScheduleKey, Scheduler};

const BIOS_OK_SIZE: usize = 0x80000;

fn make_bios() -> Bios {
    let mut data = vec![0u8; BIOS_OK_SIZE];
    data[0x0000] = 0x3C;
    data[0x0001] = 0x1F;
    data[0x0004] = 0xAC;
    data[0x0005] = 0x81;
    data[0x0008] = 0x34;
    data[0x0009] = 0x21;
    data[0x000C] = 0x00;
    data[0x000D] = 0x80;
    Bios::from_bytes(data).unwrap()
}

fn make_bus() -> Bus {
    let bios = make_bios();
    let ram = Ram::new();
    Bus::new(ram, bios)
}

// ─── KUSEG ↔ KSEG0 ↔ KSEG1 routing ───

#[test]
fn ram_read_ku0() {
    let mut bus = make_bus();
    bus.write32::<BusRead>(0x0000_0000, 0xDEAD_BEEFu32);
    let val = bus.read32::<BusRead>(0x0000_0000);
    assert_eq!(val, 0xDEAD_BEEF, "KUSEG write/read mesma addr");
}

#[test]
fn ram_kuseg_kseg0_mirror() {
    let mut bus = make_bus();
    bus.write32::<BusRead>(0x0000_1000, 0xAABB_CCDDu32);
    let val = bus.read32::<BusRead>(0x8000_1000);
    assert_eq!(val, 0xAABB_CCDD, "KSEG0 le o escrito em KUSEG");
}

#[test]
fn ram_kuseg_kseg1_mirror() {
    let mut bus = make_bus();
    bus.write32::<BusRead>(0x0000_2000, 0x1122_3344u32);
    let val = bus.read32::<BusRead>(0xA000_2000);
    assert_eq!(val, 0x1122_3344, "KSEG1 le o escrito em KUSEG");
}

#[test]
fn ram_kseg0_kseg1_consistente() {
    let mut bus = make_bus();
    bus.write32::<BusRead>(0x8000_3000, 0x5566_7788u32);
    let via_kuseg = bus.read32::<BusRead>(0x0000_3000);
    let via_kseg1 = bus.read32::<BusRead>(0xA000_3000);
    assert_eq!(via_kuseg, 0x5566_7788, "KUSEG ve escrito em KSEG0");
    assert_eq!(via_kseg1, 0x5566_7788, "KSEG1 ve escrito em KSEG0");
}

#[test]
fn ram_top_of_2mb() {
    let mut bus = make_bus();
    bus.write32::<BusRead>(0x001F_FFFC, 0xFFFF_0000u32);
    let val = bus.read32::<BusRead>(0x801F_FFFC);
    assert_eq!(val, 0xFFFF_0000, "topo dos 2MB RAM via KSEG0");
}

// ─── BIOS read ───

#[test]
fn bios_read_bfc00000() {
    let bus = make_bus();
    let val = bus.read32::<BusRead>(0xBFC0_0000);
    let expected = u32::from_le_bytes([0x3C, 0x1F, 0x00, 0x00]);
    assert_eq!(val, expected, "BIOS first word via KSEG1");
}

#[test]
fn bios_read_9fc00000() {
    let bus = make_bus();
    let val = bus.read32::<BusRead>(0x9FC0_0000);
    let expected = u32::from_le_bytes([0x3C, 0x1F, 0x00, 0x00]);
    assert_eq!(val, expected, "BIOS first word via KSEG0");
}

#[test]
fn bios_read_1fc00000() {
    let bus = make_bus();
    let val = bus.read32::<BusRead>(0x1FC0_0000);
    let expected = u32::from_le_bytes([0x3C, 0x1F, 0x00, 0x00]);
    assert_eq!(val, expected, "BIOS first word via KUSEG (physical)");
}

// ─── Scheduler ───

const CB_A: EventId = EventId(1);
const CB_B: EventId = EventId(2);

#[test]
fn scheduler_eventos_ordem() {
    let mut sched = Scheduler::new();
    sched.schedule(ScheduleKey::new(10), CB_A);
    sched.schedule(ScheduleKey::new(5), CB_B);

    let pendentes = sched.pending_events();
    assert_eq!(pendentes.len(), 2, "dois eventos pendentes");

    // B (ticks=5) deve vir antes de A (ticks=10)
    let ev = sched.advance_to(5);
    assert!(ev == Some(CB_B), "primeiro evento deve ser B (ticks=5)");
    let ev = sched.advance_to(10);
    assert!(ev == Some(CB_A), "segundo evento deve ser A (ticks=10)");
}

#[test]
fn scheduler_ne_avanca_sem_evento() {
    let mut sched = Scheduler::new();
    sched.schedule(ScheduleKey::new(100), CB_A);

    let ev = sched.advance_to(50);
    assert!(ev.is_none(), "nenhum evento antes de 100");
    assert_eq!(sched.pending_events().len(), 1, "evento ainda pendente");
}

#[test]
fn scheduler_evento_imediatamente() {
    let mut sched = Scheduler::new();
    sched.schedule(ScheduleKey::new(0), CB_A);
    let ev = sched.advance_to(0);
    assert!(ev == Some(CB_A), "evento no tick 0 deve disparar");
}

#[test]
fn leitura_de_8_e_16_bits_da_bios_nao_cai_na_ram() {
    let mut bios_bytes = vec![0u8; 0x80000];
    bios_bytes[0] = 0xAA;
    bios_bytes[1] = 0xBB;
    let bios = Bios::from_bytes(bios_bytes).unwrap();
    let mut bus = Bus::new(Ram::new(), bios);
    bus.write8::<BusRead>(0x0000_0000, 0x11);
    bus.write8::<BusRead>(0x0000_0001, 0x22);
    assert_eq!(
        bus.read8::<BusRead>(0xBFC0_0000),
        0xAA,
        "read8 em KSEG1 da BIOS deve vir da ROM, nao da RAM"
    );
    assert_eq!(
        bus.read16::<BusRead>(0xBFC0_0000),
        0xBBAA,
        "read16 em KSEG1 da BIOS deve vir da ROM, nao da RAM"
    );
}
