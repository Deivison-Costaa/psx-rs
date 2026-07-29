mod support;

use std::fs;

#[path = "support/mutation_format.rs"]
mod mutation_format;

use mutation_format::{PRIMEIRA_ITER_COM_MANIFESTO, RecordKind, load_manifests};

#[test]
fn bateria_existencia_manifestos_ou_opt_out() {
    let root = support::repo_root();
    let iter_dir = root.join("docs/iterations");
    let mutantes_dir = root.join("docs/mutantes");
    let mut errs = Vec::new();

    let entries = fs::read_dir(&iter_dir).expect("docs/iterations deve existir");
    for entry in entries {
        let entry = entry.expect("entrada de docs/iterations");
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
                    "{}: iteracao >= {PRIMEIRA_ITER_COM_MANIFESTO} sem manifesto e doc inacessivel",
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
    assert!(errs.is_empty(), "{errs:#?}");
}

#[derive(Debug)]
struct ResultadoRow {
    id: String,
    tipo: String,
    _esperado: String,
    obtido: String,
    _segundos: String,
    testes: String,
}

fn parse_resultado(content: &str) -> Result<(String, Vec<ResultadoRow>), Vec<String>> {
    let mut errs = Vec::new();
    let mut commit: Option<String> = None;
    let mut rows: Vec<ResultadoRow> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("# gerado por") {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("# commit: ") {
            commit = Some(rest.to_string());
            continue;
        }
        if trimmed.starts_with("# rodado_em:") {
            continue;
        }
        let parts: Vec<&str> = trimmed.split(',').collect();
        if parts.len() < 6 {
            errs.push(format!("linha com menos de 6 colunas: '{trimmed}'"));
            continue;
        }
        rows.push(ResultadoRow {
            id: parts[0].to_string(),
            tipo: parts[1].to_string(),
            _esperado: parts[2].to_string(),
            obtido: parts[3].to_string(),
            _segundos: parts[4].to_string(),
            testes: parts[5..].join(",").to_string(),
        });
    }

    let commit = commit.unwrap_or_default();
    if commit.is_empty() {
        errs.push("resultado sem '# commit: <sha>'".to_string());
    }
    if errs.is_empty() {
        Ok((commit, rows))
    } else {
        Err(errs)
    }
}

#[test]
fn bateria_resultados_consistem_com_manifestos() {
    let manifests = load_manifests().expect("erro carregando manifestos");
    let root = support::repo_root();
    let mutantes_dir = root.join("docs/mutantes");
    let mut errs = Vec::new();

    for (_path, rel, manifest) in &manifests {
        let candidates: Vec<_> = match fs::read_dir(&mutantes_dir) {
            Ok(entries) => entries
                .filter_map(|e| {
                    let p = e.ok()?.path();
                    (p.extension().is_some_and(|e| e == "resultado")
                        && p.file_stem()
                            .and_then(|s| s.to_str())
                            .is_some_and(|s| s.starts_with(&format!("{:04}", manifest.iteracao))))
                    .then_some(p)
                })
                .collect(),
            Err(_) => Vec::new(),
        };
        if candidates.is_empty() {
            errs.push(format!(
                "{}: manifesto sem .resultado correspondente em docs/mutantes/",
                rel
            ));
            continue;
        }
        let result_path = &candidates[0];
        let result_content = match fs::read_to_string(result_path) {
            Ok(c) => c,
            Err(e) => {
                errs.push(format!("{}: .resultado ilegível: {e}", rel));
                continue;
            }
        };
        let (result_commit, rows) = match parse_resultado(&result_content) {
            Ok(r) => r,
            Err(e) => {
                errs.extend(e.iter().map(|e| format!("{}: {e}", rel)));
                continue;
            }
        };
        if result_commit.len() < 7 {
            errs.push(format!(
                "{}: '# commit:' nao tem forma de sha: '{result_commit}'",
                rel
            ));
        }
        let manifest_ids: std::collections::HashSet<&str> =
            manifest.records.iter().map(|r| r.id.as_str()).collect();
        let result_ids: std::collections::HashSet<&str> =
            rows.iter().map(|r| r.id.as_str()).collect();
        if manifest_ids != result_ids {
            let only_manifest: Vec<_> = manifest_ids.difference(&result_ids).collect();
            let only_result: Vec<_> = result_ids.difference(&manifest_ids).collect();
            errs.push(format!(
                "{}: ids nao formam bijecao — so no manifesto: {:?}, so no resultado: {:?}",
                rel, only_manifest, only_result
            ));
        }
        for row in &rows {
            match row.tipo.as_str() {
                "mutante" => {
                    if row.obtido != "morreu" {
                        errs.push(format!(
                            "{}: mutante '{}' com obtido='{}' (esperado=morreu)",
                            rel, row.id, row.obtido
                        ));
                    }
                    if row.testes.is_empty() {
                        errs.push(format!(
                            "{}: mutante '{}' com coluna testes vazia",
                            rel, row.id
                        ));
                    }
                }
                "controle" | "equivalente" => {
                    if row.obtido != "sobreviveu" {
                        errs.push(format!(
                            "{}: '{}' '{}' com obtido='{}' (esperado=sobreviveu)",
                            rel, row.tipo, row.id, row.obtido
                        ));
                    }
                }
                _ => {
                    errs.push(format!(
                        "{}: tipo desconhecido '{}' no registro '{}'",
                        rel, row.tipo, row.id
                    ));
                }
            }
        }
    }
    assert!(errs.is_empty(), "{errs:#?}");
}

fn find_test_functions(file_content: &str) -> std::collections::HashSet<String> {
    let mut names = std::collections::HashSet::new();
    for line in file_content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("fn ") {
            if let Some(end) = rest.find('(') {
                let name = rest[..end].trim().to_string();
                names.insert(name);
            }
        }
    }
    names
}

