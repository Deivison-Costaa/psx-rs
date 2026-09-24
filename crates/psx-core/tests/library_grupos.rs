use psx_core::app::library::{Ordem, Regiao, agrupa, casa_busca, detalhe, ordena, rotulo_do_disco};
use psx_core::app::sessao::Recentes;

fn nomes(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

#[test]
fn discos_do_mesmo_jogo_viram_um_grupo_em_ordem_de_disco() {
    let n = nomes(&[
        "Final Fantasy IX (USA) (Disc 2)",
        "Crash Bandicoot (USA)",
        "Final Fantasy IX (USA) (Disc 1)",
        "Final Fantasy IX (USA) (Disc 4)",
    ]);
    let g = agrupa(&n);
    assert_eq!(g.len(), 2);
    let ff9 = g
        .iter()
        .find(|x| x.titulo == "Final Fantasy IX (USA)")
        .expect("grupo do FF9");
    assert_eq!(ff9.membros, vec![2, 0, 3]);
    let crash = g
        .iter()
        .find(|x| x.titulo == "Crash Bandicoot (USA)")
        .expect("crash");
    assert_eq!(crash.membros, vec![1]);
}

#[test]
fn rotulo_do_disco_sai_do_nome() {
    assert_eq!(
        rotulo_do_disco("Final Fantasy IX (USA) (Disc 4)").as_deref(),
        Some("Disco 4")
    );
    assert_eq!(
        rotulo_do_disco("Jogo (Disk 2) (Rev 1)").as_deref(),
        Some("Disco 2")
    );
    assert_eq!(rotulo_do_disco("Crash Bandicoot (USA)"), None);
}

#[test]
fn disco_10_vem_depois_do_disco_9() {
    let n = nomes(&["J (Disc 10)", "J (Disc 9)"]);
    assert_eq!(agrupa(&n)[0].membros, vec![1, 0]);
}

#[test]
fn busca_ignora_caixa_acento_e_ordem_dos_termos() {
    assert!(casa_busca(
        "crash",
        &["Crash Bandicoot (USA)", "SCUS-94900"]
    ));
    assert!(casa_busca("BANDICOOT crash", &["Crash Bandicoot (USA)"]));
    assert!(casa_busca("pokemon", &["Pokémon Stadium"]));
    assert!(casa_busca("scus-949", &["Crash", "SCUS-94900"]));
    assert!(casa_busca("   ", &["qualquer"]));
    assert!(!casa_busca(
        "spyro",
        &["Crash Bandicoot (USA)", "SCUS-94900"]
    ));
}

#[test]
fn ordena_por_nome_ignorando_caixa() {
    let itens = vec![
        ("b".to_string(), None),
        ("A".to_string(), Some(5)),
        ("c".to_string(), None),
    ];
    assert_eq!(ordena(&itens, Ordem::Nome), vec![1, 0, 2]);
}

#[test]
fn ordena_por_recentes_poe_o_mais_novo_primeiro_e_o_nunca_jogado_no_fim() {
    let itens = vec![
        ("Zeta".to_string(), None),
        ("Alfa".to_string(), Some(10)),
        ("Beta".to_string(), Some(30)),
        ("Gama".to_string(), None),
    ];
    assert_eq!(ordena(&itens, Ordem::Recentes), vec![2, 1, 3, 0]);
}

#[test]
fn detalhe_nao_repete_o_serial_e_mostra_o_tempo() {
    assert_eq!(
        detalhe(Regiao::America, Some("SCUS-94900"), 2 * 3600 + 600),
        "NTSC-U · SCUS-94900 · jogado 2 h 10 min"
    );
    assert_eq!(detalhe(Regiao::Europa, Some("SLES-1"), 0), "PAL · SLES-1");
    assert_eq!(
        detalhe(Regiao::Desconhecida, None, 0),
        "disco sem identificação"
    );
}

#[test]
fn recentes_sabem_a_ultima_vez_de_cada_serial() {
    let r = Recentes::default()
        .registra("A", "a", 10, 100)
        .registra("B", "b", 10, 200);
    assert_eq!(r.ultima_vez_de("A"), Some(100));
    assert_eq!(r.ultima_vez_de("B"), Some(200));
    assert_eq!(r.ultima_vez_de("C"), None);
}
