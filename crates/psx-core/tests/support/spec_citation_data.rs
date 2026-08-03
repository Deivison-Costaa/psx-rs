use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct RefDoc {
    pub corpo_line: usize,
    pub total_lines: usize,
    pub index_entries: Vec<IndexEntry>,
}

pub struct IndexEntry {
    pub offset_k: u32,
    pub title: String,
}

pub struct Citation {
    pub source_path: String,
    pub source_line: usize,
    pub ref_file: String,
    pub num: u32,
    pub num_end: Option<u32>,
    pub section_title: Option<String>,
    pub raw_text: String,
}

pub fn load_ref_docs(root: &Path) -> HashMap<String, RefDoc> {
    let ref_dir = root.join("docs").join("reference");
    let mut docs = HashMap::new();
    for entry in fs::read_dir(&ref_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e != "md") {
            continue;
        }
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        if name == "README.md" {
            continue;
        }
        let content = fs::read_to_string(&path).unwrap();
        let total_lines = content.lines().count();
        let mut corpo_line = 0usize;
        let mut index_entries = Vec::new();
        for (i, line) in content.lines().enumerate() {
            let line_num = i + 1;
            if line.trim() == "CORPO:" && corpo_line == 0 {
                corpo_line = line_num;
            }
            if let Some(entry) = parse_index_line(line) {
                index_entries.push(IndexEntry {
                    offset_k: entry.0,
                    title: entry.1,
                });
            }
        }
        docs.insert(
            name,
            RefDoc {
                corpo_line,
                total_lines,
                index_entries,
            },
        );
    }
    docs
}

fn parse_index_line(line: &str) -> Option<(u32, String)> {
    let trimmed = line.trim();
    if !trimmed.starts_with("- L") && !trimmed.starts_with("-L") {
        return None;
    }
    let after_dash = trimmed.trim_start_matches('-').trim();
    let rest = after_dash.strip_prefix('L')?;
    let colon_pos = rest.find(':')?;
    let num: u32 = rest[..colon_pos].trim().parse().ok()?;
    let title = rest[colon_pos + 1..].trim().to_string();
    Some((num, title))
}

pub fn normalize_title(t: &str) -> String {
    t.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '/')
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn title_contains(candidate: &str, target: &str) -> bool {
    let cn = normalize_title(candidate);
    let tn = normalize_title(target);
    cn.contains(&tn) || tn.contains(&cn)
}

pub fn find_in_index<'a>(index: &'a [IndexEntry], search: &str) -> Option<&'a IndexEntry> {
    let s = normalize_title(search);
    let matches: Vec<&IndexEntry> = index
        .iter()
        .filter(|e| title_contains(&s, &e.title))
        .collect();
    if matches.is_empty() {
        return None;
    }
    if matches.len() == 1 {
        return Some(matches[0]);
    }
    if let Some(exact) = matches.iter().find(|e| normalize_title(&e.title) == s) {
        return Some(exact);
    }
    let max_len = matches
        .iter()
        .map(|e| normalize_title(&e.title).len())
        .max()
        .unwrap_or(0);
    let longest: Vec<&&IndexEntry> = matches
        .iter()
        .filter(|e| normalize_title(&e.title).len() == max_len)
        .collect();
    if longest.len() == 1 {
        return Some(longest[0]);
    }
    None
}

pub fn index_match_ambiguity(index: &[IndexEntry], search: &str) -> Option<String> {
    let s = normalize_title(search);
    let matches: Vec<&IndexEntry> = index
        .iter()
        .filter(|e| title_contains(&s, &e.title))
        .collect();
    if matches.len() <= 1 {
        return None;
    }
    if matches.iter().any(|e| normalize_title(&e.title) == s) {
        return None;
    }
    let max_len = matches
        .iter()
        .map(|e| normalize_title(&e.title).len())
        .max()
        .unwrap_or(0);
    let longest: Vec<&&IndexEntry> = matches
        .iter()
        .filter(|e| normalize_title(&e.title).len() == max_len)
        .collect();
    if longest.len() <= 1 {
        return None;
    }
    let titles: Vec<String> = longest.iter().map(|e| e.title.clone()).collect();
    Some(format!(
        "ambíguo: {} candidatas empatadas para '{}': {}",
        longest.len(),
        search,
        titles.join("', '")
    ))
}

pub fn find_next_index_k(index: &[IndexEntry], current_k: u32) -> u32 {
    let mut next = u32::MAX;
    for e in index {
        if e.offset_k > current_k && e.offset_k < next {
            next = e.offset_k;
        }
    }
    next
}

pub fn is_scan_target(p: &Path) -> bool {
    let s = p.to_string_lossy().replace('\\', "/");
    if !s.ends_with(".md") {
        return false;
    }
    if s.contains("/target/") || s.contains("/.git/") {
        return false;
    }
    if s.contains("/docs/reference/") {
        return false;
    }
    true
}

pub fn collect_md_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_md_recursive(root, root, &mut files);
    files.sort();
    files
}

#[allow(clippy::only_used_in_recursion)]
fn collect_md_recursive(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            let path = entry.unwrap().path();
            if path.is_dir() {
                let dname = path.file_name().unwrap().to_str().unwrap();
                if dname == "target" || dname == ".git" {
                    continue;
                }
                if dname == "worktrees" && path.parent().is_some_and(|p| p.ends_with(".claude")) {
                    continue;
                }
                collect_md_recursive(root, &path, out);
            } else if is_scan_target(&path) {
                out.push(path);
            }
        }
    }
}

pub fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
        .replace('\\', "/")
}

pub fn check_c_body_check() -> Vec<String> {
    let mut errors = Vec::new();
    let ref_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .unwrap()
        .join("docs")
        .join("reference");
    for entry in fs::read_dir(&ref_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e != "md") {
            continue;
        }
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        if name == "README.md" {
            continue;
        }
        let content = fs::read_to_string(&path).unwrap();
        let count = content.lines().filter(|l| l.trim() == "CORPO:").count();
        if count != 1 {
            errors.push(format!(
                "{}: esperava exatamente 1 linha CORPO:, encontrei {count}",
                name
            ));
        }
    }
    errors
}