#[test]
fn bateria_nomes_de_teste_existem() {
    let manifests = load_manifests().expect("erro carregando manifestos");
    let root = support::repo_root();
    let mutantes_dir = root.join("docs/mutantes");
    let tests_dir = root.join("crates/psx-core/tests");
    let mut errs = Vec::new();

    for (_path, rel, manifest) in &manifests {
        let candidates: Vec<_> = match fs::read_dir(&mutantes_dir) {
            Ok(entries) => entries
                .filter_map(|e| {
                    let p = e.ok()?.path();
                    (p.extension().is_some_and(|e| e == "resultado")
                        && p.file_stem()
                            .and_then(|s| s.to_str())
                            .is_some_and(|s| s.starts_with(&format!("{:04}", manifest.iteracao))))
                    .then_some(p)
                })
                .collect(),
            Err(_) => Vec::new(),
        };
        if candidates.is_empty() {
            continue;
        }
        let result_content = match fs::read_to_string(&candidates[0]) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let (_commit, rows) = match parse_resultado(&result_content) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let test_file = tests_dir.join(format!("{}.rs", manifest.teste));
        let test_fns = if test_file.exists() {
            find_test_functions(&fs::read_to_string(&test_file).unwrap_or_default())
        } else {
            std::collections::HashSet::new()
        };
        for row in &rows {
            if row.testes.is_empty() {
                continue;
            }
            for test_name in row.testes.split(';') {
                let name = test_name.trim();
                if name.is_empty() {
                    continue;
                }
                if !test_fns.contains(name) {
                    errs.push(format!(
                        "{}: registro '{}' credita teste '{}' que nao existe como fn em \
                         crates/psx-core/tests/{}.rs. Se o nome foi digitado errado, \
                         corrija no .resultado e rode a bateria de novo. Se acha que o \
                         credito e correto mas o nome nao existe, o .resultado foi \
                         adulterado — o script NAO inventa nomes.",
                        rel, row.id, name, manifest.teste
                    ));
                }
            }
        }
    }
    assert!(errs.is_empty(), "{errs:#?}");
}

