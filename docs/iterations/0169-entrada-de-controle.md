<!-- Custo, tokens e duracao ficam em docs/metricas.csv, medidos pelo runner. -->

# 0169 — entrada-de-controle

- **Data:** 2026-08-02
- **Item do roadmap:** 10.96
- **Objetivo:** o `psx-cli` passa a conectar um pad digital e apertar botões em passos dados.
- **Fonte:** orquestrador.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Standard Controllers (L618) | docs/reference/10-controllers-memcards.md |

A ordem dos dezesseis bits do halfword 1 — `0 Select, 1 L3, 2 R3, 3 Start, 4 Up, 5 Right,
6 Down, 7 Left, 8 L2, 9 R2, 10 L1, 11 R1, 12 /\, 13 (), 14 ><, 15 []` — e a convenção
`0=Pressed, 1=Released` vêm dessa tabela, e um teste as fixa nome a nome.

## Por que esta rodada existe

O `psx-desktop` já ligava o pad (`connect_digital_pad`, `set_buttons`), mas o `psx-cli` **nunca**:
`Sio::pad_connected` nascia `false` e nada o ligava, então toda resposta a `01h`/`42h` era `0xFF`.
Duas hipóteses dependiam disso e as duas foram testadas aqui.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição diz | Como foi pego |
|---|---|---|---|---|
| 1 | hipótese | Que o Rayman estivesse parado numa tela de título esperando START — ele imprime `PS-X Control PAD Driver Ver 3.0`, passa ~1 s de splash e só então entra no laço. | Não é assunto de spec. | Com `--pad` e catorze apertos de START/X entre 250 M e 550 M passos, `[0x801CEEBC]` continua `0`, `[0x801CF2CC]` continua `2` e os `VSync: timeout` continuam **522**. Byte a byte igual ao run sem pad. **Refutada.** |
| 2 | hipótese | Que a suíte `psxtest_gte` estivesse num menu esperando START (foi o que uma sonda de investigação concluiu). | Não é assunto de spec. | Com pad e sete apertos de START, o TTY continua nas mesmas 3 linhas, parando em `Running tests`. Ela cai no mesmo travamento de `ResetGraph` das outras 21 suítes (item 10.95), não num menu. **Refutada.** |
| 3 | API-Rust | Que o teste vermelho pudesse simplesmente importar um módulo inexistente. | Não é assunto de spec. | R5 pede falha por **asserção**, e `unresolved import` é falha de compilação. Commitei o esqueleto da API junto do teste: 6 dos 8 testes falharam por asserção antes da implementação. |
| 4 | ferramenta | Que a âncora do manifesto sobrevivesse ao `cargo fmt`. | Não é assunto de spec. | O `fmt` reagrupou o array `BUTTONS` de dezesseis linhas para duas, e a âncora do `m5` casou 0 vezes. A bateria morreu no meio e deixou a sentinela `logs/mutantes-em-andamento.txt`; conferi que o fonte não ficara mutado antes de apagá-la. **Formatar antes de escrever âncora.** |

## Bateria de mutação

Placar da bateria: **7/7 mutantes mortos, 2/2 controles verdes, 0 equivalente** —
`docs/mutantes/0169-entrada-de-controle.resultado`.

Os mutantes atacam as duas bordas da janela de aperto (`m1`, `m2`), a inversão do sentido do bit
(`m3`, que é o erro clássico de pad ativo-baixo), a validação de duração (`m4`, `m7`), a ordem da
tabela da spec (`m5`) e a insensibilidade a caixa (`m6`). Nenhum morreu de erro de compilação.

## Placar antes → depois

Workspace: **930 → 938** testes.

Amidog e Rayman: **inalterados** — era o desfecho possível e é o que a rodada mediu.

## Revisão cruzada (orquestrador)

Rodada de autoria do orquestrador; autorrevisão registrada como limite. O que segura o resultado
não é opinião: as duas hipóteses foram testadas com o binário construído nesta rodada e as duas
caíram, com número. Registrar a refutação vale mais do que o código novo — o código apenas tornou
a refutação possível.

## Decisões e notas

A lógica de agenda ficou em `crates/psx-core/src/pad_script.rs` (dados puros, sem I/O, sem
dependência nova — passa no `purity.rs`) e o `psx-cli` só traduz argumentos. A janela de um aperto
é semiaberta `[início, início+duração)`, e a duração padrão são 2.000.000 de passos — cerca de
três quadros e meio, o bastante para o kernel amostrar o pad mais de uma vez.

O laço do runner só chama `set_buttons` quando a máscara muda, para não pagar por passo.

**O que estas duas refutações eliminam:** o Rayman não está esperando entrada, e a suíte de GTE
não está esperando entrada. Sobra o item **10.95** — o caminho `--exe` grava `jr $ra` em
`0x00A0/0x00B0/0x00C0` e um stub de seis instruções em `0x80000080`, de modo que a BIOS é
carregada mas nunca inicializa. É ele que segura as 22 suítes, e é a próxima rodada.
