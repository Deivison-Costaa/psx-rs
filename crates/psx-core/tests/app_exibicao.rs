use psx_core::app::config::Config;
use psx_core::app::exibicao::{
    DURACAO_DO_TOAST, MedidorDeQuadros, ModoDeImagem, Retangulo, Sobreposicao, Toast,
    retangulo_da_imagem,
};

const BASE: (f32, f32) = (320.0, 240.0);

#[test]
fn ajustar_a_janela_ocupa_o_maior_4_por_3_centralizado_na_janela_larga() {
    let r = retangulo_da_imagem((1600.0, 1000.0), BASE, ModoDeImagem::AjustarAJanela);
    assert_eq!(r.altura, 1000.0, "limitado pela altura");
    assert_eq!(
        r.largura, 1333.0,
        "4:3 de 1000 linhas, sem passar da janela"
    );
    assert_eq!(r.y, 0.0);
    assert_eq!(r.x, 133.0, "sobra horizontal dividida nos dois lados");
}

#[test]
fn ajustar_a_janela_alta_centraliza_na_vertical() {
    let r = retangulo_da_imagem((500.0, 900.0), BASE, ModoDeImagem::AjustarAJanela);
    assert_eq!((r.largura, r.altura), (500.0, 375.0));
    assert_eq!((r.x, r.y), (0.0, 262.0));
}

#[test]
fn janela_menor_que_a_imagem_encolhe_em_vez_de_cortar() {
    for modo in [ModoDeImagem::AjustarAJanela, ModoDeImagem::EscalaInteira] {
        let r = retangulo_da_imagem((200.0, 150.0), BASE, modo);
        assert_eq!(
            r,
            Retangulo {
                x: 0.0,
                y: 0.0,
                largura: 200.0,
                altura: 150.0
            },
            "{modo:?}"
        );
    }
}

#[test]
fn escala_inteira_usa_o_maior_multiplo_que_cabe() {
    let r = retangulo_da_imagem((1600.0, 1000.0), BASE, ModoDeImagem::EscalaInteira);
    assert_eq!((r.largura, r.altura), (1280.0, 960.0), "4x cabe, 5x nao");
    assert_eq!((r.x, r.y), (160.0, 20.0));
}

#[test]
fn imagem_nunca_passa_da_janela() {
    for (w, h) in [
        (1500.0, 950.0),
        (500.0, 380.0),
        (1920.0, 1080.0),
        (641.0, 479.0),
    ] {
        for modo in [ModoDeImagem::AjustarAJanela, ModoDeImagem::EscalaInteira] {
            let r = retangulo_da_imagem((w, h), (298.0, 224.0), modo);
            assert!(r.x >= 0.0 && r.y >= 0.0, "{w}x{h} {modo:?}: {r:?}");
            assert!(r.x + r.largura <= w, "{w}x{h} {modo:?}: {r:?}");
            assert!(r.y + r.altura <= h, "{w}x{h} {modo:?}: {r:?}");
        }
    }
}

#[test]
fn janela_ou_imagem_vazia_da_retangulo_vazio() {
    let vazio = retangulo_da_imagem((0.0, 0.0), BASE, ModoDeImagem::AjustarAJanela);
    assert_eq!(vazio.largura * vazio.altura, 0.0);
    let sem_imagem = retangulo_da_imagem((800.0, 600.0), (0.0, 0.0), ModoDeImagem::EscalaInteira);
    assert_eq!(sem_imagem.largura * sem_imagem.altura, 0.0);
}

#[test]
fn toast_fica_opaco_e_some_sozinho_depois_da_duracao() {
    let t = Toast::novo("slot 0 salvo", 10.0);
    assert_eq!(t.texto, "slot 0 salvo");
    assert_eq!(t.opacidade(10.0), 1.0);
    assert_eq!(t.opacidade(11.0), 1.0);
    let sumindo = t.opacidade(10.0 + DURACAO_DO_TOAST - 0.1);
    assert!(sumindo > 0.0 && sumindo < 1.0, "esmaece no fim: {sumindo}");
    assert!(!t.expirou(10.0 + DURACAO_DO_TOAST - 0.1));
    assert!(t.expirou(10.0 + DURACAO_DO_TOAST));
    assert_eq!(t.opacidade(20.0), 0.0);
}

#[test]
fn toast_com_relogio_para_tras_continua_visivel() {
    let t = Toast::novo("x", 10.0);
    assert_eq!(t.opacidade(5.0), 1.0);
    assert!(!t.expirou(5.0));
}

#[test]
fn f1_alterna_status_ajuda_e_nada() {
    let s = Sobreposicao::default();
    assert_eq!(s, Sobreposicao::Nenhuma, "barra escondida por padrao");
    assert_eq!(s.proxima(), Sobreposicao::Status);
    assert_eq!(s.proxima().proxima(), Sobreposicao::Ajuda);
    assert_eq!(s.proxima().proxima().proxima(), Sobreposicao::Nenhuma);
}

#[test]
fn medidor_de_quadros_so_publica_depois_de_meio_segundo() {
    let por_quadro = 566_187;
    let m = MedidorDeQuadros::default();
    assert_eq!(m.fps(), None);
    let m = m.acumula(por_quadro * 15, 0.25, por_quadro);
    assert_eq!(m.fps(), None, "um quarto de segundo ainda e pouco");
    let m = m.acumula(por_quadro * 15, 0.25, por_quadro);
    let fps = m.fps().expect("meio segundo publica");
    assert!((fps - 60.0).abs() < 1e-9, "30 quadros em 0,5 s: {fps}");
    let m = m.acumula(por_quadro * 5, 0.5, por_quadro);
    let lento = m.fps().expect("nova janela");
    assert!(
        (lento - 10.0).abs() < 1e-9,
        "a janela anterior foi zerada: {lento}"
    );
}

#[test]
fn medidor_ignora_ciclos_por_quadro_zero() {
    let m = MedidorDeQuadros::default().acumula(1000, 1.0, 0);
    assert_eq!(m.fps(), None);
}

#[test]
fn modo_de_imagem_padrao_e_ajustar_e_vai_para_o_toml() {
    assert_eq!(
        Config::default().modo_de_imagem,
        ModoDeImagem::AjustarAJanela
    );
    let c = Config {
        modo_de_imagem: ModoDeImagem::EscalaInteira,
        ..Config::default()
    };
    assert_eq!(c.ajustada().modo_de_imagem, ModoDeImagem::EscalaInteira);
    assert_eq!(ModoDeImagem::TODOS.len(), 2);
    assert_eq!(ModoDeImagem::AjustarAJanela.rotulo(), "Ajustar à janela");
    assert_eq!(ModoDeImagem::EscalaInteira.rotulo(), "Escala inteira");
}
