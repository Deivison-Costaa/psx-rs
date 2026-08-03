mod support;

use std::collections::HashMap;
use std::fs;

const ACHADOS: &str = "docs/achados.md";
const FECHADOS: &str = "docs/ROADMAP-fechado.md";
const TETO_BYTES: u64 = 24_000;

fn ler(rel: &str) -> String {
    let path = support::repo_root().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} deve existir e ser legivel: {e}"))
}

fn id_do_item(linha: &str) -> Option<(String, bool)> {
    let fechado = linha.starts_with("- [x] ");
    if !fechado && !linha.starts_with("- [ ] ") {
        return None;
    }
    Some((linha[6..].split_whitespace().next()?.to_string(), fechado))
}

#[test]
fn achados_dentro_do_teto() {
    let path = support::repo_root().join(ACHADOS);
    let size = fs::metadata(&path).expect("achados.md deve existir").len();
    assert!(
        size <= TETO_BYTES,
        "{ACHADOS} com {size} bytes (teto {TETO_BYTES}). O teto e generoso de proposito — este \
         arquivo cresce com a medicao — mas nao e infinito: uma linha por achado, contexto em \
         docs/iterations/NNNN-*.md."
    );
}

#[test]
fn achados_so_contem_item_aberto() {
    let fechados: Vec<String> = ler(ACHADOS)
        .lines()
        .filter_map(id_do_item)
        .filter(|(_, f)| *f)
        .map(|(id, _)| id)
        .collect();
    assert!(
        fechados.is_empty(),
        "item fechado ocupando espaco em {ACHADOS}: mova para {FECHADOS}. Achados fechados: \
         {fechados:?}"
    );
}

// A colisao que este teste pega custou renumeracao manual em dois merges na noite de 02-03/08:
// dois lotes paralelos escolheram `10.108` e dois escolheram `10.102`, sem saber um do outro.
#[test]
fn nenhum_identificador_de_achado_se_repete() {
    let texto = ler(ACHADOS);
    let mut vistos: HashMap<String, usize> = HashMap::new();
    for linha in texto.lines() {
        if let Some((id, _)) = id_do_item(linha) {
            *vistos.entry(id).or_insert(0) += 1;
        }
    }
    let repetidos: Vec<(&String, &usize)> = vistos.iter().filter(|(_, n)| **n > 1).collect();
    assert!(
        repetidos.is_empty(),
        "identificador repetido em {ACHADOS}: dois achados com o mesmo numero viram um so na \
         primeira vez que alguem citar. Numere pela iteracao que achou (`NNNN.k`), que nao \
         colide entre rodadas paralelas. Repetidos: {repetidos:?}"
    );
}

#[test]
fn achado_nao_aparece_tambem_no_arquivo_de_fechados() {
    let abertos: Vec<String> = ler(ACHADOS)
        .lines()
        .filter_map(id_do_item)
        .map(|(id, _)| id)
        .collect();
    let arquivados: Vec<String> = ler(FECHADOS)
        .lines()
        .filter_map(id_do_item)
        .map(|(id, _)| id)
        .collect();
    let ambos: Vec<&String> = abertos
        .iter()
        .filter(|id| arquivados.contains(id))
        .collect();
    assert!(
        ambos.is_empty(),
        "achado em dois arquivos ao mesmo tempo: duas fontes de verdade divergem na primeira \
         edicao. Duplicados entre {ACHADOS} e {FECHADOS}: {ambos:?}"
    );
}

// O ponteiro no cabecalho e o que faz alguem achar o arquivo. Sem ele, a escada parece ser tudo
// que existe e os achados viram um deposito que ninguem abre.
#[test]
fn roadmap_aponta_para_os_achados_no_cabecalho() {
    let roadmap = ler("ROADMAP.md");
    let cabecalho = roadmap
        .split_once("\n## ")
        .map(|(antes, _)| antes.to_string())
        .unwrap_or(roadmap);
    assert!(
        cabecalho.contains(ACHADOS),
        "o ponteiro para {ACHADOS} tem de estar no CABECALHO do ROADMAP, antes do primeiro marco"
    );
}
