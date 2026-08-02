#[derive(Clone, Copy)]
struct HookEntry {
    cause: u32,
    i_stat: u16,
    i_mask: u16,
}

const FIRST_TWENTY: [HookEntry; 20] = [
    HookEntry {
        cause: 0,
        i_stat: 0x0001,
        i_mask: 0x000D,
    },
    HookEntry {
        cause: 0,
        i_stat: 0x0008,
        i_mask: 0x000D,
    },
    HookEntry {
        cause: 0,
        i_stat: 0x0008,
        i_mask: 0x000D,
    },
    HookEntry {
        cause: 0,
        i_stat: 0,
        i_mask: 0x000D,
    },
    HookEntry {
        cause: 0,
        i_stat: 0,
        i_mask: 0x000D,
    },
    HookEntry {
        cause: 0,
        i_stat: 0,
        i_mask: 0x008D,
    },
    HookEntry {
        cause: 0,
        i_stat: 0,
        i_mask: 0x000D,
    },
    HookEntry {
        cause: 0,
        i_stat: 0,
        i_mask: 0x008D,
    },
    HookEntry {
        cause: 0,
        i_stat: 0,
        i_mask: 0x000D,
    },
    HookEntry {
        cause: 0,
        i_stat: 0,
        i_mask: 0x008D,
    },
    HookEntry {
        cause: 0,
        i_stat: 0,
        i_mask: 0x000D,
    },
    HookEntry {
        cause: 0,
        i_stat: 0x0004,
        i_mask: 0x000D,
    },
    HookEntry {
        cause: 0,
        i_stat: 0x0004,
        i_mask: 0x000D,
    },
    HookEntry {
        cause: 0,
        i_stat: 0,
        i_mask: 0x000D,
    },
    HookEntry {
        cause: 0,
        i_stat: 0,
        i_mask: 0x000D,
    },
    HookEntry {
        cause: 0,
        i_stat: 0,
        i_mask: 0x000D,
    },
    HookEntry {
        cause: 0,
        i_stat: 0,
        i_mask: 0x000D,
    },
    HookEntry {
        cause: 0,
        i_stat: 0,
        i_mask: 0x000D,
    },
    HookEntry {
        cause: 0,
        i_stat: 0,
        i_mask: 0x000D,
    },
    HookEntry {
        cause: 0,
        i_stat: 0,
        i_mask: 0x000D,
    },
];

fn exc_code(cause: u32) -> u32 {
    (cause >> 2) & 0x1F
}

#[test]
fn entradas_do_hook_classificam_syscalls_e_interrupcao() {
    let syscall_count = FIRST_TWENTY
        .iter()
        .filter(|entry| exc_code(entry.cause) == 0x08)
        .count();
    assert!(
        syscall_count >= 15,
        "a maioria das 20 entradas deve ser syscall (CAUSE.ExcCode=08h; docs/reference/02-cpu.md L676-L698), nao IRQ"
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
}
