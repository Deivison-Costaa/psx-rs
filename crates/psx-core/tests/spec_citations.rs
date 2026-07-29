mod support;

use std::collections::HashMap;
use std::fs;

use support::spec_citation_data::{
    Citation, check_c_body_check, collect_md_files, find_in_index, find_next_index_k,
    load_ref_docs, relative_path,
};

struct RawRef {
    num: u32,
    num_end: Option<u32>,
    raw: String,
}

fn scan_ref_filename(chars: &[char], start: usize) -> Option<usize> {
    let len = chars.len();
    if start + 6 > len {
        return None;
    }
    if !chars[start].is_ascii_digit() || !chars[start + 1].is_ascii_digit() {
        return None;
    }
    if chars[start + 2] != '-' {
        return None;
    }
    let mut pos = start + 3;
    while pos < len
        && (chars[pos].is_ascii_lowercase() || chars[pos].is_ascii_digit() || chars[pos] == '-')
    {
        pos += 1;
    }
    if pos + 3 > len {
        return None;
    }
    if chars[pos] == '.' && chars[pos + 1] == 'm' && chars[pos + 2] == 'd' {
        return Some(pos + 3);
    }
    None
}

fn find_file_mentions(line: &str) -> Vec<String> {
    let no_bt: String = line.chars().filter(|&c| c != '`').collect();
    let chars: Vec<char> = no_bt.chars().collect();
    let mut files = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if let Some(end) = scan_ref_filename(&chars, i) {
            let fname: String = chars[i..end].iter().collect();
            if !files.contains(&fname) {
                files.push(fname);
            }
            i = end;
        } else {
            i += 1;
        }
    }
    files
}

fn find_line_refs(line: &str) -> Vec<RawRef> {
    let no_fmt: String = line
        .chars()
        .filter(|&c| c != '*' && c != '`' && c != '(' && c != ')')
        .collect();
    let chars: Vec<char> = no_fmt.chars().collect();
    let mut refs = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == 'L'
            && i + 1 < chars.len()
            && chars[i + 1].is_ascii_digit()
            && (i == 0 || !chars[i - 1].is_alphanumeric())
        {
            let num_start = i + 1;
            let mut j = num_start;
            while j < chars.len() && chars[j].is_ascii_digit() {
                j += 1;
            }
            let num_str: String = chars[num_start..j].iter().collect();
            if let Ok(num) = num_str.parse::<u32>() {
                let raw = format!("L{num_str}");
                let mut end = None;
                if j + 1 < chars.len() && chars[j] == '-' && chars[j + 1].is_ascii_digit() {
                    let mut k = j + 1;
                    while k < chars.len() && chars[k].is_ascii_digit() {
                        k += 1;
                    }
                    if let Ok(e) = chars[j + 1..k].iter().collect::<String>().parse::<u32>() {
                        end = Some(e);
                    }
                    i = k;
                } else {
                    i = j;
                }
                refs.push(RawRef {
                    num,
                    num_end: end,
                    raw,
                });
            } else {
                i = j;
            }
        } else {
            i += 1;
        }
    }
    refs
}

fn extract_title_for_ref(line: &str, ref_num: u32, ref_end: Option<u32>) -> Option<String> {
    let ref_str = if let Some(e) = ref_end {
        format!("L{ref_num}-L{e}")
    } else {
        format!("L{ref_num}")
    };

    let pos = line.find(&ref_str)?;
    let before = &line[..pos];
    let has_paren_before = before.ends_with('(');

    if let Some(title) = extract_after_section_keyword(before) {
        return Some(title);
    }

    if let Some(title) = extract_after_section_sign_before_ref(before) {
        return Some(title);
    }

    if has_paren_before {
        return extract_text_before_paren(before);
    }

    None
}

fn extract_after_section_keyword(before: &str) -> Option<String> {
    let lower = before.to_lowercase();
    for marker in &["seção", "secoes", "seções", "section"] {
        if let Some(pos) = lower.rfind(marker) {
            let after = before[pos + marker.len()..].trim();
            let cleaned = after.trim_matches(|c: char| c == ':').trim();
            if !cleaned.is_empty()
                && cleaned.len() >= 3
                && cleaned.len() <= 80
                && !cleaned.contains('|')
            {
                return Some(cleaned.to_string());
            }
        }
    }
    None
}

fn extract_after_section_sign_before_ref(before: &str) -> Option<String> {
    let chars: Vec<char> = before.chars().collect();
    for (i, &ch) in chars.iter().enumerate().rev() {
        if ch == '§' {
            let raw: String = chars[i + 1..].iter().collect();
            let title = raw.trim().trim_end_matches('(').trim().to_string();
            if title.len() >= 3 && !title.contains('|') {
                return Some(title);
            }
            break;
        }
    }
    None
}

