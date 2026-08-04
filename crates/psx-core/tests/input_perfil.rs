use psx_core::app::input_map::{Entrada, Perfil, SOLTO};
use psx_core::pad_script::button_bit;

fn bit(nome: &str) -> u16 {
    1u16 << button_bit(nome).expect("botao conhecido")
}

fn apertado(palavra: u16, nome: &str) -> bool {
    palavra & bit(nome) == 0
}

#[test]
fn nada_apertado_deixa_a_palavra_toda_em_um() {
    assert_eq!(Perfil::padrao().palavra(&[]), SOLTO);
}

#[test]
fn cada_face_do_controle_cai_no_botao_certo() {
    let p = Perfil::padrao();
    assert!(apertado(p.palavra(&[Entrada::Sul]), "cross"));
    assert!(apertado(p.palavra(&[Entrada::Leste]), "circle"));
    assert!(apertado(p.palavra(&[Entrada::Norte]), "triangle"));
    assert!(apertado(p.palavra(&[Entrada::Oeste]), "square"));
}

#[test]
fn um_botao_apertado_nao_aperta_os_outros() {
    let palavra = Perfil::padrao().palavra(&[Entrada::Sul]);
    assert_eq!(palavra, SOLTO & !bit("cross"));
}

#[test]
fn entradas_simultaneas_apagam_varios_bits() {
    let palavra =
        Perfil::padrao().palavra(&[Entrada::DpadCima, Entrada::Sul, Entrada::R1, Entrada::Start]);
    for nome in ["up", "cross", "r1", "start"] {
        assert!(apertado(palavra, nome), "{nome} deveria estar apertado");
    }
    assert!(!apertado(palavra, "down"));
    assert!(!apertado(palavra, "l1"));
}

#[test]
fn entrada_sem_ligacao_nao_muda_nada() {
    let p = Perfil::vazio("cru");
    assert_eq!(p.palavra(&[Entrada::Sul, Entrada::Start]), SOLTO);
}

#[test]
fn direcional_analogico_cai_no_direcional_digital() {
    let p = Perfil::padrao();
    assert!(apertado(p.palavra(&[Entrada::EixoNegativo(0)]), "left"));
    assert!(apertado(p.palavra(&[Entrada::EixoPositivo(0)]), "right"));
    assert!(apertado(p.palavra(&[Entrada::EixoNegativo(1)]), "down"));
    assert!(apertado(p.palavra(&[Entrada::EixoPositivo(1)]), "up"));
}

#[test]
fn dpad_e_analogico_podem_apontar_para_o_mesmo_botao() {
    let p = Perfil::padrao();
    assert_eq!(
        p.palavra(&[Entrada::DpadEsquerda]),
        p.palavra(&[Entrada::EixoNegativo(0)])
    );
}

#[test]
fn ligar_devolve_perfil_novo_sem_mexer_no_antigo() {
    let antigo = Perfil::vazio("meu");
    let novo = antigo.liga(Entrada::Norte, "start").expect("botao valido");
    assert_eq!(antigo.palavra(&[Entrada::Norte]), SOLTO);
    assert!(apertado(novo.palavra(&[Entrada::Norte]), "start"));
}

#[test]
fn religar_a_mesma_entrada_troca_o_botao_em_vez_de_somar() {
    let p = Perfil::vazio("meu")
        .liga(Entrada::Sul, "cross")
        .and_then(|p| p.liga(Entrada::Sul, "square"))
        .expect("botoes validos");
    let palavra = p.palavra(&[Entrada::Sul]);
    assert!(apertado(palavra, "square"));
    assert!(
        !apertado(palavra, "cross"),
        "uma entrada fisica so pode acionar um botao; remapear substitui"
    );
    assert_eq!(p.ligacoes().len(), 1);
}

