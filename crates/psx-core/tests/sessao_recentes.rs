use psx_core::app::sessao::{
    self, LIMITE_DE_RECENTES, Recentes, VELOCIDADE_MAX, formata_tempo, passos_por_quadro,
    proxima_velocidade,
};

fn com_tres() -> Recentes {
    Recentes::default()
        .registra("SLUS-00005", "Rayman", 60, 1000)
        .registra("SCUS-94900", "Crash", 30, 2000)
        .registra("SLUS-00594", "Tekken", 10, 3000)
}

#[test]
fn lista_vazia_nao_conhece_ninguem() {
    let r = Recentes::default();
    assert!(r.itens().is_empty());
    assert_eq!(r.tempo_de("SLUS-00005"), 0);
}

#[test]
fn o_ultimo_jogado_fica_no_topo() {
    let r = com_tres();
    let seriais: Vec<&str> = r.itens().iter().map(|i| i.serial.as_str()).collect();
    assert_eq!(seriais, ["SLUS-00594", "SCUS-94900", "SLUS-00005"]);
}

#[test]
fn rejogar_sobe_para_o_topo_sem_duplicar() {
    let r = com_tres().registra("SLUS-00005", "Rayman", 5, 4000);
    assert_eq!(r.itens().len(), 3);
    assert_eq!(r.itens()[0].serial, "SLUS-00005");
    assert_eq!(r.itens()[0].ultima_vez, 4000);
}

#[test]
fn tempo_de_jogo_acumula_entre_sessoes() {
    let r = com_tres().registra("SLUS-00005", "Rayman", 90, 4000);
    assert_eq!(r.tempo_de("SLUS-00005"), 150);
    assert_eq!(r.tempo_de("SCUS-94900"), 30);
}

#[test]
fn registrar_devolve_lista_nova_sem_mexer_na_antiga() {
    let antiga = com_tres();
    let nova = antiga.registra("SLES-00001", "Outro", 1, 5000);
    assert_eq!(antiga.itens().len(), 3);
    assert_eq!(nova.itens().len(), 4);
}

#[test]
fn lista_para_de_crescer_no_limite_e_perde_o_mais_antigo() {
    let mut r = Recentes::default();
    for i in 0..(LIMITE_DE_RECENTES + 3) {
        r = r.registra(&format!("SLUS-{i:05}"), "Jogo", 1, 1000 + i as u64);
    }
    assert_eq!(r.itens().len(), LIMITE_DE_RECENTES);
    assert_eq!(
        r.itens()[0].serial,
        format!("SLUS-{:05}", LIMITE_DE_RECENTES + 2)
    );
    assert_eq!(r.tempo_de("SLUS-00000"), 0, "o mais antigo saiu da lista");
}

#[test]
fn jogo_sem_serial_nao_entra_na_lista() {
    let r = Recentes::default().registra("", "Sem serial", 60, 1000);
    assert!(
        r.itens().is_empty(),
        "sem serial nao ha como acumular tempo"
    );
}

#[test]
fn titulo_novo_substitui_o_antigo_do_mesmo_serial() {
    let r = Recentes::default()
        .registra("SLUS-00005", "rayman.cue", 10, 1000)
        .registra("SLUS-00005", "Rayman (USA)", 10, 2000);
    assert_eq!(r.itens()[0].titulo, "Rayman (USA)");
}

#[test]
fn multiplicador_de_velocidade_multiplica_os_passos() {
    assert_eq!(passos_por_quadro(1000, 1), 1000);
    assert_eq!(passos_por_quadro(1000, 4), 4000);
}

#[test]
fn multiplicador_zero_nao_congela_o_emulador() {
    assert_eq!(
        passos_por_quadro(1000, 0),
        1000,
        "zero passo por quadro deixaria o jogo parado sem o usuario saber por que"
    );
}

#[test]
fn velocidade_cicla_e_volta_para_um() {
    let mut v = 1;
    let mut vistas = vec![v];
    for _ in 0..3 {
        v = proxima_velocidade(v);
        vistas.push(v);
    }
    assert_eq!(vistas, [1, 2, 4, 8]);
    assert_eq!(proxima_velocidade(VELOCIDADE_MAX), 1);
}

#[test]
fn velocidade_desconhecida_volta_para_um() {
    assert_eq!(proxima_velocidade(3), 1);
    assert_eq!(proxima_velocidade(0), 1);
}

#[test]
fn tempo_e_escrito_na_unidade_que_faz_sentido() {
    assert_eq!(formata_tempo(0), "0 s");
    assert_eq!(formata_tempo(59), "59 s");
    assert_eq!(formata_tempo(60), "1 min");
    assert_eq!(formata_tempo(3599), "59 min");
    assert_eq!(formata_tempo(3600), "1 h 00 min");
    assert_eq!(formata_tempo(7 * 3600 + 5 * 60), "7 h 05 min");
}

#[test]
fn recentes_sobrevivem_a_ida_e_volta_por_serializacao() {
    let r = com_tres();
    let bytes = bincode::serde::encode_to_vec(&r, bincode::config::standard()).expect("gravar");
    let (volta, _): (Recentes, usize) =
        bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).expect("ler");
    assert_eq!(volta, r);
}

#[test]
fn velocidades_disponiveis_sao_potencias_de_dois() {
    assert_eq!(sessao::VELOCIDADES, [1, 2, 4, 8]);
}
