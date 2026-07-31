# 0112 — display-polaridade

- **Data:** 2026-07-30
- **Item do roadmap:** 2.10
- **Objetivo:** `framebuffer_for_display()` devolvia `None` com o display LIGADO — a guarda lia o
  GPUSTAT.23 invertido. Uma linha de produção, três arquivos de teste virados, três baterias.

## Revisão do PR anterior

PR #128 (iter 0111), do próprio orquestrador: quatro checks verdes, mergeado no início da rodada.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § GPU Status Register (L1001) — bit23: 0=Enabled, 1=Disabled | docs/reference/03-gpu.md |
| psx-spx | § GP1(03h) - Display Enable (L779) — param bit0: 0=On, 1=Off | docs/reference/03-gpu.md |

## O defeito, e a arqueologia dele

`gpu.rs:446` fazia `bit23 == 0 → None`. O handler de `GP1(03h)` sempre escreveu o bit CERTO
(0→limpa, 1→seta); só a leitura estava espelhada. O erro nasceu na iteração 0053 e foi
**fossilizado pelos próprios testes**: d1/d2 (0053), o teste de boot da 0090 ("GPU padrão tem
display ligado" — falso: reset deixa GPUSTAT.23=1 = desligado) e o controle "cosmético" K2 da
bateria 0053, que reescrevia a função preservando a inversão com fidelidade. Ninguém notava
porque, até a 0111, a BIOS nunca chegava a LIGAR o display — o mundo inteiro rodava com bit23=1,
onde a inversão por acaso mostrava imagem.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que o comportamento real é | Como foi pego |
|---|---|---|---|---|
| 1 | processo | Que virar o teste da 0090 para "None por padrão" bastasse | O flip removeu a única asserção que matava o mutante "sempre None" daquela bateria — **m2 da 0090 SOBREVIVEU** na primeira rodada | A bateria acusou; o teste ganhou a segunda metade (GP1(03h)=0 → `Some`) e o m2 morreu |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0112-display-polaridade.mut

A 0112 cobre o handler de GP1(03h) (comparação invertida, no-ops, bit errado). As baterias
0053 e 0090, que ancoravam na linha antiga, foram reancoradas na polaridade correta e
re-rodadas: **5/5 + 2/2 cada** (a 0090 só depois do reforço do teste, ver acima).

## Placar antes → depois

Workspace: **741** → **745** testes (4 novos em `gpu_display_polaridade.rs`).

## Revisão cruzada (orquestrador)

Iteração inteira do orquestrador.

## Decisões e notas

1. **Controle de mutação também envelhece.** O K2 da 0053 era "cosmético" em relação ao código
   errado; depois do fix, o mesmo K2 seria um MUTANTE. Reancorado com o espelhamento correto.
2. **Item novo 2.11** (aberto no ROADMAP): `display_height` devolve `y2-y1` cru; em 480i
   (GPUSTAT.19/22) são `(y2-y1)*2` linhas — sem isso o app mostra só a metade de cima da cena.
   É a última peça entre o fix e a tela do logo aparecer inteira na janela do psx-desktop.
