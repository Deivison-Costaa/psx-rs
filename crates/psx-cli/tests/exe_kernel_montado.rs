use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn bins_dir() -> PathBuf {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests");
    d.push("bins");
    d
}

fn workspace_bios() -> PathBuf {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("..");
    d.push("..");
    d.push("bios");
    d.push("SCPH1001.BIN");
    d
}

fn j(opcode: u32, target_addr: u32) -> u32 {
    (opcode << 26) | ((target_addr >> 2) & 0x03FF_FFFF)
}

fn nop() -> u32 {
    0x0000_0000
}

fn build_ps_exe(code: &[u32], dest_addr: u32, initial_pc: u32) -> Vec<u8> {
    let header_size = 0x800;
    let body_words = code.len();
    let body_size = ((body_words * 4) + 0x7FF) & !0x7FF;

    let mut data = vec![0u8; header_size + body_size];
    data[0..8].copy_from_slice(b"PS-X EXE");

    data[0x10..0x14].copy_from_slice(&initial_pc.to_le_bytes());
    data[0x14..0x18].copy_from_slice(&0u32.to_le_bytes());
    data[0x18..0x1C].copy_from_slice(&dest_addr.to_le_bytes());
    data[0x1C..0x20].copy_from_slice(&(body_size as u32).to_le_bytes());
    data[0x30..0x34].copy_from_slice(&0x801F_FFF0u32.to_le_bytes());
    data[0x34..0x38].copy_from_slice(&0u32.to_le_bytes());

    for (i, &word) in code.iter().enumerate() {
        let pos = header_size + i * 4;
        data[pos..pos + 4].copy_from_slice(&word.to_le_bytes());
    }

    data
}

/// Recolhe as linhas `  ENDERECO: PALAVRA` que `--dump-mem` grava no stderr
/// num mapa endereco->palavra, para ler os valores de volta por asserção.
fn parse_dump_words(stderr: &str) -> HashMap<u32, u32> {
    let mut out = HashMap::new();
    for line in stderr.lines() {
        let line = line.trim();
        let Some((addr_str, word_str)) = line.split_once(':') else {
            continue;
        };
        let addr_str = addr_str.trim();
        let word_str = word_str.trim();
        if addr_str.len() != 8 || word_str.len() != 8 {
            continue;
        }
        if let (Ok(addr), Ok(word)) = (
            u32::from_str_radix(addr_str, 16),
            u32::from_str_radix(word_str, 16),
        ) {
            out.insert(addr, word);
        }
    }
    out
}

