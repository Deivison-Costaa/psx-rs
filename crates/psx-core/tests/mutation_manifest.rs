mod support;

use std::collections::HashMap;

#[path = "support/mutation_format.rs"]
mod mutation_format;

use mutation_format::{MIN_CONTROLES, MIN_MUTANTES, RecordKind, load_manifests};

#[test]
fn manifesto_de_mutacao_forma_e_integridade() {
    let manifests = load_manifests().expect("erro carregando manifestos");

    let mut errs = Vec::new();
    let mut total_mutantes = 0usize;
    let mut total_controles = 0usize;
    let mut all_edits: HashMap<(String, String), Vec<String>> = HashMap::new();

    for (_path, rel, manifest) in &manifests {
        let _ = &manifest.formato;
        let _ = &manifest.iteracao;
        let _ = &manifest.item;
        let _ = &manifest.teste;
        for rec in &manifest.records {
            let _ = &rec.rotulo;
            let _ = &rec.teste;
            let _ = &rec.justificativa;
            match rec.kind {
                RecordKind::Mutante => total_mutantes += 1,
                RecordKind::Controle => total_controles += 1,
                RecordKind::Equivalente => {}
            }
            for edit in &rec.edits {
                let key = (edit.de.clone(), edit.para.clone());
                all_edits
                    .entry(key)
                    .or_default()
                    .push(format!("{rel}/{}", rec.id));
            }
        }
    }

    if total_mutantes < MIN_MUTANTES {
        errs.push(format!(
            "volume insuficiente: {total_mutantes} mutantes (minimo {MIN_MUTANTES}). \
             Cada manifesto precisa de pelo menos {MIN_MUTANTES} mutantes para que a \
             bateria tenha poder estatistico."
        ));
    }
    if total_controles < MIN_CONTROLES {
        errs.push(format!(
            "volume insuficiente: {total_controles} controles (minimo {MIN_CONTROLES}). \
             Controles provam que o mecanismo de mutacao nao quebra compilacao \
             trivialmente."
        ));
    }

    for ((de, para), registros) in &all_edits {
        if registros.len() > 1 {
            errs.push(format!(
                "edicao (de, para) duplicada entre registros {}.\n\
                 de: {de:?}\npara: {para:?}\n\
                 Dois mutantes com a mesma edicao inflam o placar — foi o padrao da \
                 iter 0038.",
                registros.join(", ")
            ));
        }
    }

    assert!(errs.is_empty(), "{errs:#?}");
}
