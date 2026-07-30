use psx_core::bus::{Bios, Bus, BusRead, Ram};
use psx_core::gpu::Gpu;

fn bus_zerado() -> Bus {
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS de teste");
    Bus::new(Ram::new(), bios)
}

const GP0: u32 = 0xBF80_1810;
const GP1: u32 = 0xBF80_1814;

#[test]
fn apos_reset_gpustat_15_nao_reflete_e1h_bit11() {
    let mut bus = bus_zerado();

    bus.write32::<BusRead>(GP0, (0xE1 << 24) | 0xFFF);
    let stat = bus.read32::<BusRead>(GP1);

    assert_eq!(
        (stat >> 15) & 1,
        0,
        "GP1(09h).0=0 apos reset → GPUSTAT.15 deve ser 0 mesmo com E1h.11=1"
    );
}

#[test]
fn gp1_09h_abre_o_gate_do_bit_15() {
    let mut bus = bus_zerado();

    bus.write32::<BusRead>(GP1, (0x09 << 24) | 0x01);
    bus.write32::<BusRead>(GP0, (0xE1 << 24) | 0x800);
    let stat = bus.read32::<BusRead>(GP1);

    assert_eq!(
        (stat >> 15) & 1,
        1,
        "GP1(09h).0=1 → GPUSTAT.15 reflete E1h.11=1"
    );
}

#[test]
fn gp1_09h_bit0_zero_proibe_gpustat_15() {
    let mut bus = bus_zerado();

    bus.write32::<BusRead>(GP1, (0x09 << 24) | 0x01);
    bus.write32::<BusRead>(GP0, (0xE1 << 24) | 0x800);
    bus.write32::<BusRead>(GP1, 0x09 << 24);
    let stat = bus.read32::<BusRead>(GP1);

    assert_eq!(
        (stat >> 15) & 1,
        0,
        "GP1(09h).0=0 → fecha o gate e GPUSTAT.15 volta a 0"
    );
}

#[test]
fn gp1_00h_reset_fecha_o_gate_do_bit_15() {
    let mut bus = bus_zerado();

    bus.write32::<BusRead>(GP1, (0x09 << 24) | 0x01);
    bus.write32::<BusRead>(GP1, 0x00 << 24);
    bus.write32::<BusRead>(GP0, (0xE1 << 24) | 0x800);
    let stat = bus.read32::<BusRead>(GP1);

    assert_eq!(
        (stat >> 15) & 1,
        0,
        "GP1(00h) reset → gate fecha e GPUSTAT.15=0 mesmo com E1h.11=1"
    );
}

#[test]
fn e1h_com_gate_aberto_mantem_bits_0_10() {
    let mut bus = bus_zerado();

    bus.write32::<BusRead>(GP1, (0x09 << 24) | 0x01);
    bus.write32::<BusRead>(GP0, (0xE1 << 24) | 0x7FF | 0x800);
    let stat = bus.read32::<BusRead>(GP1);

    assert_eq!(stat & 0xF, 0xF, "GPUSTAT.0-3 preservados");
    assert_eq!((stat >> 4) & 1, 1, "GPUSTAT.4 preservado");
    assert_eq!((stat >> 5) & 3, 3, "GPUSTAT.5-6 preservados");
    assert_eq!((stat >> 7) & 3, 3, "GPUSTAT.7-8 preservados");
    assert_eq!((stat >> 9) & 1, 1, "GPUSTAT.9 preservado");
    assert_eq!((stat >> 10) & 1, 1, "GPUSTAT.10 preservado");
    assert_eq!((stat >> 15) & 1, 1, "GPUSTAT.15=1 (gate aberto, bit11=1)");
}

#[test]
fn e1h_com_gate_fechado_ainda_escreve_bits_0_10() {
    let mut bus = bus_zerado();

    bus.write32::<BusRead>(GP0, (0xE1 << 24) | 0x7FF | 0x800);
    let stat = bus.read32::<BusRead>(GP1);

    assert_eq!(stat & 0xF, 0xF, "GPUSTAT.0-3=0xF (bits 0-10 funcionam)");
    assert_eq!((stat >> 4) & 1, 1, "GPUSTAT.4=1 (bit 4 funciona)");
    assert_eq!((stat >> 5) & 3, 3, "GPUSTAT.5-6=3 (funciona)");
    assert_eq!((stat >> 7) & 3, 3, "GPUSTAT.7-8=3 (funciona)");
    assert_eq!((stat >> 9) & 1, 1, "GPUSTAT.9=1 (funciona)");
    assert_eq!((stat >> 10) & 1, 1, "GPUSTAT.10=1 (funciona)");
    assert_eq!(
        (stat >> 15) & 1,
        0,
        "GPUSTAT.15=0 (gate fechado ignora bit11)"
    );
}

#[test]
fn comando_e1h_sozinho_nao_altera_gpustat_15() {
    let mut bus = bus_zerado();

    bus.write32::<BusRead>(GP0, (0xE1 << 24) | 0x800);
    let stat = bus.read32::<BusRead>(GP1);

    assert_eq!(
        (stat >> 15) & 1,
        0,
        "escrever E1h com bit11=1 e gate fechado: GPUSTAT.15=0"
    );
}

#[test]
fn poligono_texturizado_com_gate_fechado_nao_seta_gpustat_15() {
    let mut gpu = Gpu::new();

    gpu.write32(0, 0x24 << 24);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0080_0000);
    gpu.write32(0, 0x0001_0001);
    gpu.write32(0, 0x0800_0000);
    gpu.write32(0, 0x0002_0002);
    gpu.write32(0, 0x0000_0000);

    let stat = gpu.read32(4);
    assert_eq!(
        (stat >> 15) & 1,
        0,
        "texpage do poligono texturizado com gate fechado: GPUSTAT.15=0"
    );
}

#[test]
fn poligono_texturizado_com_gate_aberto_seta_gpustat_15() {
    let mut gpu = Gpu::new();

    gpu.write32(4, (0x09 << 24) | 0x01);
    gpu.write32(0, 0x24 << 24);
    gpu.write32(0, 0x0000_0000);
    gpu.write32(0, 0x0080_0000);
    gpu.write32(0, 0x0001_0001);
    gpu.write32(0, 0x0800_0000);
    gpu.write32(0, 0x0002_0002);
    gpu.write32(0, 0x0000_0000);

    let stat = gpu.read32(4);
    assert_eq!(
        (stat >> 15) & 1,
        1,
        "texpage do poligono texturizado com gate aberto: GPUSTAT.15=1"
    );
}
