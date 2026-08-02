#[derive(Clone, Copy)]
struct HookEntry {
    cause: u32,
    i_stat: u16,
    i_mask: u16,
}

const FIRST_TWENTY: [HookEntry; 20] = [
    HookEntry {
        cause: 0x0000_0400,
        i_stat: 0x0001,
        i_mask: 0x000D,
    },
    HookEntry {
        cause: 0x0000_0400,
        i_stat: 0x0008,
        i_mask: 0x000D,
    },
    HookEntry {
        cause: 0x0000_0400,
        i_stat: 0x0008,
        i_mask: 0x000D,
    },
    HookEntry {
        cause: 0x0000_0000,
        i_stat: 0,
        i_mask: 0x000D,
    },
    HookEntry {
        cause: 0xC000_0000,
        i_stat: 0,
        i_mask: 0x000D,
    },
    HookEntry {
        cause: 0x0000_0000,
        i_stat: 0,
        i_mask: 0x008D,
    },
    HookEntry {
        cause: 0xC000_0000,
        i_stat: 0,
        i_mask: 0x000D,
    },
    HookEntry {
        cause: 0x0000_0000,
        i_stat: 0,
        i_mask: 0x008D,
    },
    HookEntry {
        cause: 0x0000_0000,
        i_stat: 0,
        i_mask: 0x000D,
    },
    HookEntry {
        cause: 0x0000_0000,
        i_stat: 0,
        i_mask: 0x008D,
    },
    HookEntry {
        cause: 0x0000_0000,
        i_stat: 0,
        i_mask: 0x000D,
    },
    HookEntry {
        cause: 0x0000_0400,
        i_stat: 0x0004,
        i_mask: 0x000D,
    },
    HookEntry {
        cause: 0x0000_0400,
        i_stat: 0x0004,
        i_mask: 0x000D,
    },
    HookEntry {
        cause: 0x0000_0000,
        i_stat: 0,
        i_mask: 0x000D,
    },
    HookEntry {
        cause: 0xC000_0000,
        i_stat: 0,
        i_mask: 0x000D,
    },
    HookEntry {
        cause: 0x0000_0000,
        i_stat: 0,
        i_mask: 0x000D,
    },
    HookEntry {
        cause: 0x0000_0000,
        i_stat: 0,
        i_mask: 0x000D,
    },
    HookEntry {
        cause: 0x0000_0000,
        i_stat: 0,
        i_mask: 0x000D,
    },
    HookEntry {
        cause: 0x0000_0000,
        i_stat: 0,
        i_mask: 0x000D,
    },
    HookEntry {
        cause: 0x0000_0000,
        i_stat: 0,
        i_mask: 0x000D,
    },
];

fn exc_code(cause: u32) -> u32 {
    (cause >> 2) & 0x1F
}

fn store_address(base: u32, instr: u32) -> u32 {
    base.wrapping_add((instr as i16 as i32) as u32)
}

#[test]
fn entradas_do_hook_classificam_syscalls_e_interrupcao() {
    let syscall_count = FIRST_TWENTY
        .iter()
        .filter(|entry| exc_code(entry.cause) == 0x08)
        .count();
    assert!(
        syscall_count == 0,
        "nenhuma das 20 entradas medidas e syscall: CAUSE.ExcCode=08h; docs/reference/02-cpu.md L676-L698"
    );
    assert!(
        FIRST_TWENTY.iter().all(|entry| exc_code(entry.cause) == 0),
        "as 20 entradas medidas preservam ExcCode=00h (INT; docs/reference/02-cpu.md L689)"
    );

    assert_eq!(
        exc_code(FIRST_TWENTY[0].cause),
        0,
        "a entrada 0 com VBlank pendente deve ter ExcCode=00h (INT; docs/reference/02-cpu.md L689)"
    );
    assert_eq!(
        FIRST_TWENTY[0].i_stat & FIRST_TWENTY[0].i_mask,
        1,
        "a entrada 0 deve preservar a evidencia de VBlank pendente"
    );
    let vblank_count = FIRST_TWENTY
        .iter()
        .filter(|entry| entry.i_stat & entry.i_mask & 1 != 0)
        .count();
    assert_eq!(
        vblank_count, 1,
        "a amostra das 20 entradas tem exatamente um VBlank habilitado pendente"
    );
    assert_eq!(
        store_address(0x801D_0000, 0xAC22_F2CC),
        0x801C_F2CC,
        "sw 0xAC22F2CC com base 0x801D0000 usa imediato com sinal, conforme [rs+imm] em docs/reference/02-cpu.md L202-L203"
    );
    assert_ne!(
        store_address(0x801D_0000, 0xAC22_F2CC),
        0x801D_F2CC,
        "o PC 0x801B8C50 nao escreve o endereco alegado 0x801DF2CC"
    );
}
