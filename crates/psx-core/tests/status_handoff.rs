mod support;

use std::collections::HashSet;
use std::fs;

const INVARIANTES: &str = "docs/invariantes.md";
const MINIMO_DE_INVARIANTES: usize = 10;

fn ler(rel: &str) -> String {
    let path = support::repo_root().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} deve existir e ser legivel: {e}"))
}

fn secao<'a>(texto: &'a str, titulo: &str) -> Option<&'a str> {
    let cabecalho = format!("## {titulo}");
    let inicio = texto.find(&cabecalho)? + cabecalho.len();
    let resto = &texto[inicio..];
    Some(match resto.find("\n## ") {
        Some(fim) => &resto[..fim],
        None => resto,
    })
}

// Desde a 0181 o item pode morar na escada (`ROADMAP.md`) ou nos achados (`docs/achados.md`),
// e a prosa natural do handoff passou a ser "Achado 0182.2". Exigir so o prefixo "ROADMAP "
// obrigaria a escrever "ROADMAP 0182.2" para um item que nao esta no roadmap.
fn ids_citados(texto: &str) -> Vec<String> {
    let mut saida = Vec::new();
    let pedacos = texto
        .split("ROADMAP ")
        .skip(1)
        .chain(texto.split("Achado ").skip(1));
    for pedaco in pedacos {
        let id: String = pedaco
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.' || c.is_ascii_lowercase())
            .collect();
        if id.contains('.') && id.starts_with(|c: char| c.is_ascii_digit()) {
            saida.push(id.trim_end_matches('.').to_string());
        }
    }
    saida
}

fn ids_existentes() -> HashSet<String> {
    let mut saida = HashSet::new();
    for arquivo in ["ROADMAP.md", "docs/achados.md", "docs/ROADMAP-fechado.md"] {
        for linha in ler(arquivo).lines() {
            if linha.starts_with("- [x] ") || linha.starts_with("- [ ] ") {
                if let Some(id) = linha[6..].split_whitespace().next() {
                    saida.insert(id.to_string());
                }
            }
        }
    }
    saida
}

fn numerados(texto: &str) -> HashSet<u32> {
    texto
        .lines()
        .filter_map(|l| l.split_once(". "))
        .filter_map(|(n, _)| n.trim().parse::<u32>().ok())
        .collect()
}

#[test]
fn proxima_tarefa_cita_item_que_existe_no_roadmap() {
    let status = ler("STATUS.md");
    let tarefa =
        secao(&status, "Próxima tarefa").expect("STATUS.md precisa da secao 'Próxima tarefa'");
    let citados = ids_citados(tarefa);

    assert!(
        !citados.is_empty(),
        "a 'Próxima tarefa' tem de citar um item do ROADMAP na forma 'ROADMAP X.Y'. Sem isso o \
         trabalhador nao sabe qual checkbox marcar no passo 8 e o titulo do PR nao passa no \
         commit-lint da CI."
    );

    let existentes = ids_existentes();
    let fantasmas: Vec<&String> = citados
        .iter()
        .filter(|id| !existentes.contains(*id))
        .collect();
    assert!(
        fantasmas.is_empty(),
        "a 'Próxima tarefa' aponta para item que nao existe nem no ROADMAP.md nem no arquivo de \
         fechados: {fantasmas:?}. Handoff para item fantasma custou dois PRs (#114 e #115, ambos \
         intitulados 'ROADMAP 4.4f' quando so existiam 4.4 e 4.4a-4.4e): o trabalhador nao tem \
         linha para marcar e inventa uma. Crie o item ANTES de escrever o handoff."
    );
}

#[test]
fn status_nao_guarda_invariante_no_lugar_do_handoff() {
    let status = ler("STATUS.md");
    let secoes: Vec<&str> = status
        .lines()
        .filter(|l| l.starts_with("## Notas") || l.starts_with("## Invariantes"))
        .collect();

    assert!(
        secoes.is_empty(),
        "invariante e NOTA sao referencia, nao handoff: moram em {INVARIANTES} e a 'Próxima \
         tarefa' cita as relevantes por numero. No STATUS.md elas sao relidas em toda rodada, \
         inclusive nas que nao tocam CPU — 5 KB de nota da era da CPU pagos numa iteracao de \
         GPU. Secoes encontradas: {secoes:?}"
    );

    let quantas = numerados(&ler(INVARIANTES)).len();
    assert!(
        quantas >= MINIMO_DE_INVARIANTES,
        "{INVARIANTES} tem {quantas} entradas numeradas (minimo {MINIMO_DE_INVARIANTES}). Este \
         teste existe para MOVER a referencia, nao para apaga-la: sem esta segunda afirmacao, \
         deletar as notas do STATUS e nao criar o arquivo deixaria o teste verde."
    );
}

#[test]
fn placar_do_status_bate_com_a_contagem_de_testes() {
    let status = ler("STATUS.md");
    let declarado: usize = status
        .split("Workspace: **")
        .nth(1)
        .and_then(|resto| resto.split("**").next())
        .and_then(|n| n.trim().parse().ok())
        .expect("STATUS.md precisa da linha 'Workspace: **N** testes.'");

    let mut real = 0usize;
    for arquivo in support::rust_test_files()
        .into_iter()
        .chain(support::rust_source_files())
    {
        let texto = fs::read_to_string(&arquivo).unwrap_or_default();
        real += texto.lines().filter(|l| l.trim() == "#[test]").count();
    }

    assert_eq!(
        declarado, real,
        "o placar do STATUS.md diz {declarado} testes e o workspace tem {real}. O placar e o \
         unico numero que o orquestrador usa para saber se uma rodada acrescentou ou destruiu \
         teste; errado, ele mente em silencio (ficou em 690 por quatro iteracoes enquanto o \
         workspace ja tinha 703). Atualize o placar no passo 8, junto com o resto do handoff."
    );
}

#[test]
fn handoff_aponta_invariantes_por_numero() {
    let status = ler("STATUS.md");
    let tarefa =
        secao(&status, "Próxima tarefa").expect("STATUS.md precisa da secao 'Próxima tarefa'");
    let linha = tarefa
        .lines()
        .find(|l| l.contains("Invariantes relevantes:"))
        .unwrap_or_else(|| {
            panic!(
                "a 'Próxima tarefa' precisa de uma linha 'Invariantes relevantes: N, M' apontando \
                 para {INVARIANTES}. Escreva 'Invariantes relevantes: nenhum' quando o item nao \
                 tocar nenhuma — a escolha tem de ser deliberada, senao mover as notas para fora \
                 do STATUS so faz o trabalhador deixar de le-las."
            )
        });

    let depois = linha.split("Invariantes relevantes:").nth(1).unwrap_or("");
    if depois.contains("nenhum") {
        return;
    }

    let citados: Vec<u32> = depois
        .split(|c: char| !c.is_ascii_digit())
        .filter_map(|n| n.parse::<u32>().ok())
        .collect();
    assert!(
        !citados.is_empty(),
        "'Invariantes relevantes:' sem numero nenhum e sem a palavra 'nenhum': {linha:?}"
    );

    let existentes = numerados(&ler(INVARIANTES));
    let fantasmas: Vec<&u32> = citados
        .iter()
        .filter(|n| !existentes.contains(*n))
        .collect();
    assert!(
        fantasmas.is_empty(),
        "o handoff cita invariante que nao existe em {INVARIANTES}: {fantasmas:?}. Numero errado \
         manda o trabalhador ler a nota errada, que e pior que nao ler nenhuma."
    );
}