fn extract_text_before_paren(before: &str) -> Option<String> {
    let trimmed = before.trim_end();
    if trimmed.is_empty() {
        return None;
    }

    let last_seg = trimmed
        .rsplit([',', ';', '|'])
        .next()
        .unwrap_or(trimmed)
        .trim();

    if last_seg.len() < 4 || last_seg.len() > 100 || last_seg.contains('|') {
        return None;
    }

    let noise = [
        "conforme",
        "spec",
        "ver",
        "item",
        "resolve",
        "comportamento",
        "assumido",
        "implementado",
        "conferir",
        "confirmar",
        "de",
        "para",
    ];
    let fw = last_seg.split_whitespace().next().unwrap_or("");
    if noise.contains(&fw.to_lowercase().as_str()) {
        return None;
    }

    Some(last_seg.to_string())
}

fn parse_citations_in_file(source_path: &str, content: &str) -> Vec<Citation> {
    let mut citations = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut last_file_mention: Option<(usize, String)> = None;

    for (idx, raw_line) in lines.iter().enumerate() {
        let line_num = idx + 1;
        let line_files = find_file_mentions(raw_line);
        let raw_refs = find_line_refs(raw_line);

        if line_files.is_empty() && raw_refs.is_empty() {
            continue;
        }

        let resolved_file = if line_files.len() == 1 {
            let f = line_files[0].clone();
            last_file_mention = Some((line_num, f.clone()));
            Some(f)
        } else if line_files.len() > 1 && !raw_refs.is_empty() {
            for r in &raw_refs {
                citations.push(Citation {
                    source_path: source_path.to_string(),
                    source_line: line_num,
                    ref_file: String::new(),
                    num: r.num,
                    num_end: r.num_end,
                    section_title: None,
                    raw_text: format!(
                        "ambíguo: 2+ menções a arquivo ({}) com ref na mesma linha — \
                         ponha o nome do arquivo na mesma linha da citação",
                        line_files.join(", ")
                    ),
                });
            }
            continue;
        } else if raw_refs.is_empty() {
            continue;
        } else if let Some((ref_ln, ref_file)) = &last_file_mention {
            if line_num - ref_ln <= 10 {
                Some(ref_file.clone())
            } else {
                for r in &raw_refs {
                    citations.push(Citation {
                        source_path: source_path.to_string(),
                        source_line: line_num,
                        ref_file: String::new(),
                        num: r.num,
                        num_end: r.num_end,
                        section_title: None,
                        raw_text: "sem menção a arquivo em 10 linhas — ponha o nome do \
                                   arquivo na mesma linha da citação"
                            .to_string(),
                    });
                }
                continue;
            }
        } else {
            for r in &raw_refs {
                citations.push(Citation {
                    source_path: source_path.to_string(),
                    source_line: line_num,
                    ref_file: String::new(),
                    num: r.num,
                    num_end: r.num_end,
                    section_title: None,
                    raw_text: "sem menção a arquivo anterior — ponha o nome do arquivo \
                               na mesma linha da citação"
                        .to_string(),
                });
            }
            continue;
        };

        let file_name = match resolved_file {
            Some(f) => f,
            None => continue,
        };

        for r in &raw_refs {
            let section_title = extract_title_for_ref(raw_line, r.num, r.num_end);
            citations.push(Citation {
                source_path: source_path.to_string(),
                source_line: line_num,
                ref_file: file_name.clone(),
                num: r.num,
                num_end: r.num_end,
                section_title,
                raw_text: r.raw.clone(),
            });
        }
    }

    citations
}

/// Docs de iteracao anteriores a este numero nao sao varridos: medi 54 citacoes
/// deslocadas no acervo historico, e corrigi-las e arqueologia, nao medicao. Mesmo
/// raciocinio do MAX_LAG de metrics_freshness.rs. Os docs VIVOS (STATUS, ROADMAP,
/// CLAUDE, SKILL) sao varridos sempre — e onde o handoff do proximo item e escrito.
const PISO_ITERACOES: u32 = 43;

