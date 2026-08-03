mod support;

use std::fs;
use support::spec_citation_data::collect_md_files;

// Cinco trabalhadores em paralelo criam worktrees git em `.claude/worktrees/agent-*`, cada um
// com o acervo inteiro de `docs/iterations/`. O varredor de citacoes andava por dentro deles e
// reprovava a arvore limpa com 295 erros de documentos que nem sao desta arvore.
#[test]
fn varredor_nao_entra_em_worktree_aninhado() {
    let raiz = std::env::temp_dir().join(format!("psx-escopo-{}", std::process::id()));
    let _ = fs::remove_dir_all(&raiz);
    let proprio = raiz.join("docs/iterations");
    let alheio = raiz.join(".claude/worktrees/agent-x/docs/iterations");
    fs::create_dir_all(&proprio).expect("cria docs do proprio repo");
    fs::create_dir_all(&alheio).expect("cria docs do worktree aninhado");
    fs::write(proprio.join("9998-proprio.md"), "# proprio\n").expect("escreve doc proprio");
    fs::write(alheio.join("9999-alheio.md"), "# alheio\n").expect("escreve doc alheio");

    let achados = collect_md_files(&raiz);
    let _ = fs::remove_dir_all(&raiz);

    let nomes: Vec<String> = achados
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert!(
        nomes.iter().any(|n| n == "9998-proprio.md"),
        "o varredor tem de continuar achando os docs do proprio repositorio; achou {nomes:?}"
    );
    assert!(
        !nomes.iter().any(|n| n == "9999-alheio.md"),
        "o varredor entrou em `.claude/worktrees/`: docs de outra arvore nao sao desta rodada"
    );
}
