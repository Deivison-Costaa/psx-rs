mod support;

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn pwsh_disponivel() -> bool {
    Command::new("pwsh")
        .arg("-NoProfile")
        .arg("-Command")
        .arg("$true")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn temp_dir_unico(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("relogio")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("psx-oraculo-tty-{tag}-{nanos}"));
    fs::create_dir_all(&dir).expect("criar diretorio temporario");
    dir
}

/// Chama `Get-TtyVeredito` de `scripts/lib/tty-veredito.ps1` com strings sinteticas,
/// sem depender de BIOS nem de EXE real. `gabarito = None` simula o gabarito ausente
/// (arquivo `psx.log` que nao existe ao lado do EXE).
fn chama_get_tty_veredito(real: &str, gabarito: Option<&str>) -> (String, String) {
    let lib = support::repo_root().join("scripts/lib/tty-veredito.ps1");
    let dir = temp_dir_unico("chamada");

    let real_path = dir.join("real.txt");
    fs::write(&real_path, real).expect("escrever real.txt");

    let gabarito_path = dir.join("gabarito.txt");
    if let Some(g) = gabarito {
        fs::write(&gabarito_path, g).expect("escrever gabarito.txt");
    }

    let wrapper = dir.join("wrapper.ps1");
    let script = format!(
        ". '{lib}'\n\
         $real = Get-Content -Raw -Path '{real_path}' -ErrorAction SilentlyContinue\n\
         if (Test-Path '{gabarito_path}') {{ $gabarito = Get-Content -Raw -Path '{gabarito_path}' }} else {{ $gabarito = $null }}\n\
         $r = Get-TtyVeredito -Real $real -Gabarito $gabarito\n\
         Write-Output \"$($r.Status)|$($r.Detalhe)\"\n",
        lib = lib.display(),
        real_path = real_path.display(),
        gabarito_path = gabarito_path.display(),
    );
    fs::write(&wrapper, script).expect("escrever wrapper.ps1");

    let output = Command::new("pwsh")
        .arg("-NoProfile")
        .arg("-File")
        .arg(&wrapper)
        .output()
        .expect("executar pwsh");

    let _ = fs::remove_dir_all(&dir);

    assert!(
        output.status.success(),
        "pwsh falhou ao rodar Get-TtyVeredito: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let linha = stdout
        .lines()
        .next_back()
        .expect("Get-TtyVeredito deve imprimir uma linha 'Status|Detalhe'");
    let (status, detalhe) = linha
        .split_once('|')
        .expect("saida deve estar no formato 'Status|Detalhe'");
    (status.to_string(), detalhe.to_string())
}

macro_rules! skip_sem_pwsh {
    () => {
        if !pwsh_disponivel() {
            eprintln!("SKIP: pwsh nao encontrado no PATH -- ambiente sem PowerShell Core");
            return;
        }
    };
}

#[test]
fn saida_identica_classifica_como_identico() {
    skip_sem_pwsh!();
    let (status, detalhe) = chama_get_tty_veredito("a\nb\nc", Some("a\nb\nc"));
    assert_eq!(status, "identico");
    assert_eq!(detalhe, "0/3");
}

#[test]
fn saida_com_k_linhas_diferentes_reporta_k_de_m() {
    skip_sem_pwsh!();
    let (status, detalhe) = chama_get_tty_veredito("a\nX\nc\nd", Some("a\nb\nc\nY"));
    assert_eq!(status, "difere");
    assert_eq!(detalhe, "2/4");
}

#[test]
fn saida_vazia_classifica_como_sem_saida() {
    skip_sem_pwsh!();
    let (status, _detalhe) = chama_get_tty_veredito("", Some("a\nb"));
    assert_eq!(status, "sem-saida");
}

#[test]
fn gabarito_ausente_nao_e_confundido_com_diferenca() {
    skip_sem_pwsh!();
    let (status, _detalhe) = chama_get_tty_veredito("qualquer coisa", None);
    assert_eq!(status, "sem-gabarito");
}

#[test]
fn normaliza_crlf_antes_de_comparar() {
    skip_sem_pwsh!();
    let (status, detalhe) = chama_get_tty_veredito("a\r\nb\r\nc", Some("a\nb\nc"));
    assert_eq!(
        status, "identico",
        "CRLF vs LF devia ser normalizado antes do diff"
    );
    assert_eq!(detalhe, "0/3");
}

#[test]
fn arreio_ps1_existe_e_usa_a_biblioteca_de_veredito() {
    let script = fs::read_to_string(support::repo_root().join("scripts/oraculo-tty.ps1"))
        .expect("scripts/oraculo-tty.ps1 deve existir");
    assert!(
        script.contains("tty-veredito.ps1"),
        "scripts/oraculo-tty.ps1 deve dot-source scripts/lib/tty-veredito.ps1 para reusar \
         a classificacao testada em isolamento (sem duplicar a logica de diff)."
    );
}

#[test]
fn arreio_varre_ps1_tests_em_busca_de_psx_log() {
    let script = fs::read_to_string(support::repo_root().join("scripts/oraculo-tty.ps1"))
        .expect("scripts/oraculo-tty.ps1 deve existir");
    assert!(
        script.contains("psx.log"),
        "scripts/oraculo-tty.ps1 deve procurar por 'psx.log' -- e o gabarito de \
         hardware real ao lado de cada EXE nas 21 suites do ps1-tests."
    );
    assert!(
        script.contains("-Recurse"),
        "scripts/oraculo-tty.ps1 deve varrer tests/exes/ps1-tests recursivamente \
         (-Recurse) para achar os psx.log em subpastas como cpu/cop ou dma/dpcr."
    );
}

#[test]
fn arreio_sem_bios_classifica_sem_bios_e_sai_com_sucesso() {
    let script = fs::read_to_string(support::repo_root().join("scripts/oraculo-tty.ps1"))
        .expect("scripts/oraculo-tty.ps1 deve existir");
    assert!(
        script.contains("sem-bios"),
        "scripts/oraculo-tty.ps1 deve rotular todas as suites como 'sem-bios' quando \
         a BIOS nao existe (repete o defeito do item 10.24 se nao fizer isso)."
    );

    let bloco_sem_bios: String = script
        .lines()
        .skip_while(|l| !l.contains("haveBios"))
        .take(20)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        bloco_sem_bios.contains("exit 0"),
        "scripts/oraculo-tty.ps1 deve sair com 'exit 0' quando a BIOS esta ausente -- \
         a ausencia de material nao pode derrubar a CI (mesma armadilha do item 10.24)."
    );
}

