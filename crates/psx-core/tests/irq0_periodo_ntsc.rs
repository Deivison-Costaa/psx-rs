use psx_core::bus::{Bios, Bus, Ram};

fn bus_ntsc() -> Bus {
    let bios = Bios::from_bytes(vec![0; 0x80000]).expect("BIOS sintetica valida");
    Bus::new(Ram::new(), bios)
}

#[test]
fn irq0_ntsc_mantem_periodo_de_um_frame_em_ciclos() {
    let mut bus = bus_ntsc();
    let mut contagem_anterior = bus.irq().raise_count(0);
    let mut subidas = Vec::new();

    for _ in 0..(566_187 * 3) {
        bus.tick_timers(1);
        let contagem_atual = bus.irq().raise_count(0);
        if contagem_atual > contagem_anterior {
            subidas.push(bus.total_cycles());
            contagem_anterior = contagem_atual;
        }
    }

    assert_eq!(
        subidas.len(),
        3,
        "tres frames devem produzir tres subidas de IRQ0"
    );
    assert_eq!(subidas[1] - subidas[0], 566_187);
    assert_eq!(subidas[2] - subidas[1], 566_187);
}
