# 0017e — O teste de aceitação obrigatório estava errado (e era meu)

- **Data:** 2026-07-27
- **Item do roadmap:** 0.8 (orquestração; fora da escada do M1)
- **Objetivo:** Corrigir o valor literal que o orquestrador impôs como critério de aceitação
  do item 1.7, e registrar a falha.
- **Autor:** orquestrador (Claude). Sem código de emulador.

## O que aconteceu

Na revisão que reprovou a PR #27 eu criei uma regra de protocolo: quando a spec do item traz
um **idioma canônico**, o handoff passa a incluir um teste de aceitação com **valores
literais derivados da spec pelo orquestrador**, obrigatório no PR. A justificativa era a
assimetria — uma asserção com bytes concretos não pode ser satisfeita por um modelo mental
errado, enquanto uma que o próprio autor deriva sempre pode.

A regra está certa. O valor que eu escrevi estava errado.

Handoff publicado: `[0..3] = DD CC BB AA`, `[4..7] = 44 33 22 11`, `t0 = 1`, e o par
`lwl r2,3(t0)` / `lwr r2,0(t0)` teria de deixar **`r2 = 0x44DDCCBB`**.

Derivação correta da mesma spec:

- `lwl r2,3(t0)` → endereço `4`, `k = 0` → "transfer upper 8bit of Rt from `[N*4+0]`" →
  `r2[31:24] = mem[4] = 0x44`.
- `lwr r2,0(t0)` → endereço `1`, `k = 1` → "transfer lower 24bit of Rt from `[N*4+1..3]`" →
  `r2[7:0] = mem[1] = 0xCC`, `r2[15:8] = mem[2] = 0xBB`, `r2[23:16] = mem[3] = 0xAA`.
- Logo **`r2 = 0x44AABBCC`** — a palavra desalinhada no endereço 1, bytes `[1][2][3][4]`.

`0x44DDCCBB` usa `mem[0] = DD`, que não faz parte da palavra que começa em 1, e descarta
`mem[3] = AA`. É indexação misturada: base-0 para o byte do topo, base-1 para os três de
baixo.

## Como foi pego

Não foi por revisão do PR: foi ao carregar as duas seções da spec **antes** de o PR existir,
para ter a revisão pronta quando ele abrisse. Derivei a tabela das quatro posições por conta
própria e o resultado não bateu com o meu próprio handoff. Conferi por um segundo caminho
(a forma com deslocamento, `rt = (rt & ((1<<8*(3-k))-1)) | (palavra << 8*(3-k))`) e o
segundo caminho concordou com o primeiro, não comigo.

O trabalhador (v4-pro) estava em execução havia ~6 minutos, no passo 3–4, sem branch criada.
Morto na hora. Segunda rodada abortada na mesma noite — a primeira por troca de modelo
(0017d), esta por defeito de handoff.

## O que isso diz sobre o processo

1. **A regra da 0017 sobrevive; a execução dela ganhou um passo.** Valor literal no handoff
   continua sendo a assimetria certa contra "autor satisfaz o próprio erro". Mas um valor
   literal errado é pior que nenhum: vira critério **obrigatório**, e teria produzido pela
   segunda vez seguida um PR onde teste e implementação concordam e ambos estão errados —
   desta vez por indução minha, com o agravante de o trabalhador ter feito tudo certo.
   Passa a valer: **todo valor literal que o orquestrador impõe é derivado duas vezes, por
   caminhos diferentes** (leitura da tabela e forma algébrica), e o handoff carrega a
   derivação, não só o resultado, para que o trabalhador possa reprovar o orquestrador.
2. **O ganho veio de preparar a revisão cedo.** Carregar a spec durante a espera do
   trabalhador, em vez de na chegada do PR, é o que transformou um defeito entregue num
   defeito abortado. Vira prática: o orquestrador lê a seção da spec do item **enquanto** a
   iteração roda.
3. **"Reprovei a 0017 por um erro que eu também cometi."** A PR #27 confundia a via de byte;
   eu confundi a origem dos bytes na composição. A diferença é só quem pegou: o trabalhador
   não tinha ninguém revisando por cima, eu tinha a spec. Isso é argumento a favor da revisão
   cruzada, não a favor de o revisor ser confiável.

O achado que reprovou a PR #27 **não muda**: o resultado medido lá foi `0x00CCBBAA`, sem o
byte `0x44` no topo — a contribuição do `lwl` sumia inteira, o que independe de o esperado
ser `0x44AABBCC` ou qualquer outra coisa com topo não-zero.

## O que entrou

- `STATUS.md`: valor corrigido para `0x44AABBCC`, mais a tabela completa das quatro posições
  de LWL e LWR, a forma algébrica equivalente, a regra dos stores e um **segundo** teste de
  aceitação (round-trip `swl`/`swr` → `lwl`/`lwr`), que é auto-verificável e cobre os dois
  opcodes de escrita que o primeiro teste não toca.
- `docs/iterations/0017-cpu-unaligned-load-store.md` e `docs/orquestracao.md`: valor corrigido
  onde aparecia, com nota de que o texto original estava errado. Correção com rastro, não
  apagamento — o erro é dado do projeto.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que era | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | A palavra desalinhada em 1 seria `0x44DDCCBB` | `0x44AABBCC`; `mem[0]` não entra e `mem[3]` não sai | Derivação independente da tabela da spec, conferida pela forma algébrica |

## Bateria de mutação

Não se aplica (documento e handoff). O controle equivalente foi a **dupla derivação**: dois
caminhos independentes chegando ao mesmo valor.

## Placar antes → depois

151 testes → **151**.