#[test]
fn arreio_usa_max_steps_generoso_por_padrao() {
    let script = fs::read_to_string(support::repo_root().join("scripts/oraculo-tty.ps1"))
        .expect("scripts/oraculo-tty.ps1 deve existir");
    assert!(
        script.contains("800000000"),
        "scripts/oraculo-tty.ps1 deve usar 800000000 como --max-steps padrao (handoff \
         da tarefa 10.23): suites pequenas ainda demoram para emitir todo o TTY."
    );
}

// ===== Alinhamento (item 10.98) =====
// Com o kernel real (0170) o nosso TTY passa a trazer o banner de boot da BIOS, que o gabarito
// do ps1-tests nao tem. Sem alinhar, TODA linha diverge e o K/M vira ruido.

#[test]
fn preambulo_da_bios_e_descartado_antes_de_comparar() {
    if !pwsh_disponivel() {
        return;
    }
    let (status, detalhe) = chama_get_tty_veredito(
        "PS-X Realtime Kernel\nKERNEL SETUP!\npass - um\npass - dois",
        Some("pass - um\npass - dois"),
    );
    assert_eq!(status, "identico", "detalhe={detalhe}");
}

#[test]
fn prefixo_uniforme_do_gabarito_e_removido() {
    if !pwsh_disponivel() {
        return;
    }
    let (status, detalhe) =
        chama_get_tty_veredito("pass - um\npass - dois", Some("% pass - um\n% pass - dois"));
    assert_eq!(status, "identico", "detalhe={detalhe}");
}

#[test]
fn prefixo_nao_uniforme_nao_e_removido() {
    if !pwsh_disponivel() {
        return;
    }
    let (status, _) = chama_get_tty_veredito("pass - um\npass - dois", Some("% pass - um\nx"));
    assert_ne!(
        status, "identico",
        "so remover o prefixo quando TODAS as linhas o tem; senao e chute sobre o formato"
    );
}

#[test]
fn sem_ancora_comum_nao_e_confundido_com_divergencia() {
    if !pwsh_disponivel() {
        return;
    }
    let (status, _) = chama_get_tty_veredito("so banner de boot", Some("pass - um\npass - dois"));
    assert_eq!(
        status, "sem-alinhamento",
        "nenhuma linha nossa casa com o gabarito: reportar isso, nao um K/M inventado"
    );
}
