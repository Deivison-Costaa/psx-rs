mod support;

use std::collections::HashSet;
use std::fs;

const ARQUIVO: &str = "docs/ROADMAP-fechado.md";

fn ler(rel: &str) -> String {
    let path = support::repo_root().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} deve existir e ser legivel: {e}"))
}

fn id_do_item(linha: &str) -> Option<(String, bool)> {
    let fechado = linha.starts_with("- [x] ");
    let aberto = linha.starts_with("- [ ] ");
    if !fechado && !aberto {
        return None;
    }
    let resto = &linha[6..];
    let id = resto.split_whitespace().next()?;
    Some((id.to_string(), fechado))
}

fn marcos(texto: &str) -> Vec<(String, Vec<(String, bool)>)> {
    let mut saida: Vec<(String, Vec<(String, bool)>)> = Vec::new();
    for linha in texto.lines() {
        if linha.starts_with("## ") {
            saida.push((linha.to_string(), Vec::new()));
        } else if let Some(item) = id_do_item(linha) {
            if let Some(ultimo) = saida.last_mut() {
                ultimo.1.push(item);
            }
        }
    }
    saida
}

#[test]
fn marco_totalmente_fechado_nao_fica_no_roadmap() {
    let roadmap = ler("ROADMAP.md");
    let fechados: Vec<String> = marcos(&roadmap)
        .into_iter()
        .filter(|(_, itens)| !itens.is_empty() && itens.iter().all(|(_, f)| *f))
        .map(|(titulo, itens)| format!("{titulo} ({} itens, todos fechados)", itens.len()))
        .collect();

    assert!(
        fechados.is_empty(),
        "marco 100% fechado nao pode ocupar bytes do ROADMAP: mova para {ARQUIVO} e deixe \
         so um ponteiro de uma linha. O teto de 10 KB de roadmap_size.rs existe porque o \
         ROADMAP e a escada do que FALTA; o que ja subiu vira historico. Marcos a mover: {fechados:?}"
    );
}

#[test]
fn arquivo_de_fechados_so_contem_item_fechado() {
    let arquivo = ler(ARQUIVO);
    let abertos: Vec<String> = arquivo
        .lines()
        .filter_map(id_do_item)
        .filter(|(_, fechado)| !*fechado)
        .map(|(id, _)| id)
        .collect();

    assert!(
        abertos.is_empty(),
        "{ARQUIVO} e historico: item ABERTO ali some da escada e ninguem trabalha nele. \
         Itens abertos encontrados: {abertos:?}"
    );
}

#[test]
fn nenhum_item_aparece_nos_dois_arquivos() {
    let no_roadmap: HashSet<String> = ler("ROADMAP.md")
        .lines()
        .filter_map(id_do_item)
        .map(|(id, _)| id)
        .collect();
    let no_arquivo: HashSet<String> = ler(ARQUIVO)
        .lines()
        .filter_map(id_do_item)
        .map(|(id, _)| id)
        .collect();

    let ambos: Vec<&String> = no_roadmap.intersection(&no_arquivo).collect();
    assert!(
        ambos.is_empty(),
        "item duplicado entre ROADMAP.md e {ARQUIVO}: duas fontes de verdade divergem na \
         primeira edicao. Duplicados: {ambos:?}"
    );
    assert!(
        !no_arquivo.is_empty(),
        "{ARQUIVO} sem item nenhum — ou o arquivamento nao aconteceu, ou apagou o historico"
    );
}

#[test]
fn roadmap_aponta_para_o_arquivo_de_fechados() {
    let roadmap = ler("ROADMAP.md");
    assert!(
        roadmap.contains(ARQUIVO),
        "o ROADMAP tem de citar {ARQUIVO} em algum lugar: sem o ponteiro, quem le a escada \
         nao descobre que existe historico e reabre item ja fechado"
    );
    let _ = ler(ARQUIVO);
}