/// ROADMAP 10.95: no caminho `--bios` + `--exe`, o kernel tem de estar
/// MONTADO (A0h/B0h/C0h reais, Table of Tables e A-jump-table preenchidas
/// pelo proprio boot da BIOS) no instante em que o PS-EXE ganha o controle,
/// em vez do `install_return_stubs` que grava `jr $ra` puro nesses vetores.
#[test]
fn exe_com_bios_boota_kernel_antes_de_saltar_para_o_psexe() {
    let bios_path = workspace_bios();

    if !bios_path.exists() {
        eprintln!("SKIP: BIOS nao encontrada");
        return;
    }

    let _ = fs::create_dir_all(bins_dir());

    // User RAM comeca em 0x00010000 (docs/reference/13-kernel-bios.md L433):
    // fora da area de kernel (tabelas, ExCB/EvCB/TCB) que o boot acabou de montar.
    let dest_addr: u32 = 0x8001_0000;
    let fingerprint_addr: u32 = dest_addr + 0x100;
    let loop_addr: u32 = dest_addr + 0x14;
    let code = [
        0x3C09_8001, // lui  $t1, 0x8001
        0x3529_0100, // ori  $t1, $t1, 0x0100      ; $t1 = fingerprint_addr
        0x3C08_CAFE, // lui  $t0, 0xCAFE
        0x3508_F00D, // ori  $t0, $t0, 0xF00D      ; $t0 = 0xCAFEF00D
        0xAD28_0000, // sw   $t0, 0($t1)           ; prova que o PS-EXE rodou
        j(0x02, loop_addr),
        nop(),
    ];
    let exe_data = build_ps_exe(&code, dest_addr, dest_addr);

    let exe_path = bins_dir().join("exe_kernel_montado.psexe");
    fs::write(&exe_path, &exe_data).expect("escrever EXE sintetico");

    let bin = env!("CARGO_BIN_EXE_psx-cli");

    let output = Command::new(bin)
        .arg("--bios")
        .arg(&bios_path)
        .arg("--exe")
        .arg(&exe_path)
        .arg("--max-steps")
        .arg("8")
        .arg("--dump-mem")
        .arg("000000A0")
        .arg("10")
        .arg("--dump-mem")
        .arg("000000B0")
        .arg("10")
        .arg("--dump-mem")
        .arg("000000C0")
        .arg("10")
        .arg("--dump-mem")
        .arg("00000100")
        .arg("10")
        .arg("--dump-mem")
        .arg("00000200")
        .arg("10")
        .arg("--dump-mem")
        .arg(format!("{:08X}", fingerprint_addr))
        .arg("4")
        .output()
        .expect("executar psx-cli --bios + --exe com --dump-mem");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "psx-cli deve sair com exit code 0; exit={} stderr:\n{}",
        output.status,
        stderr
    );

    assert!(
        stdout.contains("PS-X Realtime Kernel"),
        "TTY deve conter o banner do kernel: a BIOS precisa ter bootado de \
         verdade antes do sideload, nao so sido lida do arquivo; TTY={:?}",
        stdout
    );

    let words = parse_dump_words(&stderr);
    let word_at = |addr: u32| -> u32 {
        *words.get(&addr).unwrap_or_else(|| {
            panic!(
                "dump nao trouxe a palavra em 0x{:08X}; stderr:\n{}",
                addr, stderr
            )
        })
    };

    let jr_ra: u32 = 0x03E0_0008;

    for (label, addr) in [
        ("A0h", 0x0000_00A0u32),
        ("B0h", 0x0000_00B0u32),
        ("C0h", 0x0000_00C0u32),
    ] {
        let got = word_at(addr);
        assert_ne!(
            got, jr_ra,
            "vetor {} (0x{:08X}) nao pode ser jr $ra puro (install_return_stubs) \
             no caminho --bios + --exe",
            label, addr
        );
        assert_eq!(
            got, 0x3C08_0000,
            "vetor {} (0x{:08X}) deve ser o dispatcher real da BIOS \
             (lui $t0,hi ; addiu $t0,$t0,lo ; jr $t0), nao um stub; obtido=0x{:08X}",
            label, addr, got
        );
    }

    // Table of Tables (docs/reference/13-kernel-bios.md L442-443): ExCB em
    // [0x100]=endereco, [0x104]=tamanho. So existe valor aqui se SysInitMemory
    // rodou de verdade.
    assert_eq!(
        word_at(0x0000_0100),
        0xA000_E004,
        "Table of Tables [0x100] deve conter o endereco real do ExCB montado pelo boot"
    );
    assert_eq!(
        word_at(0x0000_0104),
        0x0000_0020,
        "Table of Tables [0x104] deve conter o tamanho real do ExCB (4*08h)"
    );

    // A(nnh) Jump Table (docs/reference/13-kernel-bios.md L423): primeira
    // entrada tem de ser um endereco de funcao real, nao zero.
    assert_eq!(
        word_at(0x0000_0200),
        0x0000_2958,
        "A-jump-table [0x200] deve conter o endereco real da primeira funcao A(nnh)"
    );

    assert_eq!(
        word_at(fingerprint_addr),
        0xCAFE_F00D,
        "o PS-EXE sideloaded precisa ter ganho o controle e escrito seu fingerprint"
    );

    let _ = fs::remove_file(&exe_path);
}