#[test]
fn bateria_placar_bate_com_resultado() {
    let manifests = load_manifests().expect("erro carregando manifestos");
    let root = support::repo_root();
    let iter_dir = root.join("docs/iterations");
    let mutantes_dir = root.join("docs/mutantes");
    let mut errs = Vec::new();

    for (_path, rel, manifest) in &manifests {
        let iter_doc = iter_dir.join(format!("{:04}-{}.md", manifest.iteracao, {
            let stem = rel
                .strip_prefix("docs/mutantes/")
                .and_then(|s| s.strip_suffix(".mut"))
                .unwrap_or("");
            if stem.len() > 5 && stem[4..].starts_with('-') {
                &stem[5..]
            } else if stem.len() > 4 {
                &stem[4..]
            } else {
                stem
            }
        }));

        let doc_content = match fs::read_to_string(&iter_doc) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let placar_line = doc_content.lines().find(|l| {
            let t = l.trim();
            t.starts_with("Placar da bateria:")
        });
        let placar_line = match placar_line {
            Some(line) => line.trim().to_string(),
            None => {
                errs.push(format!(
                    "{}: doc de iteracao sem linha 'Placar da bateria:'",
                    rel
                ));
                continue;
            }
        };

        let candidates: Vec<_> = match fs::read_dir(&mutantes_dir) {
            Ok(entries) => entries
                .filter_map(|e| {
                    let p = e.ok()?.path();
                    (p.extension().is_some_and(|e| e == "resultado")
                        && p.file_stem()
                            .and_then(|s| s.to_str())
                            .is_some_and(|s| s.starts_with(&format!("{:04}", manifest.iteracao))))
                    .then_some(p)
                })
                .collect(),
            Err(_) => Vec::new(),
        };
        if candidates.is_empty() {
            continue;
        }
        let result_content = match fs::read_to_string(&candidates[0]) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let (_commit, rows) = match parse_resultado(&result_content) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let mut_morreu = rows
            .iter()
            .filter(|r| r.tipo == "mutante" && r.obtido == "morreu")
            .count();
        let ctrl_sobreviveu = rows
            .iter()
            .filter(|r| r.tipo == "controle" && r.obtido == "sobreviveu")
            .count();
        let eq_count = rows.iter().filter(|r| r.tipo == "equivalente").count();

        let mut_count_manifest = manifest
            .records
            .iter()
            .filter(|r| r.kind == RecordKind::Mutante)
            .count();
        let ctrl_count_manifest = manifest
            .records
            .iter()
            .filter(|r| r.kind == RecordKind::Controle)
            .count();

        if mut_morreu != mut_count_manifest {
            errs.push(format!(
                "{}: placar diz {}/{} mutantes mortos, mas numerador ({}) != denominador ({})",
                rel, mut_morreu, mut_count_manifest, mut_morreu, mut_count_manifest
            ));
        }
        if ctrl_sobreviveu != ctrl_count_manifest {
            errs.push(format!(
                "{}: placar diz {}/{} controles verdes, mas numerador ({}) != denominador ({})",
                rel, ctrl_sobreviveu, ctrl_count_manifest, ctrl_sobreviveu, ctrl_count_manifest
            ));
        }

        if !placar_line.contains(&format!(
            "{}/{} mutantes mortos, {}/{} controles verdes",
            mut_morreu, mut_count_manifest, ctrl_sobreviveu, ctrl_count_manifest
        )) {
            errs.push(format!(
                "{}: a linha 'Placar da bateria:' no doc nao confere com .resultado.\n\
                 Doc:    {placar_line}\n\
                 Esperado conter: {}/{} mutantes mortos, {}/{} controles verdes, {} equivalente",
                rel, mut_morreu, mut_count_manifest, ctrl_sobreviveu, ctrl_count_manifest, eq_count,
            ));
        }
    }
    assert!(errs.is_empty(), "{errs:#?}");
}

#[test]
fn bateria_protocolo_e_ferramenta_nao_driftam() {
    let root = support::repo_root();
    let mut errs = Vec::new();

    let skill_path = root.join(".claude/skills/iterate/SKILL.md");
    let skill_content =
        fs::read_to_string(&skill_path).expect(".claude/skills/iterate/SKILL.md deve existir");
    if !skill_content.contains("scripts/mutantes.ps1") {
        errs.push(
            "SKILL.md nao menciona 'scripts/mutantes.ps1'. \
             O protocolo de iteracao precisa referenciar o script de bateria \
             de mutacao para que o passo 6 (bateria de mutacao) sempre seja \
             executado com a ferramenta, nunca a mao."
                .to_string(),
        );
    }

    let ci_path = root.join(".github/workflows/ci.yml");
    let ci_content = fs::read_to_string(&ci_path).expect(".github/workflows/ci.yml deve existir");
    if !ci_content.contains("mutantes:") {
        errs.push(
            "ci.yml nao tem o job 'mutantes'. \
             O job de bateria de mutacao e obrigatorio para validar que \
             o placar do .resultado e regerado a cada PR."
                .to_string(),
        );
    }
    if ci_content.contains("mutantes:") {
        let section_start = ci_content.find("mutantes:").unwrap_or(0);
        let rest = &ci_content[section_start..];
        let has_ce = rest
            .lines()
            .take_while(|l| {
                !l.trim().ends_with(':') || l.trim().starts_with('-') || l.trim().starts_with('#')
            })
            .any(|l| l.contains("continue-on-error"));
        if has_ce {
            errs.push(
                "ci.yml: job 'mutantes' tem 'continue-on-error'. \
                 Passo pulado nao mede nada e deixa o job verde. \
                 O job mutantes falha se qualquer mutante sobreviver."
                    .to_string(),
            );
        }
    }
    assert!(errs.is_empty(), "{errs:#?}");
}
