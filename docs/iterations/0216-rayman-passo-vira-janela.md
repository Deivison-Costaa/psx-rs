<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0216 — rayman-passo-vira-janela

- **Data:** 2026-08-07
- **Item do roadmap:** 0216.1 — preparo do Degrau 9 da escada de timing de CPU/barramento
- **Objetivo:** converter as asserções de passo absoluto do Rayman em `rayman_autoack.rs`,
  `rayman_exception_chain.rs` e `rayman_tty_boot.rs` (achado 10.115) para janelas, ANTES do
  Degrau 9 (DMA cobrando ciclos de verdade) tocar em `bus.rs` — exigência explícita do plano
  da escada, para não confundir uma melhoria legítima de timing com uma regressão real.

## Spec consultada

Não é spec de hardware — é manutenção de teste (R5 continua valendo: os testes precisam
medir o comportamento certo, não um artefato de calibração antiga). Nenhuma seção de
`docs/reference/` foi consultada nesta iteração.

## Erros de primeira tentativa

Nenhum na conversão em si — o padrão já estava estabelecido em `rayman_evcb_descritores.rs`
(convertido antes, também pelo achado 10.115). A parte não trivial foi perceber que
`rayman_tty_boot.rs::desligar_o_auto_ack_na_religada_faz_o_contador_de_vsync_andar` tinha um
problema mais fundo que a asserção final: o próprio GATE da lógica (`step > EXECUTE_STEP`)
usava uma constante calibrada pro timing antigo (164_000_000). Se o Degrau 9 deslocar o
`Execute!` real, o gate desalinha e passa a interceptar cedo ou tarde demais, corrompendo o
que o teste mede, não só o valor esperado. Corrigido com a mesma detecção dinâmica de
"Execute !" no TTY que o primeiro teste do arquivo já usava.

## Bateria de mutação

Bateria de mutação: não se aplica — nenhum arquivo em `crates/*/src/` foi tocado, só testes.

## Placar antes → depois

Workspace: **1344** testes (sem novos — só as 3 asserções de passo absoluto viraram janela,
mais o gate dinâmico do segundo teste de `rayman_tty_boot.rs`).

## Revisão cruzada (orquestrador)

Rodada de autoria do orquestrador. **Não pude rodar os 3 testes contra o disco real do
Rayman nesta máquina/sessão** — a imagem (`Rayman (USA) DADOS.cue`, gitignored, fornecida
pelo usuário) não está em `../roms/extraido/` neste ambiente. Verificação disponível:
compilação limpa (`cargo test --no-run`), `cargo fmt --all -- --check` e `cargo clippy
--all-targets -- -D warnings` limpos, e execução real confirmando o caminho de skip
gracioso ("BIOS ou disco Rayman nao encontrado — teste ignorado") sem panic nem erro de
tipo antes do `return` — não prova que as janelas estão certas contra dados reais, só que a
lógica compila e não quebra o esqueleto do teste. Fica registrado como limitação, não como
verificação completa.

## Decisões e notas

Janelas escolhidas por inspeção (140M-220M passos, ~140M de largura em torno dos valores
antigos ~164M-178M): generosas o bastante pra absorver o deslocamento que o Degrau 9 pode
causar (DMA charging afeta jogos com streaming pesado de CD-ROM como o Rayman de forma mais
que proporcional aos deslocamentos pequenos já vistos em 0185/0187, que eram de dezenas de
milhares de passos, não milhões). Se a janela se provar estreita demais depois do Degrau 9
rodar de verdade, é achado novo, não bug desta conversão.

Assercões de IDENTIDADE DE CÓDIGO (endereços, ordem de handlers visitados, conteúdo de nós
da cadeia de exceção, bits de STAT) permanecem exatas — não são fragilidade de timing, são
o que o teste realmente prova.

Próximo (Degrau 9, agora liberado pra tocar `bus.rs`): acumular `Dma::transfer_cost`
(Degrau 8) e somar em `Bus::tick_timers` antes de drenar o scheduler e antes de
`Timers::tick`. Rodar os oráculos `tests/exes/ps1-tests/dma`/`.../spu` antes/depois
(achado 10.114), e RE-VERIFICAR os 3 testes convertidos aqui contra o disco real do Rayman
assim que a máquina/sessão tiver a imagem disponível.
