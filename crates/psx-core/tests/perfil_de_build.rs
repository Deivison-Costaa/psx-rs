mod support;

use std::fs;

fn cargo_da_raiz() -> String {
    let path = support::repo_root().join("Cargo.toml");
    fs::read_to_string(&path).expect("Cargo.toml da raiz deve existir")
}

fn secao(toml: &str, nome: &str) -> String {
    let cabecalho = format!("[{nome}]");
    let pos = toml
        .find(&cabecalho)
        .unwrap_or_else(|| panic!("Cargo.toml da raiz deve declarar [{nome}]"));
    let resto = &toml[pos + cabecalho.len()..];
    let fim = resto.find("\n[").unwrap_or(resto.len());
    resto[..fim].to_string()
}

fn valor(corpo: &str, chave: &str) -> String {
    let agulha = format!("{chave} = ");
    let pos = corpo
        .find(&agulha)
        .unwrap_or_else(|| panic!("perfil deve declarar `{chave}` explicitamente"));
    let resto = &corpo[pos + agulha.len()..];
    let fim = resto.find('\n').unwrap_or(resto.len());
    resto[..fim].trim().trim_matches('"').to_string()
}

fn perfis() -> [(&'static str, String); 2] {
    let toml = cargo_da_raiz();
    [
        ("profile.dev", secao(&toml, "profile.dev")),
        ("profile.test", secao(&toml, "profile.test")),
    ]
}

#[test]
fn perfil_dev_otimiza() {
    let corpo = secao(&cargo_da_raiz(), "profile.dev");
    assert_ne!(
        valor(&corpo, "opt-level"),
        "0",
        "[profile.dev] com opt-level 0 custa 7,5x no tempo de teste (528 s contra 70 s \
         no testevent_descritor). A lib psx-core linkada por teste de integracao compila \
         sob o perfil dev, entao e ele que governa o custo de cpu.step()"
    );
}

#[test]
fn perfil_test_otimiza() {
    let corpo = secao(&cargo_da_raiz(), "profile.test");
    assert_ne!(
        valor(&corpo, "opt-level"),
        "0",
        "[profile.test] governa o binario de teste, onde moram os lacos de sondagem"
    );
}

#[test]
fn otimizacao_preserva_debug_assertions() {
    for (nome, corpo) in perfis() {
        assert_eq!(
            valor(&corpo, "debug-assertions"),
            "true",
            "[{nome}] nao pode comprar velocidade desligando debug-assertions: \
             `opt-level` e `debug-assertions` sao chaves independentes"
        );
    }
}

#[test]
fn otimizacao_preserva_overflow_checks() {
    for (nome, corpo) in perfis() {
        assert_eq!(
            valor(&corpo, "overflow-checks"),
            "true",
            "[{nome}] sem overflow-checks esconde estouro de inteiro no R3000A, \
             que e exatamente a classe de erro que o emulador precisa ver"
        );
    }
}
