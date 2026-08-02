use psx_core::bus::{Bios, Bus, BusRead, Ram};
use psx_core::cpu::Cpu;

fn optional_path<'a>(candidates: &'a [&'a str]) -> Option<&'a str> {
    candidates
        .iter()
        .copied()
        .find(|path| std::path::Path::new(path).exists())
}

fn setup() -> Option<(Bus, Cpu)> {
    let bios_path = optional_path(&["bios/SCPH1001.BIN", "../../bios/SCPH1001.BIN"])?;
    let disc_path = optional_path(&[
        "../roms/extraido/Rayman (USA) DADOS.cue",
        "../../roms/extraido/Rayman (USA) DADOS.cue",
        "../../../roms/extraido/Rayman (USA) DADOS.cue",
    ])?;
    let bios = Bios::from_bytes(std::fs::read(bios_path).expect("ler BIOS")).expect("BIOS valida");
    let cue = std::fs::read_to_string(disc_path).expect("ler CUE");
    let layout = psx_core::cdrom_bin_cue::parse_cue(&cue);
    let directory = std::path::Path::new(disc_path)
        .parent()
        .expect("CUE deve ter diretorio");
    let bin = std::fs::read(directory.join(&layout.bin_path)).expect("ler BIN");
    let mut bus = Bus::new(Ram::new(), bios);
    bus.inject_disc(layout, bin);
    bus.cdrom_mut().insert_disc();
    Some((bus, Cpu::new()))
}

const EXE_INICIO: u32 = 0x8012_5000;
const EXE_TAMANHO: u32 = 0x000A_A800;
const HOOK_DO_JOGO: u32 = 0x801B_8E60;
const CONTADOR_VSYNC: u32 = 0x801C_F2CC;

fn indice(texto: &str, agulha: &str) -> usize {
    texto
        .find(agulha)
        .unwrap_or_else(|| panic!("TTY sem {agulha:?}"))
}

#[test]
fn o_jogo_so_executa_depois_do_bootstrap_e_ja_perde_o_vsync() {
    let Some((mut bus, mut cpu)) = setup() else {
        eprintln!("BIOS ou disco Rayman nao encontrado — teste ignorado");
        return;
    };

    let mut tty = String::new();
    let mut passo_do_execute = 0usize;
    let mut passo_do_primeiro_timeout = 0usize;

    for step in 1..=200_000_000usize {
        cpu.step(&mut bus);
        if step % 200_000 == 0 {
            let pedaco = String::from_utf8_lossy(&bus.take_tty()).to_string();
            if passo_do_execute == 0 && pedaco.contains("Execute !") {
                passo_do_execute = step;
            }
            if passo_do_primeiro_timeout == 0 && pedaco.contains("VSync: timeout") {
                passo_do_primeiro_timeout = step;
            }
            tty.push_str(&pedaco);
        }
    }
    tty.push_str(&String::from_utf8_lossy(&bus.take_tty()));

    let bootstrap = indice(&tty, "BOOTSTRAP LOADER");
    let system_cnf = indice(&tty, "cdrom:SYSTEM.CNF;1");
    let segundo_setup = tty.rfind("KERNEL SETUP!").expect("TTY sem KERNEL SETUP");
    let nome_do_boot = indice(&tty, "cdrom:\\SLUS-000.05;1");
    let execute = indice(&tty, "Execute !");
    let pad_driver = indice(&tty, "PS-X Control PAD Driver");
    let timeout = indice(&tty, "VSync: timeout");

    assert!(
        bootstrap < system_cnf && system_cnf < nome_do_boot,
        "o BOOTSTRAP LOADER le o SYSTEM.CNF e so entao conhece o nome do executavel"
    );
    assert!(
        nome_do_boot < segundo_setup && segundo_setup < execute,
        "o segundo KERNEL SETUP e do proprio bootstrap, entre ler o SYSTEM.CNF e executar — nao e reinicializacao no meio do jogo"
    );
    assert!(
        execute < pad_driver && pad_driver < timeout,
        "o driver de pad entra depois do Execute!, e o timeout de VSync depois dele"
    );

    assert_eq!(passo_do_execute, 164_000_000, "amostragem de 200 k passos");
    assert_eq!(passo_do_primeiro_timeout, 167_000_000);

    let ocorrencias = tty.matches("VSync: timeout").count();
    assert!(
        ocorrencias > 100,
        "o VSync do jogo estoura o tempo de forma continua, nao uma vez: {ocorrencias} ocorrencias"
    );

    assert!(tty.contains("T_ADDR(80125000)"));
    let fim = EXE_INICIO + EXE_TAMANHO;
    assert!(
        (EXE_INICIO..fim).contains(&HOOK_DO_JOGO),
        "o hook mora dentro do executavel carregado, entao e codigo do jogo"
    );
    assert!(
        (EXE_INICIO..fim).contains(&CONTADOR_VSYNC),
        "o contador de VSync tambem"
    );
}
