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
    let displacement = i32::from((instr & 0xFFFF) as i16).wrapping_mul(4) as u32;
    Some(pc.wrapping_add(4).wrapping_add(displacement))
}

#[test]
fn deslocamento_de_branch_e_sinalizado() {
    assert_eq!(
        beq_target(0x801B_8EA0, 0x1040_003C, 0, 0),
        Some(0x801B_8F94)
    );
    assert_eq!(
        beq_target(0x801B_8EA0, 0x1040_FFFC, 0, 0),
        Some(0x801B_8E94)
    );
}

#[test]
fn entrada_do_hook_refuta_r2_e_localiza_desvio_por_i_stat() {
    assert_eq!(ENTRY_V0.len(), ENTRY_I_STAT.len());
    assert_eq!(ENTRY_V0.len(), ENTRY_I_MASK.len());

    let com_irq_habilitada = ENTRY_I_STAT
        .iter()
        .zip(ENTRY_I_MASK)
        .filter(|(i_stat, i_mask)| *i_stat & *i_mask != 0)
        .count();
    assert_eq!(
        com_irq_habilitada, 5,
        "das 20 ativacoes medidas do hook, so 5 tinham alguma IRQ habilitada pendente: o hook \
         roda 15 vezes com I_STAT & I_MASK == 0, ou seja sem causa nenhuma para tratar"
    );

    for (index, ((v0, i_stat), i_mask)) in ENTRY_V0
        .iter()
        .zip(ENTRY_I_STAT)
        .zip(ENTRY_I_MASK)
        .enumerate()
    {
        assert_eq!(
            *v0, 1,
            "entrada {index}: a spec (13-kernel-bios.md L1480) exige r2=1 na chamada do hook"
        );
        let pendente = u32::from(i_stat & i_mask);
        let alvo = beq_target(0x801B_8EA0, 0x1040_003C, pendente, 0);
        if pendente == 0 {
            assert_eq!(
                alvo,
                Some(0x801B_8F94),
                "entrada {index}: sem IRQ pendente o beq de 0x801B8EA0 desvia para fora do \
                 caminho do contador"
            );
        } else {
            assert_eq!(
                alvo, None,
                "entrada {index}: com IRQ pendente o beq de 0x801B8EA0 NAO deve ser tomado"
            );
        }
    }
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
