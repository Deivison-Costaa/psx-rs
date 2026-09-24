use psx_core::app::input_map::Entrada;
use psx_core::app::pausa::{
    Acao, Comando, ITENS_DE_PAUSA, ItemDePausa, MenuDePausa, comando_do_controle, pede_pausa,
    slot_vizinho,
};

fn menu_em(item: ItemDePausa) -> MenuDePausa {
    let i = ITENS_DE_PAUSA
        .iter()
        .position(|x| *x == item)
        .expect("item existe");
    MenuDePausa::default().seleciona(i)
}

#[test]
fn menu_abre_em_continuar_com_todos_os_itens() {
    let m = MenuDePausa::default();
    assert_eq!(m.item(), ItemDePausa::Continuar);
    for item in [
        ItemDePausa::SalvarEstado,
        ItemDePausa::CarregarEstado,
        ItemDePausa::Slot,
        ItemDePausa::EstadosSalvos,
        ItemDePausa::MemoryCard,
        ItemDePausa::TrocarDisco,
        ItemDePausa::Controles,
        ItemDePausa::Ajustes,
        ItemDePausa::Atalhos,
        ItemDePausa::SairDoJogo,
    ] {
        assert!(ITENS_DE_PAUSA.contains(&item), "{item:?} falta no menu");
        assert!(!item.rotulo().is_empty());
    }
    assert_eq!(ITENS_DE_PAUSA.last(), Some(&ItemDePausa::SairDoJogo));
}

#[test]
fn setas_andam_e_dao_a_volta() {
    let m = MenuDePausa::default();
    let (acima, acao) = m.aplica(Comando::Cima);
    assert_eq!(acao, None);
    assert_eq!(acima.item(), ItemDePausa::SairDoJogo, "de cima para o fim");
    let (abaixo, _) = acima.aplica(Comando::Baixo);
    assert_eq!(abaixo.item(), ItemDePausa::Continuar, "do fim para o topo");
    assert_eq!(m.item(), ItemDePausa::Continuar, "o original nao muda");
}

#[test]
fn enter_executa_o_item_e_esc_continua() {
    let (_, acao) = menu_em(ItemDePausa::SalvarEstado).aplica(Comando::Confirma);
    assert_eq!(acao, Some(Acao::Executa(ItemDePausa::SalvarEstado)));
    let (_, acao) = menu_em(ItemDePausa::Continuar).aplica(Comando::Confirma);
    assert_eq!(acao, Some(Acao::Continua));
    let (_, acao) = menu_em(ItemDePausa::Ajustes).aplica(Comando::Volta);
    assert_eq!(acao, Some(Acao::Continua));
}

#[test]
fn laterais_no_slot_trocam_o_slot_e_fora_dele_nao_fazem_nada() {
    let slot = menu_em(ItemDePausa::Slot);
    assert_eq!(slot.aplica(Comando::Direita).1, Some(Acao::MudaSlot(1)));
    assert_eq!(slot.aplica(Comando::Esquerda).1, Some(Acao::MudaSlot(-1)));
    assert_eq!(slot.aplica(Comando::Confirma).1, Some(Acao::MudaSlot(1)));
    let outro = menu_em(ItemDePausa::MemoryCard);
    assert_eq!(outro.aplica(Comando::Direita), (outro, None));
}

#[test]
fn sair_pede_confirmacao_e_o_padrao_e_nao() {
    let (pergunta, acao) = menu_em(ItemDePausa::SairDoJogo).aplica(Comando::Confirma);
    assert_eq!(acao, None, "sair nunca e imediato");
    assert_eq!(pergunta.confirmacao, Some(false), "cursor comeca no 'nao'");
    let (cancelado, acao) = pergunta.aplica(Comando::Confirma);
    assert_eq!(acao, None);
    assert_eq!(cancelado.confirmacao, None);
    assert_eq!(cancelado.item(), ItemDePausa::SairDoJogo);

    let (sim, _) = pergunta.aplica(Comando::Esquerda);
    assert_eq!(sim.confirmacao, Some(true));
    let (fim, acao) = sim.aplica(Comando::Confirma);
    assert_eq!(acao, Some(Acao::SaiDoJogo));
    assert_eq!(fim.confirmacao, None);
}

#[test]
fn esc_na_confirmacao_so_fecha_a_pergunta() {
    let (pergunta, _) = menu_em(ItemDePausa::SairDoJogo).aplica(Comando::Confirma);
    let (sim, _) = pergunta.aplica(Comando::Baixo);
    let (volta, acao) = sim.aplica(Comando::Volta);
    assert_eq!(acao, None, "Esc na pergunta nao continua o jogo");
    assert_eq!(volta.confirmacao, None);
}

#[test]
fn slot_vizinho_da_a_volta_nos_dez() {
    assert_eq!(slot_vizinho(0, -1, 10), 9);
    assert_eq!(slot_vizinho(9, 1, 10), 0);
    assert_eq!(slot_vizinho(4, 1, 10), 5);
    assert_eq!(slot_vizinho(3, 0, 0), 0, "sem slots nao divide por zero");
}

#[test]
fn start_mais_select_pede_pausa_so_na_borda() {
    let nada: Vec<Entrada> = vec![];
    let start = vec![Entrada::Start];
    let os_dois = vec![Entrada::Start, Entrada::Select];
    assert!(pede_pausa(&start, &os_dois));
    assert!(pede_pausa(&nada, &os_dois));
    assert!(!pede_pausa(&os_dois, &os_dois), "segurar nao repete");
    assert!(!pede_pausa(&nada, &start), "so Start e do jogo");
}

#[test]
fn controle_navega_o_menu_na_borda() {
    let nada: Vec<Entrada> = vec![];
    let casos = [
        (Entrada::DpadCima, Comando::Cima),
        (Entrada::DpadBaixo, Comando::Baixo),
        (Entrada::DpadEsquerda, Comando::Esquerda),
        (Entrada::DpadDireita, Comando::Direita),
        (Entrada::Sul, Comando::Confirma),
        (Entrada::Leste, Comando::Volta),
    ];
    for (entrada, comando) in casos {
        assert_eq!(
            comando_do_controle(&nada, &[entrada]),
            Some(comando),
            "{entrada:?}"
        );
        assert_eq!(
            comando_do_controle(&[entrada], &[entrada]),
            None,
            "segurado"
        );
    }
}
