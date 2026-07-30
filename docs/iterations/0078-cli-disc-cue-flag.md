# 0078 — cli-disc-cue-flag

- **Data:** 2026-07-29
- **Item do roadmap:** 4.3c
- **Objetivo:** adicionar flag --disc ao psx-cli para aceitar caminho de .cue (ou .bin cru), montar DiscLayout via parse_cue e injetar no Cdrom.

## Revisão do PR anterior

Revisão do PR anterior (#91, iter 0077): sem achados. Padrões conferidos:
1. Teste que não mede — asserções concretas (valores de bytes 0xDEADBEEF do BIN, não round-trip nem assert_ne!)
2. Parâmetro não consumido — sem novos comandos GP0
3. Regra de borda trocada — sem rasterização
4. Campo de bit lido errado — bcd_to_int lê nibbles corretamente; offset 0x10 no setor raw correto
5. Panic ou laço ilimitado — read_sector_from_disc tem guarda data_end > bin.len(); sem unwrap/expect
6. Citação de spec — confere-citacoes.ps1 verde
7. Escopo transbordado — item 4.3b bem delimitado, _layout placeholder documentado
8. Portão que não mede — bateria 5/5, .resultado rastreado
9. Manifesto arquivado — 0065 ancora reparada (indentação de 20→24), não arquivada

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| (nenhuma) | Flag de CLI não tem spec de hardware | — |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | manifesto | Que o parser de manifesto aceita linhas vazias em blocos @@DE | O parser do mutation_format.rs filtra linhas vazias antes de entrar nos blocos @@DE/@@PARA (linha 175) | mutation_anchors reprovou m5: ancora esperada 1 vez, encontrada 0 |
| 2 | manifesto | Que mutantes.ps1 aceita testes do psx-cli | O script Invoke-CargoTest roda `cargo test -p psx-core` fixo (linha 290) | Bateria manual: script deu erro para m1-m4, Die no m5 |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente - ./docs/mutantes/0078-cli-disc-cue-flag.mut

| Registro | Rótulo | Testes que pegaram |
|---|---|---|
| m1 | --disc nao eh parseado no laco de argumentos | disc_flag_cue_minimo_aceito_com_bios |
| m2 | load_disc nao resolve BIN relativo ao diretorio do CUE | disc_flag_cue_minimo_aceito_com_bios |
| m3 | imprime numero de faixas errado (tracks.len() + 1) | disc_flag_cue_minimo_aceito_com_bios |
| m4 | load_disc ignora o CUE e devolve layout vazio (0 faixas) | disc_flag_cue_minimo_aceito_com_bios |
| m5 | --disc nao exige --bios (validacao removida) | disc_flag_sem_bios_erro |
| c1 | renomeia load_disc para load_disc_from_cue (cosmetico) | sobreviveu |
| c2 | acrescenta comentario no topo de load_disc (cosmetico) | sobreviveu |

## Placar antes → depois

Workspace: **564** → **567** testes (+3: `disc_flag_cue_minimo_aceito_com_bios`, `disc_flag_sem_bios_erro`, `disc_flag_arquivo_cue_inexistente_erro`).

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo orquestrador. -->

## Decisões e notas

1. O flag `--disc` exige `--bios` (erro claro se usado sozinho). Com `--bios --disc` (sem `--exe`), imprime info do disco e injeta no Bus.
2. Com `--bios --disc --exe`, o disco é injetado antes de carregar o EXE, para uso futuro (4.4).
3. `mutantes.ps1` não suporta testes do `psx-cli` — bateria rodada manualmente. Dívida registrada como 10.33.
4. Parser de manifesto (`mutation_format.rs`) descarta linhas vazias em blocos @@DE (linha 175: `if line.is_empty()` antes do `collecting_de`). Workaround: evitar linhas vazias nos blocos.
