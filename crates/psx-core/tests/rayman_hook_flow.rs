const ENTRY_V0: [u32; 20] = [1; 20];
const ENTRY_I_STAT: [u16; 20] = [
    0x0001, 0x0008, 0x0008, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0004,
    0x0004, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
];
const ENTRY_I_MASK: [u16; 20] = [
    0x000D, 0x000D, 0x000D, 0x000D, 0x000D, 0x008D, 0x000D, 0x008D, 0x000D, 0x008D, 0x000D, 0x000D,
    0x000D, 0x000D, 0x000D, 0x000D, 0x000D, 0x000D, 0x000D, 0x000D,
];

fn beq_target(pc: u32, instr: u32, rs: u32, rt: u32) -> Option<u32> {
    if instr >> 26 != 0b000100 || rs != rt {
        return None;
    }
    let displacement = i32::from((instr & 0xFFFF) as u16).wrapping_mul(4) as u32;
    Some(pc.wrapping_add(4).wrapping_add(displacement))
}

#[test]
fn entrada_do_hook_refuta_r2_e_localiza_desvio_por_i_stat() {
    assert!(ENTRY_V0.iter().all(|value| *value == 1));

    let first_without_enabled_irq = ENTRY_I_STAT
        .iter()
        .zip(ENTRY_I_MASK)
        .find(|(i_stat, i_mask)| *i_stat & *i_mask == 0)
        .map(|(i_stat, i_mask)| *i_stat & i_mask)
        .expect("a amostra deve conter I_STAT sem IRQ habilitada");
    assert_eq!(first_without_enabled_irq, 0);

    assert_eq!(
        beq_target(
            0x801B_8EA0,
            0x1040_003C,
            first_without_enabled_irq.into(),
            0
        ),
        Some(0x801B_8F94)
    );
    assert_eq!(
        beq_target(0x801B_8EA0, 0x1040_003C, 1, 0),
        None,
        "I_STAT com bit habilitado nao deve tomar o ramo de ausencia de IRQ"
    );
    assert_eq!(
        beq_target(0x801B_8F0C, 0x1060_000D, 1, 0),
        None,
        "VBlank pendente deve passar pela checagem seguinte"
    );
}
