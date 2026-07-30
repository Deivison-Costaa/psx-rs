use psx_core::bus::{Bios, Bus, BusRead, Ram};
use psx_core::gpu::Gpu;

fn make_bus() -> Bus {
    let mut data = vec![0u8; 0x80000];
    data[0x0000] = 0x3C;
    data[0x0001] = 0x1F;
    let bios = Bios::from_bytes(data).unwrap();
    Bus::new(Ram::new(), bios)
}

#[test]
fn t1_gpu_total_scanlines_ntsc() {
    let gpu = Gpu::new();
    assert_eq!(
        gpu.total_scanlines(),
        263,
        "T1: total_scanlines NTSC deve ser 263"
    );
}

#[test]
fn t2_gpu_total_scanlines_pal() {
    let mut gpu = Gpu::new();
    gpu.write32(4, 0x08_000008);
    assert_eq!(
        gpu.total_scanlines(),
        314,
        "T2: total_scanlines PAL deve ser 314"
    );
}

#[test]
fn t3_bus_agenda_evento_de_vblank() {
    let bus = make_bus();
    let n = bus.scheduler_pending_count();
    assert!(
        n >= 2,
        "T3: scheduler deve ter pelo menos 2 eventos de vblank, tem {n}"
    );
}

#[test]
fn t4_bus_levanta_irq0_em_vblank_enter() {
    let mut bus = make_bus();

    for _ in 0..1_000_000 {
        bus.tick_timers(1);
    }

    let irq_stat = bus.read32::<BusRead>(0x1F80_1070);
    let bit0 = irq_stat & 1;
    assert_eq!(
        bit0, 1,
        "T4: IRQ0 (I_STAT bit0) deve ser 1 apos ~1M ciclos; I_STAT={irq_stat:#010X}"
    );
}

#[test]
fn t5_gpu_vblank_active_durante_evento() {
    let mut bus = make_bus();

    let enter_cycle = bus.gpu().frame_cycles() * bus.gpu().display_range_y2() as u64
        / bus.gpu().total_scanlines() as u64;

    for _ in 0..enter_cycle as usize + 1 {
        bus.tick_timers(1);
    }

    let vb = bus.gpu().vblank_active();
    let irq_stat = bus.read32::<BusRead>(0x1F80_1070);
    let bit0 = irq_stat & 1;
    assert!(
        vb && bit0 == 1,
        "T5: vblank_active e IRQ0 devem estar ativos simultaneamente no ciclo {enter_cycle}; \
         vb={vb}, I_STAT={irq_stat:#010X}"
    );
}

#[test]
fn t6_eventos_de_vblank_repetem() {
    let mut bus = make_bus();

    for _ in 0..2_500_000 {
        bus.tick_timers(1);
    }

    let irq_stat = bus.read32::<BusRead>(0x1F80_1070);
    let bit0 = irq_stat & 1;
    assert_eq!(
        bit0, 1,
        "T6: IRQ0 deve estar ativo novamente apos varios frames; I_STAT={irq_stat:#010X}"
    );
}

#[test]
fn t7_vblank_nao_fica_preso_em_true() {
    let mut bus = make_bus();

    for _ in 0..800_000 {
        bus.tick_timers(1);
    }

    assert!(
        !bus.gpu().vblank_active(),
        "T7: vblank_active deve ser false apos passar pela regiao de vblank; \
         ~800k ciclos deve estar no meio da area visivel do primeiro frame"
    );
}

#[test]
fn t8_total_scanlines_consistente_com_frame_cycles() {
    let gpu = Gpu::new();
    let total_sl = gpu.total_scanlines() as u64;
    let frame = gpu.frame_cycles();
    let cycles_per_sl = gpu.cpu_cycles_per_scanline();
    let reconstituted = cycles_per_sl * total_sl;
    let drift = frame.saturating_sub(reconstituted);
    assert!(
        drift <= total_sl,
        "T8: drift entre frame_cycles e reconstituicao via cpu_cycles_per_scanline \
         deve ser <= total_scanlines; frame={frame}, cp_sl={cycles_per_sl}, \
         total_sl={total_sl}, recon={reconstituted}, drift={drift}"
    );
}

#[test]
fn t9_odd_line_alterna_apos_vblank_exit() {
    let mut bus = make_bus();

    let exit_cycle = bus.gpu().frame_cycles() * bus.gpu().display_range_y1() as u64
        / bus.gpu().total_scanlines() as u64;

    for _ in 0..exit_cycle as usize + 1 {
        bus.tick_timers(1);
    }

    let gpustat = bus.gpu().read32(4);
    let bit31 = (gpustat >> 31) & 1;
    assert_eq!(
        bit31, 1,
        "T9: GPUSTAT bit31 deve ser 1 apos sair do vblank (odd_line toggled); \
         gpustat={gpustat:#010X}"
    );
}

#[test]
fn t10_gpu_vblank_irq_bios_boot_placeholder() {
    eprintln!(
        "T10: teste de aceitacao manual — rodar psx-cli --bios <BIOS> e \
         conferir que o TTY nao contem 'VSync: timeout'"
    );
}
