mod support;

use std::fs;

const ALVO: &str = "crates/psx-core/tests/testevent_descritor.rs";
const NOME_DA_CONSTANTE: &str = "PASSOS_ENTRE_SONDAGENS";

const MINIMO: usize = 1_000;
const MAXIMO: usize = 100_000;

fn fonte() -> String {
    fs::read_to_string(support::repo_root().join(ALVO))
        .unwrap_or_else(|_| panic!("{ALVO} deve existir"))
}

fn passo_de_sondagem(fonte: &str) -> usize {
    let agulha = format!("const {NOME_DA_CONSTANTE}: usize = ");
    let pos = fonte.find(&agulha).unwrap_or_else(|| {
        panic!("{ALVO} deve declarar `const {NOME_DA_CONSTANTE}: usize = <n>;`")
    });
    let resto = &fonte[pos + agulha.len()..];
    let fim = resto
        .find(|c: char| !c.is_ascii_digit() && c != '_')
        .unwrap_or(resto.len());
    resto[..fim]
        .replace('_', "")
        .parse()
        .unwrap_or_else(|_| panic!("valor de {NOME_DA_CONSTANTE} deve ser inteiro"))
}

#[test]
fn sondagem_nao_roda_a_cada_passo() {
    let n = passo_de_sondagem(&fonte());
    assert!(
        n >= MINIMO,
        "{NOME_DA_CONSTANTE} = {n}: sondar a cada {n} passos nao paga o custo. A varredura da \
         tabela EvCB gasta ~66 leituras de barramento por vez, contra 1-2 do cpu.step, e o laco \
         so encontra o evento no passo 86.988.128 — 87 M de varreduras sem nada para achar"
    );
}

#[test]
fn sondagem_nao_pode_cegar_o_teste() {
    let n = passo_de_sondagem(&fonte());
    assert!(
        n <= MAXIMO,
        "{NOME_DA_CONSTANTE} = {n} passa do teto de {MAXIMO}. O teto vem de MEDICAO, nao de \
         gosto: a condicao 'spec=20h e spec=8000h ambos prontos' foi observada estavel de \
         87,0 M a 87,9 M, ou seja por pelo menos 900 k passos. Amostrar acima disso arrisca \
         pular a janela inteira e o teste passa a falhar por cegueira, nao por regressao"
    );
}

#[test]
fn laco_usa_a_constante_como_guarda() {
    let f = fonte();
    let guarda = format!("% {NOME_DA_CONSTANTE}");
    assert!(
        f.contains(&guarda),
        "{ALVO} declara {NOME_DA_CONSTANTE} mas nao a usa em nenhuma guarda de resto (`{guarda}`) \
         do laco. Constante declarada e nao usada deixa os outros dois testes verdes enquanto a \
         sondagem continua rodando a cada passo"
    );
    assert!(
        f.contains("if !deve_sondar(_step) {"),
        "{ALVO} deve decidir a sondagem por `deve_sondar(_step)`. A decisao mora numa fn pura de \
         proposito: `deve_sondar` e testavel SEM BIOS e SEM disco, entao a bateria de mutacao a \
         alcanca na CI. Guarda escrita direto no laco so teria como oraculo o teste caro, que a \
         CI PULA por falta dos arquivos — e mutante que sobrevive por falta de oraculo mede o \
         ambiente, nao o codigo"
    );
}
