#![allow(dead_code)]

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub const ROTULO_MIN_CHARS: usize = 15;
pub const JUSTIFICATIVA_MIN_CHARS: usize = 80;
pub const PRIMEIRA_ITER_COM_MANIFESTO: u32 = 42;
pub const MIN_MUTANTES: usize = 5;
pub const MIN_CONTROLES: usize = 2;

#[derive(Debug, Clone)]
pub struct Edit {
    pub de: String,
    pub para: String,
    pub ocorrencias: Ocorrencias,
}

#[derive(Debug, Clone)]
pub enum Ocorrencias {
    Exatamente(usize),
    Todas,
}

impl Default for Ocorrencias {
    fn default() -> Self {
        Ocorrencias::Exatamente(1)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecordKind {
    Mutante,
    Controle,
    Equivalente,
}

#[derive(Debug, Clone)]
pub struct Record {
    pub kind: RecordKind,
    pub id: String,
    pub rotulo: String,
    pub arquivo: Option<String>,
    pub teste: Option<String>,
    pub justificativa: Option<String>,
    pub edits: Vec<Edit>,
}

#[derive(Debug)]
pub struct Manifest {
    pub rel_path: String,
    pub formato: u32,
    pub iteracao: u32,
    pub item: String,
    pub alvo: String,
    pub teste: String,
    pub arquivada: Option<String>,
    pub records: Vec<Record>,
}

fn parse_directive(line: &str) -> Option<(&str, &str)> {
    let colon_pos = line.find(':')?;
    let key = line[..colon_pos].trim();
    let value = line[colon_pos + 1..].trim();
    if key.is_empty() {
        return None;
    }
    Some((key, value))
}

fn emit_record(
    kind: Option<RecordKind>,
    id: Option<String>,
    rotulo: Option<String>,
    arquivo: Option<String>,
    teste: Option<String>,
    justificativa: Option<String>,
    edits: Vec<Edit>,
    records: &mut Vec<Record>,
    errs: &mut Vec<String>,
) {
    let kind = match kind {
        Some(k) => k,
        None => return,
    };
    let id = match id {
        Some(i) => i,
        None => return,
    };
    let rotulo = rotulo.unwrap_or_default();

    if rotulo.len() < ROTULO_MIN_CHARS {
        errs.push(format!(
            "registro '{id}': rotulo tem {} caracteres (minimo {ROTULO_MIN_CHARS})",
            rotulo.len()
        ));
    }
    if edits.is_empty() {
        errs.push(format!("registro '{id}': nenhuma edicao (@@DE/@@PARA/@@FIM)"));
    }
    for (i, edit) in edits.iter().enumerate() {
        if edit.de.is_empty() {
            errs.push(format!("registro '{id}', edicao {}: @@DE vazio", i + 1));
        }
        if edit.para.is_empty() {
            errs.push(format!("registro '{id}', edicao {}: @@PARA vazio", i + 1));
        }
        if edit.de.trim() == edit.para.trim() {
            errs.push(format!(
                "registro '{id}', edicao {}: @@DE e @@PARA identicos ignorando whitespace",
                i + 1
            ));
        }
        if let Some(ref arq) = arquivo {
            if !arq.starts_with("crates/") || !arq.contains("/src/") {
                errs.push(format!(
                    "registro '{id}': arquivo '{arq}' nao esta sob crates/*/src/"
                ));
            }
        }
    }
    if kind == RecordKind::Equivalente {
        match &justificativa {
            Some(j) if j.len() >= JUSTIFICATIVA_MIN_CHARS => {}
            Some(j) => errs.push(format!(
                "registro '{id}': equivalente com justificativa de {} caracteres (minimo {JUSTIFICATIVA_MIN_CHARS})",
                j.len()
            )),
            None => errs.push(format!("registro '{id}': equivalente sem justificativa")),
        }
    }
    records.push(Record {
        kind,
        id,
        rotulo,
        arquivo,
        teste,
        justificativa,
        edits,
    });
}

pub fn parse_manifest(path: &Path, rel_path: &str) -> Result<Manifest, Vec<String>> {
    let content = fs::read_to_string(path)
        .map_err(|e| vec![format!("{rel_path}: nao foi possivel ler: {e}")])?;
    let lines: Vec<&str> = content.lines().collect();
    let mut errs: Vec<String> = Vec::new();

    let mut formato: Option<u32> = None;
    let mut iteracao: Option<u32> = None;
    let mut item: Option<String> = None;
    let mut alvo: Option<String> = None;
    let mut teste: Option<String> = None;
    let mut arquivada: Option<String> = None;
    let mut records: Vec<Record> = Vec::new();

    let mut rec_kind: Option<RecordKind> = None;
    let mut rec_id: Option<String> = None;
    let mut rec_rotulo: Option<String> = None;
    let mut rec_arquivo: Option<String> = None;
    let mut rec_teste: Option<String> = None;
    let mut rec_justificativa: Option<String> = None;
    let mut rec_ocorrencias: Ocorrencias = Ocorrencias::default();
    let mut rec_edits: Vec<Edit> = Vec::new();

    let mut in_record = false;
    let mut collecting_de: Option<Vec<String>> = None;
    let mut collecting_para: Option<Vec<String>> = None;

    for line in lines {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if collecting_para.is_some() {
            match line {
                "@@FIM" => {
                    let de_lines = collecting_de.take().unwrap_or_default();
                    let para_lines = collecting_para.take().unwrap();
                    let oc = std::mem::take(&mut rec_ocorrencias);
                    rec_edits.push(Edit {
                        de: de_lines.join("\n"),
                        para: para_lines.join("\n"),
                        ocorrencias: oc,
                    });
                    continue;
                }
                "@@DE" | "@@PARA" => {
                    errs.push(format!(
                        "{rel_path}: sentinela '{line}' dentro de @@PARA sem @@FIM"
                    ));
                    collecting_de = None;
                    collecting_para = None;
                    continue;
                }
                content_line => {
                    if let Some(ref mut p) = collecting_para {
                        p.push(content_line.to_string());
                    }
                    continue;
                }
            }
        }

        if collecting_de.is_some() {
            match line {
                "@@PARA" => {
                    collecting_para = Some(Vec::new());
                    continue;
                }
                "@@FIM" => {
                    errs.push(format!("{rel_path}: @@FIM sem @@PARA"));
                    collecting_de = None;
                    continue;
                }
                "@@DE" => {
                    errs.push(format!("{rel_path}: @@DE aninhado (sem @@FIM anterior)"));
                    collecting_de = None;
                    continue;
                }
                content_line => {
                    if let Some(ref mut d) = collecting_de {
                        d.push(content_line.to_string());
                    }
                    continue;
                }
            }
        }

        if line == "@@DE" {
            if !in_record {
                errs.push(format!("{rel_path}: @@DE antes de iniciar um registro"));
                continue;
            }
            collecting_de = Some(Vec::new());
            continue;
        }

        if line == "@@PARA" || line == "@@FIM" {
            errs.push(format!("{rel_path}: {line} sem @@DE anterior"));
            continue;
        }

        if let Some((key, value)) = parse_directive(line) {
            match key {
                "formato" => {
                    formato = Some(value.parse().unwrap_or(0));
                }
                "iteracao" => {
                    iteracao = Some(value.parse().unwrap_or(0));
                }
                "item" => item = Some(value.to_string()),
                "alvo" => alvo = Some(value.to_string()),
                "teste" => teste = Some(value.to_string()),
                "arquivada" => arquivada = Some(value.to_string()),
                "rotulo" => rec_rotulo = Some(value.to_string()),
                "arquivo" => rec_arquivo = Some(value.to_string()),
                "ocorrencias" => {
                    rec_ocorrencias = if value == "todas" {
                        Ocorrencias::Todas
                    } else {
                        Ocorrencias::Exatamente(value.parse().unwrap_or(1))
                    };
                }
                "justificativa" => rec_justificativa = Some(value.to_string()),
                "mutante" | "controle" | "equivalente" => {
                    emit_record(
                        rec_kind.take(),
                        rec_id.take(),
                        rec_rotulo.take(),
                        rec_arquivo.take(),
                        rec_teste.take(),
                        rec_justificativa.take(),
                        std::mem::take(&mut rec_edits),
                        &mut records,
                        &mut errs,
                    );
                    rec_ocorrencias = Ocorrencias::default();
                    rec_id = Some(value.to_string());
                    rec_kind = Some(match key {
                        "mutante" => RecordKind::Mutante,
                        "controle" => RecordKind::Controle,
                        "equivalente" => RecordKind::Equivalente,
                        _ => unreachable!(),
                    });
                    rec_rotulo = None;
                    rec_arquivo = None;
                    rec_teste = None;
                    rec_justificativa = None;
                    in_record = true;
                }
                _ => errs.push(format!("{rel_path}: chave desconhecida: '{key}'")),
            }
        } else {
            errs.push(format!(
                "{rel_path}: linha sem chave:valor nem sentinela: '{line}'"
            ));
        }
    }

    emit_record(
        rec_kind.take(),
        rec_id.take(),
        rec_rotulo.take(),
        rec_arquivo.take(),
        rec_teste.take(),
        rec_justificativa.take(),
        std::mem::take(&mut rec_edits),
        &mut records,
        &mut errs,
    );

    if collecting_de.is_some() || collecting_para.is_some() {
        errs.push(format!("{rel_path}: @@DE sem @@FIM (fim de arquivo)"));
    }

    let formato = formato.unwrap_or(0);
    let iteracao_num = iteracao.unwrap_or(0);
    let item_str = item.unwrap_or_default();
    let alvo_str = alvo.unwrap_or_default();
    let teste_name = teste.unwrap_or_default();

    if formato != 1 {
        errs.push(format!(
            "{rel_path}: formato deve ser 1 (encontrado {formato})"
        ));
    }
    if iteracao_num == 0 {
        errs.push(format!("{rel_path}: iteracao ausente"));
    }
    let prefix = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    if iteracao_num > 0 && !prefix.starts_with(&format!("{:04}", iteracao_num)) {
        errs.push(format!(
            "{rel_path}: iteracao {iteracao_num} nao bate com o prefixo do arquivo '{prefix}'"
        ));
    }
    if item_str.is_empty() {
        errs.push(format!("{rel_path}: item ausente"));
    }
    if alvo_str.is_empty() {
        errs.push(format!("{rel_path}: alvo ausente"));
    }
    if teste_name.is_empty() {
        errs.push(format!("{rel_path}: teste ausente"));
    }

    let mut ids = HashSet::new();
    for rec in &records {
        if !ids.insert(&rec.id) {
            errs.push(format!("{rel_path}: id duplicado '{}'", rec.id));
        }
    }

    if !errs.is_empty() {
        return Err(errs);
    }

    Ok(Manifest {
        rel_path: rel_path.to_string(),
        formato,
        iteracao: iteracao_num,
        item: item_str,
        alvo: alvo_str,
        teste: teste_name,
        arquivada,
        records,
    })
}

pub fn load_manifests() -> Result<Vec<(PathBuf, String, Manifest)>, Vec<String>> {
    let root = crate::support::repo_root();
    let mutantes_dir = root.join("docs/mutantes");
    let mut all_errs: Vec<String> = Vec::new();
    let mut manifests: Vec<(PathBuf, String, Manifest)> = Vec::new();

    match fs::read_dir(&mutantes_dir) {
        Ok(entries) => {
            for entry in entries {
                let entry = match entry {
                    Ok(e) => e,
                    Err(e) => {
                        all_errs.push(format!("erro lendo entrada de docs/mutantes/: {e}"));
                        continue;
                    }
                };
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "mut") {
                    let rel = crate::support::relative(&path);
                    match parse_manifest(&path, &rel) {
                        Ok(m) => manifests.push((path, rel, m)),
                        Err(mut e) => all_errs.append(&mut e),
                    }
                }
            }
        }
        Err(_) => {
            all_errs.push(
                "docs/mutantes/ nao existe. Crie o diretorio e o manifesto da iteracao \
                 antes de rodar este teste."
                    .to_string(),
            );
        }
    }

    if !all_errs.is_empty() {
        return Err(all_errs);
    }
    Ok(manifests)
}
