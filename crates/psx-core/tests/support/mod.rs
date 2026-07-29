#![allow(dead_code)]

pub mod asm;
pub mod spec_citation_data;

use std::fs;
use std::path::{Path, PathBuf};

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("raiz do repositório")
}

pub fn rust_source_files() -> Vec<PathBuf> {
    rust_files_in("src")
}

pub fn rust_test_files() -> Vec<PathBuf> {
    rust_files_in("tests")
}

fn rust_files_in(subdir: &str) -> Vec<PathBuf> {
    let crates = repo_root().join("crates");
    let mut files = Vec::new();
    for entry in fs::read_dir(&crates).expect("lendo crates/") {
        let dir = entry.expect("entrada de crates/").path().join(subdir);
        if dir.is_dir() {
            collect_rs(&dir, &mut files);
        }
    }
    files.sort();
    files
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("lendo diretório de fontes") {
        let path = entry.expect("entrada de diretório").path();
        if path.is_dir() {
            collect_rs(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

pub fn relative(path: &Path) -> String {
    path.strip_prefix(repo_root())
        .unwrap_or(path)
        .display()
        .to_string()
}
