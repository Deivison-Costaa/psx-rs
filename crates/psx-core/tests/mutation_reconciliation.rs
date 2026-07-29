mod support;

use std::fs;

#[path = "support/mutation_format.rs"]
mod mutation_format;

use mutation_format::{load_manifests, parse_resultado};

fn doc_de_iteracao(rel: &str, iteracao: u32) -> String {
    let stem = rel
        .strip_prefix("docs/mutantes/")
        .and_then(|s| s.strip_suffix(".mut"))
        .unwrap_or("");
    let slug = if stem.len() > 5 && stem[4..].starts_with('-') {
        &stem[5..]
    } else if stem.len() > 4 {
        &stem[4..]
    } else {
        stem
    };
    format!("{iteracao:04}-{slug}.md")
}

#[test]
fn reconciliacao_do_placar_nao_pode_ser_pulada() {
    let manifests = load_manifests().expect("erro carregando manifestos");
    let root = support::repo_root();
    let iter_dir = root.join("docs/iterations");
    let mutantes_dir = root.join("docs/mutantes");
    let mut errs = Vec::new();

    for (_path, rel, manifest) in &manifests {
        let doc_rel = doc_de_iteracao(rel, manifest.iteracao);
        let iter_doc = iter_dir.join(&doc_rel);

        let doc_content = match fs::read_to_string(&iter_doc) {
            Ok(c) => c,
            Err(e) => {
                errs.push(format!(
                    "{rel}: docs/iterations/{doc_rel} nao pode ser lido ({e}). \
                     A reconciliacao do placar depende deste doc; sem ele o placar do \
                     manifesto nao e conferido contra o .resultado por ninguem. \
                     Crie o doc da iteracao com o nome pareado pelo prefixo de 4 digitos."
                ));
                continue;
            }
        };

        let tem_placar = doc_content
            .lines()
            .any(|l| l.trim().starts_with("Placar da bateria:"));
        let tem_opt_out = doc_content
            .lines()
            .any(|l| l.trim().starts_with("Bateria de mutação: não se aplica"));
        if !tem_placar && !tem_opt_out {
            errs.push(format!(
                "{rel}: docs/iterations/{doc_rel} nao tem linha 'Placar da bateria:' \
                 nem a linha de nao-aplicabilidade. Um manifesto existe, entao o doc tem \
                 de declarar um placar que a maquina possa conferir."
            ));
        }

        let prefixo = format!("{:04}", manifest.iteracao);
        let candidatos: Vec<_> = match fs::read_dir(&mutantes_dir) {
            Ok(entries) => entries
                .filter_map(|e| {
                    let p = e.ok()?.path();
                    (p.extension().is_some_and(|x| x == "resultado")
                        && p.file_stem()
                            .and_then(|s| s.to_str())
                            .is_some_and(|s| s.starts_with(&prefixo)))
                    .then_some(p)
                })
                .collect(),
            Err(e) => {
                errs.push(format!("{rel}: docs/mutantes nao pode ser lido ({e})"));
                continue;
            }
        };

        if candidatos.is_empty() {
            errs.push(format!(
                "{rel}: nenhum docs/mutantes/{prefixo}*.resultado. O placar escrito a mao \
                 em docs/iterations/{doc_rel} nao esta sendo conferido contra nada. \
                 Rode scripts/mutantes.ps1 -Iter {prefixo} e versione o .resultado gerado; \
                 ele e a unica prova de que a bateria rodou de verdade."
            ));
            continue;
        }

        for candidato in &candidatos {
            let nome = candidato
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("<?>")
                .to_string();
            let conteudo = match fs::read_to_string(candidato) {
                Ok(c) => c,
                Err(e) => {
                    errs.push(format!(
                        "{rel}: docs/mutantes/{nome} existe mas nao pode ser lido ({e}). \
                         Resultado ilegivel equivale a bateria nao rodada."
                    ));
                    continue;
                }
            };
            if let Err(motivos) = parse_resultado(&conteudo) {
                errs.push(format!(
                    "{rel}: docs/mutantes/{nome} nao parseia como .resultado: {motivos:?}. \
                     O arquivo e gerado por scripts/mutantes.ps1 e nao deve ser editado a \
                     mao; se foi, gere de novo rodando a bateria."
                ));
            }
        }
    }

    assert!(
        errs.is_empty(),
        "a reconciliacao entre o placar do doc e o .resultado da maquina seria PULADA nos \
         casos abaixo. Cada um deles e uma porta por onde um placar inventado passa sem \
         ninguem conferir:\n{errs:#?}"
    );
}
