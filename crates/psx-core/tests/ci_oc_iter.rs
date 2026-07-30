mod support;

use std::fs;

fn oc_iter_content() -> String {
    let path = support::repo_root().join("scripts/oc-iter.ps1");
    fs::read_to_string(&path).expect("scripts/oc-iter.ps1 deve existir")
}

fn parametro_inteiro(script: &str, nome: &str) -> i64 {
    let agulha = format!("[int]${nome} = ");
    let pos = script
        .find(&agulha)
        .unwrap_or_else(|| panic!("oc-iter.ps1 deve declarar o parametro [int]${nome}"));
    let resto = &script[pos + agulha.len()..];
    let fim = resto
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(resto.len());
    resto[..fim]
        .parse()
        .unwrap_or_else(|_| panic!("valor de ${nome} deve ser inteiro"))
}

#[test]
fn espera_da_rodada_nao_e_um_waitforexit_cego_de_parede_inteira() {
    let script = oc_iter_content();

    assert!(
        !script.contains("WaitForExit($TimeoutMin"),
        "a espera nao pode ser um unico `WaitForExit($TimeoutMin * 60 * 1000)`: bloqueado assim, \
         o loop nao consegue distinguir rodada lenta de rodada travada e paga a parede inteira \
         pelas duas. Medido em 30/07: duas rodadas morreram com 89 s e 100 s de vida ativa e \
         seguraram o loop por ~43 min cada uma sem emitir um unico evento."
    );
    assert!(
        script.contains("HasExited"),
        "a espera deve ser um laco que observa o processo (`HasExited`), nao uma chamada cega"
    );
}

#[test]
fn espera_observa_o_crescimento_do_json_da_rodada() {
    let script = oc_iter_content();

    assert!(
        script.contains("(Get-Item $outFile).Length"),
        "o sinal de vida da rodada e o JSON de saida crescendo: a espera tem de LER \
         `(Get-Item $outFile).Length` a cada volta. Sem ler o tamanho, nao ha como saber que o \
         provedor parou de responder."
    );
    let pos_len = script.find("(Get-Item $outFile).Length").unwrap_or(0);
    let resto = &script[pos_len..];
    assert!(
        resto.contains("$ultimoAvanco"),
        "o tamanho lido tem de alimentar uma marca de ultimo avanco (`$ultimoAvanco`); \
         ler o tamanho e descartar nao detecta travamento nenhum"
    );
}

#[test]
fn travamento_tem_rotulo_proprio_distinto_de_timeout() {
    let script = oc_iter_content();

    assert!(
        script.contains("falha:travamento"),
        "rodada morta por travamento tem de receber rotulo PROPRIO (`falha:travamento`) na linha \
         de metrica. Sem rotulo distinto, `docs/metricas.csv` mistura 45 min de trabalho lento \
         com 90 s de provedor mudo, e as duas exigem remedios opostos."
    );
    assert!(
        script.contains("falha:timeout"),
        "o rotulo `falha:timeout` deve continuar existindo para a rodada que trabalhou ate a parede"
    );
}

#[test]
fn janela_de_travamento_e_menor_que_a_parede_da_rodada() {
    let script = oc_iter_content();
    let travamento = parametro_inteiro(&script, "TravamentoMin");
    let parede = parametro_inteiro(&script, "TimeoutMin");

    assert!(
        travamento > 0,
        "a janela de travamento tem de ser positiva (valor atual: {travamento})"
    );
    assert!(
        travamento < parede,
        "a janela de travamento ({travamento} min) tem de ser MENOR que a parede da rodada \
         ({parede} min). Igual ou maior, o detector nunca dispara antes do timeout e o conserto \
         inteiro vira decoracao."
    );
}
