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

fn pixel_le(dump: &[u8], x: usize, y: usize) -> u16 {
    let off = (y * 1024 + x) * 2;
    u16::from_le_bytes([dump[off], dump[off + 1]])
}

#[test]
fn dump_vram_grava_fill_convertido_para_15bpp() {
    let bios_path = workspace_bios();

    if !bios_path.exists() {
        eprintln!("SKIP: BIOS nao encontrada");
        return;
    }

    let _ = fs::create_dir_all(bins_dir());

    let code_addr: u32 = 0x8000_0000;
    let loop_addr: u32 = 0x8000_0024;
    let code = [
        0x3C08_1F80, // lui  $t0, 0x1F80
        0x3508_1810, // ori  $t0, $t0, 0x1810      ; $t0 = GP0
        0x3C09_0200, // lui  $t1, 0x0200
        0x3529_00FF, // ori  $t1, $t1, 0x00FF      ; GP0(02h) cor 0x0000FF (R=FF)
        0xAD09_0000, // sw   $t1, 0($t0)
        0xAD00_0000, // sw   $zero, 0($t0)         ; destino (0,0)
        0x3C0A_0010, // lui  $t2, 0x0010
        0x354A_0020, // ori  $t2, $t2, 0x0020      ; Ysiz=16, Xsiz=32
        0xAD0A_0000, // sw   $t2, 0($t0)
        j(0x02, loop_addr),
        nop(),
    ];
    let exe_data = build_ps_exe(&code, code_addr, code_addr);

    let exe_path = bins_dir().join("dump_vram_fill.psexe");
    fs::write(&exe_path, &exe_data).expect("escrever EXE sintetico");
    let dump_path = bins_dir().join("dump_vram_fill.bin");
    let _ = fs::remove_file(&dump_path);

    let bin = env!("CARGO_BIN_EXE_psx-cli");

    let output = Command::new(bin)
        .arg("--bios")
        .arg(&bios_path)
        .arg("--exe")
        .arg(&exe_path)
        .arg("--max-steps")
        .arg("64")
        .arg("--dump-vram")
        .arg(&dump_path)
        .output()
        .expect("executar psx-cli com --dump-vram");

    assert!(
        output.status.success(),
        "psx-cli deve aceitar --dump-vram e sair com exit code 0; exit={} stderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let dump = fs::read(&dump_path).expect("ler o dump de VRAM");
    assert_eq!(
        dump.len(),
        1024 * 512 * 2,
        "dump deve ter a VRAM inteira: 512 linhas de 2048 bytes"
    );

    assert_eq!(
        pixel_le(&dump, 0, 0),
        0x001F,
        "fill 24bit 0x0000FF vira 15bit 0x001F (descarta 3 LSB por canal, bit15=0)"
    );
    assert_eq!(
        pixel_le(&dump, 31, 15),
        0x001F,
        "canto inferior-direito da area 32x16 tambem preenchido"
    );
    assert_eq!(
        pixel_le(&dump, 32, 0),
        0x0000,
        "primeira coluna fora da largura do fill fica intacta"
    );
    assert_eq!(
        pixel_le(&dump, 0, 16),
        0x0000,
        "primeira linha abaixo da altura do fill fica intacta"
    );

    let _ = fs::remove_file(&exe_path);
    let _ = fs::remove_file(&dump_path);
}
