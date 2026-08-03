mod support;

use psx_core::bus::{BusRead, BusWrite};
use support::asm::bus_with_bios_empty;

// oraculo/timers (JaCzekanski/ps1-tests): as colunas de "Dot clock" e "Hblank"
// vinham erradas em qualquer resolucao != o default hardcoded (10 ciclos/pixel,
// 3413 ciclos/linha de `Timers::new`). `Timers::update_gpu_timing` existe mas
// nenhum caminho de execucao real (bus/CPU) o chama — so os testes unitarios de
// timers_dotclock_hblank.rs o chamam manualmente. `bus.tick_timers` nunca lia a
// resolucao atual da GPU.

const T0_MODE: u32 = 0x1F80_1104;
const T0_CNT: u32 = 0x1F80_1100;
const GP1: u32 = 0x1F80_1814;

fn write_gp1(bus: &mut psx_core::bus::Bus, cmd: u8, param: u32) {
    bus.write32::<BusWrite>(GP1, ((cmd as u32) << 24) | (param & 0x00FF_FFFF));
}

#[test]
fn tick_timers_le_a_resolucao_real_da_gpu_em_vez_do_default() {
    let mut bus = bus_with_bios_empty();

    // GP1(08h) com HR1=1 (320px) -> cycles_per_pix=8 (docs/reference/03-gpu.md
    // L851-854), diferente do default de 10 (256px) usado por `Timers::new`.
    write_gp1(&mut bus, 0x08, 0b0_0001);

    bus.write32::<BusRead>(T0_MODE, 0x0100); // Timer0, clock_src=dotclock
    bus.tick_timers(200);

    assert_eq!(
        bus.read32::<BusRead>(T0_CNT) & 0xFFFF,
        39,
        "200 ciclos de CPU a 11/56 dot/pulso (320px) = 39 pulsos (2200/56=39, resto=16); \
         com o default hardcoded (11/70) daria 31"
    );
}
