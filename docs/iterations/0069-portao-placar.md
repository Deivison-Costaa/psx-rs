# 0069 — portao-placar

- **Data:** 2026-07-29
- **Item do roadmap:** 10.12
- **Objetivo:** fechar as saídas silenciosas do portão que reconcilia o placar escrito à mão com
  o `.resultado` gerado pela máquina.

## Spec consultada

Nenhuma — item de processo, sem hardware envolvido.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição diz | Como foi pego |
|---|---|---|---|---|
| 1 | processo | Que a dívida 10.12 fosse um buraco **latente**: "pula em silêncio se o doc não for legível", sem incidente conhecido | Não era latente. O portão estava pulando **todos os 25 manifestos** na máquina local, sempre, desde a iteração 0041 | O teste novo reprovou de primeira no repositório real, com 25 erros do tipo `docs/iterations/0038-.md nao pode ser lido` — o slug saía vazio |
| 2 | API-Rust | Que `support::relative()` devolvesse caminho com barra normal | `path.display()` usa o separador **nativo**: no Windows o `rel` é `docs\mutantes\NNNN-slug.mut`, então todo `strip_prefix("docs/mutantes/")` falha e o `unwrap_or("")` engole a falha em silêncio | Mesmo teste. O `unwrap_or("")` é o que transforma um erro de caminho em string vazia sem ninguém perceber |
| 3 | processo | Que eu pudesse demonstrar a diferença editando o fonte com um `python -c` de substituição | O escape de `'\\'` dentro da string do script não casou, e o run "sem o conserto" na verdade ainda tinha o conserto — o resultado parecia provar o contrário do que provava | Conferi a linha com `sed -n` antes de acreditar no resultado, e o `.replace` ainda estava lá. Refeito com edição explícita |

## O que estava quebrado

`bateria_placar_bate_com_resultado` monta o nome do doc da iteração a partir do caminho relativo
do manifesto:

```rust
let stem = rel.strip_prefix("docs/mutantes/")
              .and_then(|s| s.strip_suffix(".mut"))
              .unwrap_or("");
```

No Windows `rel` chega com `\`, o `strip_prefix` falha, `unwrap_or("")` devolve vazio, o caminho
montado é `docs/iterations/0059-.md`, o arquivo não existe, e o `Err(_) => continue` pula o
manifesto **sem dizer nada**. Vinte e cinco manifestos, vinte e cinco pulos, toda vez que alguém
roda `cargo test --all` localmente.

Consequência prática: o trabalhador roda a suíte inteira antes de abrir o PR (passo 7 do
protocolo) e **o placar que ele acabou de escrever no doc não é conferido por ninguém**. O portão
só existia na CI, onde o separador já é `/`. É exatamente o sintoma que a dívida 10.12 descrevia
como "passou local e reprovou na CI na iter 0042" — a causa era esta, e não a leitura do doc.

O caso do PR #75 hoje fecha o argumento: o doc afirmava `3/3 controles verdes` com dois controles
no manifesto, passou local, e foi pego pela CI. Não foi sorte de a CI ser mais rigorosa; é que
localmente o portão nunca rodou.

## A prova de que o portão agora pega algo

Padrão exigido pelo plano: o portão tem de reprovar contra um caso real, não bastar como script
que alguém lembra de rodar. Corrompi o placar de `docs/iterations/0059-timers-sync.md` de
`7/7 mutantes mortos, 2/2 controles verdes` para `9/9 mutantes mortos, 3/3 controles verdes` — uma
mentira sobre um manifesto que tem 7 mutantes e 2 controles — e rodei o mesmo teste nas duas
versões de `relative()`:

| `relative()` | Placar mentindo 9/9 e 3/3 | Veredito |
|---|---|---|
| sem normalizar (como estava) | `bateria_placar_bate_com_resultado ... ok` | portão é no-op |
| normalizando `\` → `/` | `bateria_placar_bate_com_resultado ... FAILED` | portão pega |

Depois disso, `git checkout -- docs/iterations/0059-timers-sync.md` (restauração por git, nunca
por cópia de backup — regra que este projeto já pagou para aprender).

## O portão novo

`crates/psx-core/tests/mutation_reconciliation.rs`, um teste só,
`reconciliacao_do_placar_nao_pode_ser_pulada`, que para cada manifesto exige:

1. o doc de iteração pareado pelo prefixo de 4 dígitos existe e é legível;
2. o doc tem `Placar da bateria:` **ou** a linha de não-aplicabilidade;
3. existe pelo menos um `docs/mutantes/NNNN*.resultado`;
4. o `.resultado` é legível e parseia.

As quatro são as quatro condições sob as quais a reconciliação existente desiste em silêncio.
Ele não substitui `bateria_placar_bate_com_resultado`: garante que aquele teste nunca mais possa
ser pulado sem que alguém saiba. Um portão que verifica que o outro portão está de pé.

`parse_resultado` e `ResultadoRow` saíram de `mutation_battery.rs` para
`support/mutation_format.rs` porque passaram a ter dois consumidores. Duas cópias do parser
divergiriam — o plano deste projeto já registra essa lição sobre implementar as mesmas regras
duas vezes.

## Bateria de mutação

Bateria de mutação: não se aplica — o alvo desta iteração são arquivos de teste
(`crates/psx-core/tests/`), e o formato de manifesto exige alvo sob `crates/*/src/`, justamente
para que mutar o teste não seja a versão trapaceável do exercício. A verificação equivalente aqui
é a tabela da seção anterior: o portão foi exercitado contra uma mentira real e contra o estado
correto, nas duas versões do código.

## Placar antes → depois

Workspace: **548** → **549** testes (+1: `mutation_reconciliation`).
Portões que rodam de fato na máquina local: `bateria_placar_bate_com_resultado` passa de
**0 manifestos conferidos** para **25**.

## Revisão cruzada (orquestrador)

<!-- Preenchido na revisão do PR. -->

## Decisões e notas

1. **O conserto foi na raiz, não no consumidor.** Daria para trocar `strip_prefix("docs/mutantes/")`
   por algo tolerante a `\` em cada um dos lugares onde aparece; normalizar em `relative()` conserta
   todos de uma vez e impede que o próximo consumidor repita o erro. Rodei a suíte inteira depois:
   549 testes verdes, nenhum meta-teste dependia do separador nativo.
2. **`unwrap_or("")` é o padrão de risco aqui, não o separador.** O separador foi o gatilho; o que
   transformou um erro em silêncio foi engolir a falha do `strip_prefix` com uma string vazia. Vale
   procurar os outros `unwrap_or("")` sobre caminhos nos meta-testes — não fiz nesta iteração por
   R4, e a busca virou o item 10.25.
3. **Este é o segundo portão do dia que se descobriu não medir o que dizia medir**, junto com o
   `gpu_scoreboard.rs` da iteração 0068. Os dois passaram por revisão adversarial quando entraram.
   A diferença é que este era falsificável por uma experiência de dois minutos, e ninguém a fez até
   hoje — inclusive eu, que descrevi a 10.12 como buraco latente ainda ontem.
