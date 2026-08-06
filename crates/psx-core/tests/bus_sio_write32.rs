mod support;

use psx_core::bus::{Bus, BusRead, BusWrite};
use support::asm;

const JOY_MODE: u32 = 0x1F80_1048;
const JOY_CTRL: u32 = 0x1F80_104A;
const JOY_BAUD: u32 = 0x1F80_104E;

#[test]
fn write32_em_joy_mode_ctrl_e_baud_entrega_os_dois_bytes() {
    let mut bus: Bus = asm::bus_with_bios_empty();

    bus.write32::<BusWrite>(JOY_MODE, 0x0000_1234);
    bus.write32::<BusWrite>(JOY_CTRL, 0x0000_2A03);
    bus.write32::<BusWrite>(JOY_BAUD, 0x0000_9ABC);

    assert_eq!(
        bus.read16::<BusRead>(JOY_MODE),
        0x1234,
        "um `sw` em JOY_MODE deve escrever os bytes 1048h e 1049h"
    );
    assert_eq!(
        bus.read16::<BusRead>(JOY_CTRL),
        0x2A03,
        "um `sw` em JOY_CTRL deve escrever os bytes 104Ah e 104Bh"
    );
    assert_eq!(
        bus.read16::<BusRead>(JOY_BAUD),
        0x9ABC,
        "um `sw` em JOY_BAUD deve escrever os bytes 104Eh e 104Fh"
    );
}
