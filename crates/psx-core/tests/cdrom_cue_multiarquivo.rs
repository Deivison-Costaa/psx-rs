use psx_core::cdrom_bin_cue::parse_cue;

// Uma imagem rasgada trilha a trilha tem um FILE por TRACK, e cada INDEX volta a 00:00:00
// porque e relativo ao proprio arquivo. O `.cue` de um arquivo so guarda deslocamento absoluto.
// Guardar um `bin_path` unico faz o ultimo FILE sobrescrever todos: no Rayman, a trilha de dados
// passava a ser lida do arquivo da trilha 6, e o boot morria em `boot file : cdrom:PSX.EXE;1`.
const CUE_MULTI: &str = r#"FILE "jogo (Track 01).bin" BINARY
  TRACK 01 MODE2/2352
    INDEX 01 00:00:00
FILE "jogo (Track 02).bin" BINARY
  TRACK 02 AUDIO
    INDEX 00 00:00:00
    INDEX 01 00:02:00
FILE "jogo (Track 03).bin" BINARY
  TRACK 03 AUDIO
    INDEX 00 00:00:00
    INDEX 01 00:02:00
"#;

const CUE_UNICO: &str = r#"FILE "jogo.bin" BINARY
  TRACK 01 MODE2/2352
    INDEX 01 00:00:00
  TRACK 02 AUDIO
    INDEX 01 05:00:00
"#;

#[test]
fn cada_trilha_lembra_o_arquivo_de_onde_veio() {
    let layout = parse_cue(CUE_MULTI);
    let arquivos: Vec<&str> = layout.tracks.iter().map(|t| t.file.as_str()).collect();
    assert_eq!(
        arquivos,
        vec![
            "jogo (Track 01).bin",
            "jogo (Track 02).bin",
            "jogo (Track 03).bin"
        ],
        "cada TRACK pertence ao FILE que a precede"
    );
}

#[test]
fn cue_de_arquivo_unico_repete_o_mesmo_arquivo_em_todas_as_trilhas() {
    let layout = parse_cue(CUE_UNICO);
    assert_eq!(layout.tracks.len(), 2);
    for t in &layout.tracks {
        assert_eq!(
            t.file, "jogo.bin",
            "duas TRACKs sob um FILE so moram no mesmo arquivo"
        );
    }
}

#[test]
fn arquivos_saem_na_ordem_das_trilhas_e_sem_repetir() {
    assert_eq!(
        parse_cue(CUE_MULTI).arquivos_em_ordem(),
        vec![
            "jogo (Track 01).bin",
            "jogo (Track 02).bin",
            "jogo (Track 03).bin"
        ],
        "a concatenacao na ordem das trilhas reconstroi a imagem por LBA absoluto"
    );
    assert_eq!(
        parse_cue(CUE_UNICO).arquivos_em_ordem(),
        vec!["jogo.bin"],
        "arquivo repetido entre trilhas entra uma vez so"
    );
}

// `bin_path` continua sendo o primeiro arquivo: quem so sabe ler imagem unica nao regride.
#[test]
fn bin_path_aponta_para_o_primeiro_arquivo() {
    assert_eq!(parse_cue(CUE_MULTI).bin_path, "jogo (Track 01).bin");
    assert_eq!(parse_cue(CUE_UNICO).bin_path, "jogo.bin");
}

// O INDEX de um cue por arquivo e relativo ao PROPRIO arquivo. Sem somar o tamanho dos
// anteriores, toda trilha de audio parece comecar em 00:02:00 e a fronteira entre elas some —
// foi o que fez o autopause do Rayman nunca disparar mesmo com as trilhas presentes.
#[test]
fn lba_absoluto_soma_os_arquivos_anteriores() {
    let mut layout = parse_cue(CUE_MULTI);
    layout.atribui_lbas_absolutos(&[41_685, 850, 5_208]);
    let lbas: Vec<u32> = layout.tracks.iter().map(|t| t.start_lba).collect();
    assert_eq!(
        lbas,
        vec![0, 41_685 + 150, 41_685 + 850 + 150],
        "cada trilha comeca depois dos arquivos anteriores, mais o seu proprio INDEX 01"
    );
}

#[test]
fn cue_de_arquivo_unico_mantem_o_lba_do_index() {
    let mut layout = parse_cue(CUE_UNICO);
    layout.atribui_lbas_absolutos(&[100_000]);
    let lbas: Vec<u32> = layout.tracks.iter().map(|t| t.start_lba).collect();
    assert_eq!(
        lbas,
        vec![0, 5 * 60 * 75],
        "num arquivo so o INDEX 01 ja e absoluto e nada deve ser somado"
    );
}
