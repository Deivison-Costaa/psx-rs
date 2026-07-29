mod support;

use std::fs;

#[path = "support/mutation_format.rs"]
mod mutation_format;

use mutation_format::{load_manifests, Ocorrencias, PRIMEIRA_ITER_COM_MANIFESTO};

fn check_anchors(
    manifest: &mutation_format::Manifest,
    repo_root: &std::path::Path,
) -> Vec<String> {
    if manifest.arquivada.is_some() {
        return Vec::new();
    }
    let mut errs = Vec::new();
    for rec in &manifest.records {
        let arquivo = rec.arquivo.as_deref().unwrap_or(&manifest.alvo);
        let target_path = repo_root.join(arquivo);
        let file_content = match fs::read_to_string(&target_path) {
            Ok(c) => c,
            Err(e) => {
                errs.push(format!(
                    "{}: arquivo alvo '{}' nao pode ser lido: {e}",
                    manifest.rel_path, arquivo
                ));
                continue;
            }
        };
        let haystack = format!("\n{}\n", file_content);
        for (i, edit) in rec.edits.iter().enumerate() {
            let needle = format!("\n{}\n", edit.de);
            let count = haystack.matches(&needle).count();
            match &edit.ocorrencias {
                Ocorrencias::Exatamente(expected) => {
                    if count != *expected {
                        errs.push(format!(
                            "{}: registro '{}', edicao {}: ancora esperada {} vez(es) \
                             mas encontrada {} vez(es) em '{}'.\n\
                             Se a ancora envelheceu, atualize o manifesto e RODE A \
                             BATERIA DE NOVO, ou adicione 'arquivada: <motivo>' no \
                             cabecalho do manifesto.",
                            manifest.rel_path, rec.id, i + 1, expected, count, arquivo
                        ));
                    }
                }
                Ocorrencias::Todas => {
                    if count == 0 {
                        errs.push(format!(
                            "{}: registro '{}', edicao {}: ancora nao encontrada em '{}'",
                            manifest.rel_path, rec.id, i + 1, arquivo
                        ));
                    }
                }
            }
        }
    }
    errs
}

fn check_existence(repo_root: &std::path::Path) -> Vec<String> {
    let mut errs = Vec::new();
    let iter_dir = repo_root.join("docs/iterations");
    let mutantes_dir = repo_root.join("docs/mutantes");

    let entries = match fs::read_dir(&iter_dir) {
        Ok(e) => e,
        Err(e) => {
            errs.push(format!("docs/iterations/ nao pode ser lido: {e}"));
            return errs;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let fname = entry.file_name();
        let fname_str = fname.to_str().unwrap_or("");
        if !fname_str.ends_with(".md") || fname_str.len() < 4 {
            continue;
        }
        let prefix: u32 = match fname_str[..4].parse() {
            Ok(n) => n,
            Err(_) => continue,
        };
        if prefix < PRIMEIRA_ITER_COM_MANIFESTO {
            continue;
        }
        let stem = &fname_str[..fname_str.len() - 3];
        let manifest_path = mutantes_dir.join(format!("{stem}.mut"));
        if manifest_path.exists() {
            continue;
        }
        let doc_content = match fs::read_to_string(entry.path()) {
            Ok(c) => c,
            Err(_) => {
                errs.push(format!(
                    "{}: iteracao >= {PRIMEIRA_ITER_COM_MANIFESTO} sem manifesto e doc \
                     inacessivel",
                    fname_str
                ));
                continue;
            }
        };
        let has_opt_out = doc_content.lines().any(|l| {
            let t = l.trim();
            t.starts_with("Bateria de mutação: não se aplica") && t.len() >= 60
        });
        if !has_opt_out {
            errs.push(format!(
                "{}: iteracao >= {PRIMEIRA_ITER_COM_MANIFESTO} (prefixo {prefix}) sem \
                 docs/mutantes/{stem}.mut.\n\
                 Crie o manifesto OU adicione a linha 'Bateria de mutação: não se \
                 aplica — <motivo de pelo menos 40 chars>' em docs/iterations/{fname_str}.",
                fname_str
            ));
        }
    }
    errs
}

#[test]
fn manifesto_de_mutacao_ancoras_sao_reais() {
    let manifests = load_manifests().expect("erro carregando manifestos");
    let root = support::repo_root();
    let mut errs = Vec::new();

    for (_path, _rel, manifest) in &manifests {
        errs.extend(check_anchors(manifest, &root));
    }
    errs.extend(check_existence(&root));

    assert!(errs.is_empty(), "{errs:#?}");
}
