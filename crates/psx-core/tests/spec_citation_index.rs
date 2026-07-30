use support::spec_citation_data::{IndexEntry, find_in_index, index_match_ambiguity};

mod support;

#[test]
fn find_in_index_prefere_titulo_mais_longo_nao_substring() {
    let index = vec![
        IndexEntry {
            offset_k: 99,
            title: "Opcode/Parameter Encoding".into(),
        },
        IndexEntry {
            offset_k: 127,
            title: "Coprocessor Opcode/Parameter Encoding".into(),
        },
    ];
    let result = find_in_index(&index, "Coprocessor Opcode/Parameter Encoding");
    assert!(result.is_some(), "deveria encontrar uma seção");
    assert_eq!(
        result.unwrap().offset_k,
        127,
        "deveria casar com 'Coprocessor Opcode/Parameter Encoding' (L127), \
         não com a substring 'Opcode/Parameter Encoding' (L99)"
    );
}

#[test]
fn find_in_index_prefere_casamento_exato_sobre_mais_longo() {
    let index = vec![
        IndexEntry {
            offset_k: 99,
            title: "Opcode/Parameter Encoding".into(),
        },
        IndexEntry {
            offset_k: 127,
            title: "Coprocessor Opcode/Parameter Encoding".into(),
        },
    ];
    let result = find_in_index(&index, "Opcode/Parameter Encoding");
    assert!(result.is_some(), "deveria encontrar uma seção");
    assert_eq!(
        result.unwrap().offset_k,
        99,
        "busca por título exato deve retornar o casamento exato (L99), \
         não o mais longo 'Coprocessor Opcode/Parameter Encoding' (L127)"
    );
}

#[test]
fn find_in_index_titulos_empatados_retorna_none() {
    let index = vec![
        IndexEntry {
            offset_k: 10,
            title: "Register Read Timing".into(),
        },
        IndexEntry {
            offset_k: 20,
            title: "Register Read Status".into(),
        },
    ];
    let result = find_in_index(&index, "Register");
    assert!(
        result.is_none(),
        "dois títulos de mesmo comprimento que ambos casam devem ser ambíguos"
    );
}

#[test]
fn index_match_ambiguity_detecta_empate() {
    let index = vec![
        IndexEntry {
            offset_k: 10,
            title: "Register Read Timing".into(),
        },
        IndexEntry {
            offset_k: 20,
            title: "Register Read Status".into(),
        },
    ];
    let amb = index_match_ambiguity(&index, "Register");
    assert!(
        amb.is_some(),
        "empate de comprimento deve ser reportado como ambíguo"
    );
    let msg = amb.unwrap();
    assert!(
        msg.contains("ambíguo"),
        "mensagem deve conter 'ambíguo', mas foi: {msg}"
    );
}
