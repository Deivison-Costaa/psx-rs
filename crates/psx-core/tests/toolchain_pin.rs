mod support;

use std::fs;

const ARQUIVO: &str = "rust-toolchain.toml";

#[test]
fn toolchain_pinado_em_versao_exata() {
    let toml = fs::read_to_string(support::repo_root().join(ARQUIVO))
        .unwrap_or_else(|_| panic!("{ARQUIVO} deve existir na raiz do repositório"));
    let channel = toml
        .lines()
        .map(str::trim)
        .filter(|l| !l.starts_with('#'))
        .find_map(|l| l.strip_prefix("channel"))
        .and_then(|resto| resto.split('"').nth(1))
        .unwrap_or_else(|| panic!("{ARQUIVO} precisa de uma linha channel = \"X.Y.Z\""))
        .to_string();
    let exata = channel.matches('.').count() >= 1
        && channel
            .split('.')
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()));
    assert!(
        exata,
        "channel = {channel:?} é canal flutuante. O arquivo existe para que a CI e a máquina \
         de desenvolvimento rodem o MESMO compilador: perseguindo \"stable\" dos dois lados, \
         uma stable nova reprova o PR por lint que o clippy local não conhece (iter 0016). \
         Subir de versão é iteração própria, nunca efeito colateral de outro item."
    );
    for componente in ["rustfmt", "clippy"] {
        assert!(
            toml.contains(componente),
            "{ARQUIVO} tem de declarar o componente {componente}: sem ele o passo \
             correspondente da CI não existe no toolchain instalado"
        );
    }
}

#[test]
fn ci_nao_instala_toolchain_por_conta_propria() {
    let ci = fs::read_to_string(support::repo_root().join(".github/workflows/ci.yml"))
        .expect("ci.yml deve existir");
    let flutuante = ci
        .lines()
        .map(str::trim)
        .filter(|l| !l.starts_with('#'))
        .find(|l| l.contains("rust-toolchain@") || l.contains("toolchain: "));
    assert!(
        flutuante.is_none(),
        "ci.yml escolhe o toolchain sozinho ({flutuante:?}): a versão tem de vir do {ARQUIVO} \
         e de nenhum outro lugar, senão os dois lados voltam a divergir sem ninguém perceber."
    );
}