#[test]
fn citacoes_de_spec_sao_validas() {
    let root = support::repo_root();
    let ref_docs = load_ref_docs(&root);

    let c_errors = check_c_body_check();
    for e in &c_errors {
        eprintln!("CHECK C: {e}");
    }

    let md_files: Vec<_> = collect_md_files(&root)
        .into_iter()
        .filter(|p| {
            let rel = relative_path(&root, p);
            if rel == "STATUS.md" {
                return true;
            }
            if let Some(stripped) = rel.strip_prefix("docs/iterations/") {
                if let Ok(n) = stripped.chars().take(4).collect::<String>().parse::<u32>() {
                    return n > PISO_ITERACOES;
                }
            }
            if rel == "docs/orquestracao.md" {
                return false;
            }
            !rel.starts_with("docs/reference/")
        })
        .collect();
    let mut all_errors: Vec<String> = Vec::new();
    type BucketEntry = (String, u32, Option<u32>, u32, usize);
    let mut offset_buckets: HashMap<String, Vec<BucketEntry>> = HashMap::new();

    for path in &md_files {
        let rel = relative_path(&root, path);
        let content = fs::read_to_string(path).unwrap();
        let citations = parse_citations_in_file(&rel, &content);

        for cit in &citations {
            if cit.ref_file.is_empty() {
                let line_text = content.lines().nth(cit.source_line - 1).unwrap_or("");
                all_errors.push(format!(
                    "{}:{} — {}\n  linha: {}",
                    cit.source_path, cit.source_line, cit.raw_text, line_text
                ));
                continue;
            }

            let ref_doc = match ref_docs.get(&cit.ref_file) {
                Some(d) => d,
                None => {
                    let mut available: Vec<_> = ref_docs.keys().cloned().collect();
                    available.sort();
                    all_errors.push(format!(
                        "{}:{} — arquivo de referência '{}' não existe.\n  \
                         disponíveis: {}",
                        cit.source_path,
                        cit.source_line,
                        cit.ref_file,
                        available.join(", ")
                    ));
                    continue;
                }
            };

            if cit.num == 0 {
                continue;
            }

            if cit.num as usize > ref_doc.total_lines || cit.num < 1 {
                all_errors.push(format!(
                    "{}:{} — L{} fora dos limites de {} (1..{})",
                    cit.source_path, cit.source_line, cit.num, cit.ref_file, ref_doc.total_lines
                ));
                continue;
            }

            if let Some(end) = cit.num_end {
                if cit.num > end {
                    all_errors.push(format!(
                        "{}:{} — faixa L{}-L{} inválida (início > fim)",
                        cit.source_path, cit.source_line, cit.num, end
                    ));
                    continue;
                }
                if end - cit.num > 400 {
                    all_errors.push(format!(
                        "{}:{} — faixa L{}-L{} excede 400 linhas ({})",
                        cit.source_path,
                        cit.source_line,
                        cit.num,
                        end,
                        end - cit.num
                    ));
                    continue;
                }
            }

            if ref_doc.corpo_line == 0 {
                continue;
            }

            let offset = ref_doc.corpo_line as u32 + 1;

            if let Some(ref title) = cit.section_title {
                if let Some(idx_entry) = find_in_index(&ref_doc.index_entries, title) {
                    let k = idx_entry.offset_k;
                    let real = k + offset;
                    let next_k = find_next_index_k(&ref_doc.index_entries, k);
                    let next_real = if next_k == u32::MAX {
                        u32::MAX
                    } else {
                        next_k + offset
                    };

                    let in_real = cit.num >= real && (next_real == u32::MAX || cit.num < next_real);
                    let in_index_range = next_k != u32::MAX && cit.num >= k && cit.num < next_k;

                    if !in_real && in_index_range {
                        offset_buckets
                            .entry(cit.source_path.clone())
                            .or_default()
                            .push((
                                cit.ref_file.clone(),
                                cit.num,
                                cit.num_end,
                                offset,
                                cit.source_line,
                            ));
                        all_errors.push(format!(
                            "{}:{} — L{} é o offset do ÍNDICE, não a linha real.\n  \
                             A seção '{}' começa em L{real} \
                             (índice L{} + offset {offset} do CORPO:).\n  \
                             Provavelmente copiada do índice.",
                            cit.source_path, cit.source_line, cit.num, idx_entry.title, k,
                        ));
                    } else if !in_real {
                        let diff = (cit.num as i64 - real as i64).abs();
                        if diff > 5 && cit.num < ref_doc.corpo_line as u32 + 1 + 10 {
                            continue;
                        }
                        if diff > 3 {
                            all_errors.push(format!(
                                "{}:{} — L{} não corresponde à seção '{}'.\n  \
                                 A seção começa em L{real} (índice L{} + offset {offset}).",
                                cit.source_path, cit.source_line, cit.num, idx_entry.title, k,
                            ));
                        }
                    }
                }
            }
        }
    }

    for (doc_path, entries) in &offset_buckets {
        if entries.len() >= 2 {
            let offsets: Vec<u32> = entries.iter().map(|e| e.3).collect();
            let all_same = offsets.iter().all(|o| *o == offsets[0]);
            if all_same {
                let mut affected: Vec<&str> = entries.iter().map(|e| e.0.as_str()).collect();
                affected.sort();
                affected.dedup();
                all_errors.push(format!(
                    "AGRUPADO: {} — as {} citações deste doc batem todas com \
                     offset constante +{} — o doc inteiro veio do índice.\n  \
                     arquivos afetados: {}",
                    doc_path,
                    entries.len(),
                    offsets[0],
                    affected.join(", "),
                ));
            }
        }
    }

    all_errors.extend(c_errors);

    if !all_errors.is_empty() {
        panic!(
            "{} erro(s) de citação de spec:\n\n{}",
            all_errors.len(),
            all_errors.join("\n\n")
        );
    }
}