#[test]
fn desligar_tira_so_a_entrada_pedida() {
    let p = Perfil::padrao().desliga(Entrada::Sul);
    assert_eq!(p.palavra(&[Entrada::Sul]), SOLTO);
    assert!(apertado(p.palavra(&[Entrada::Leste]), "circle"));
}

#[test]
fn botao_desconhecido_e_recusado_com_o_nome_no_erro() {
    let erro = Perfil::vazio("meu")
        .liga(Entrada::Sul, "turbo")
        .expect_err("turbo nao existe no pad digital");
    assert!(erro.contains("turbo"), "erro foi: {erro}");
}

#[test]
fn perfil_de_faces_trocadas_difere_so_nas_duas_faces() {
    let padrao = Perfil::padrao();
    let trocado = Perfil::faces_trocadas();
    assert!(apertado(trocado.palavra(&[Entrada::Sul]), "circle"));
    assert!(apertado(trocado.palavra(&[Entrada::Leste]), "cross"));
    for entrada in [Entrada::Norte, Entrada::Oeste, Entrada::L1, Entrada::Start] {
        assert_eq!(
            padrao.palavra(&[entrada]),
            trocado.palavra(&[entrada]),
            "{entrada:?} nao deveria mudar entre os dois perfis"
        );
    }
}

#[test]
fn perfil_sobrevive_a_ida_e_volta_por_serializacao() {
    let p = Perfil::padrao()
        .liga(Entrada::Modo, "select")
        .expect("botao valido");
    let bytes = bincode::serde::encode_to_vec(&p, bincode::config::standard()).expect("gravar");
    let (volta, _): (Perfil, usize) =
        bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).expect("ler");
    assert_eq!(volta, p);
}

#[test]
fn nome_do_botao_ligado_a_uma_entrada_pode_ser_consultado() {
    let p = Perfil::padrao();
    assert_eq!(p.nome_do_botao(Entrada::Sul), Some("cross"));
    assert_eq!(p.nome_do_botao(Entrada::Modo), None);
}

#[test]
fn perfil_vira_texto_de_linha_e_volta_igual() {
    let original = Perfil::padrao();
    let texto = original.para_texto();
    assert!(texto.contains("sul = cross"), "texto foi:\n{texto}");
    assert!(texto.contains("eixo-0-negativo = left"));
    let volta = Perfil::de_texto(&original.nome, &texto);
    assert_eq!(volta.ligacoes().len(), original.ligacoes().len());
    for entrada in [Entrada::Sul, Entrada::Start, Entrada::EixoPositivo(1)] {
        assert_eq!(volta.botao_de(entrada), original.botao_de(entrada));
    }
}

#[test]
fn texto_com_lixo_e_comentario_nao_derruba_a_leitura() {
    let p = Perfil::de_texto(
        "meu",
        "# comentario\nsul = cross\nlinha sem igual\nnorte = inexistente\n\nstart = start # fim\n",
    );
    assert_eq!(p.nome_do_botao(Entrada::Sul), Some("cross"));
    assert_eq!(p.nome_do_botao(Entrada::Start), Some("start"));
    assert_eq!(p.nome_do_botao(Entrada::Norte), None);
    assert_eq!(p.ligacoes().len(), 2);
}

#[test]
fn nome_de_entrada_vai_e_volta_para_todas_as_variantes() {
    for entrada in psx_core::app::input_map::TODAS_FIXAS {
        assert_eq!(Entrada::de_nome(&entrada.nome()), Some(entrada));
    }
    for n in [0u8, 1, 7] {
        assert_eq!(
            Entrada::de_nome(&Entrada::EixoNegativo(n).nome()),
            Some(Entrada::EixoNegativo(n))
        );
        assert_eq!(
            Entrada::de_nome(&Entrada::EixoPositivo(n).nome()),
            Some(Entrada::EixoPositivo(n))
        );
    }
    assert_eq!(Entrada::de_nome("nao-existe"), None);
    assert_eq!(Entrada::de_nome("eixo-x-negativo"), None);
}
