<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0208 — timer-acumulador-fracionario

- **Data:** 2026-08-06
- **Item do roadmap:** 0208.1 (achado novo, urgência do usuário)
- **Objetivo:** corrigir o acumulador fracionário de `Timers::tick()`, que fazia o Timer 0/1
  em modo dotclock/hblank saltar milhares de unidades por chamada em vez de ~1 por pulso,
  travando o boot de praticamente todos os jogos comerciais testados.

## Como o achado surgiu

O usuário pediu pra rodar os 14 jogos comerciais disponíveis (`../roms/extraido/`) e medir
quantos abrem de verdade — só o Crash Bandicoot funcionava, os outros travavam na tela
SCEA/PlayStation ou com a tela preta. Rodada com `--dump-vram-every`/`--max-steps` de até
1,6 bilhão de passos (4x o primeiro teste) confirmou que não era boot lento: os 13 travam de
verdade. Lendo o TTY real da BIOS, todos completam o boot (`SYSTEM.CNF` lido, executável
carregado, `"Execute !"` impresso) — o travamento é dentro do código do próprio jogo, depois
do handoff.

A pedido explícito do usuário ("pode usar até um workflow, isso precisa ser resolvido
urgente"), rodei um `Workflow` de 5 agentes em paralelo, um por jogo (ff7, tekken3, re2,
tomb-raider, ctr), cada um caçando o PC onde o jogo trava e o que ele espera (via
`--sample-pcs`, `--watch-mem`, `--dump-mem`), seguido de um agente de síntese. Achado: **2 dos
5 (Tekken 3 e Resident Evil 2) travam no MESMO laço**, um idiom clássico da SDK PsyQ que lê o
Timer 1 duas vezes seguidas e só sai quando as leituras batem — exatamente o workaround
documentado na spec (ver abaixo). Os outros 3 (FF7, Tomb Raider, CTR) têm causas diferentes,
não fechadas nesta rodada (ver "Decisões e notas").

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Dotclock/Hblank (L79-86) | docs/reference/05-timers.md |
| psx-spx | § 1F801104h+N*10h - Timer 0..2 Counter Mode (L14-53) | docs/reference/05-timers.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | aritmética | Que `let total = prev_acc * denom + cycles*numer` era só uma forma equivalente de acumular o resto fracionário antes de dividir por `denom` de novo. | A spec não trata a fórmula interna do emulador — mas o próprio `cycle_acc` já é salvo como `total % denom` (linha 189), ou seja, já está em unidades de `numer`, não escalado por `denom`. Multiplicar de novo por `denom` e depois dividir cancela a escala e injeta o resto inteiro como pulsos extras a cada chamada. | Um dos 5 agentes do workflow leu `crates/psx-core/src/timers.rs` linha a linha depois de medir, no RE2, que o Timer 1 avançava ~242 unidades por iteração de um laço de 8 instruções — impossível pra um pulso hblank real (~1 a cada 2172 ciclos). Confirmei lendo o código eu mesmo antes de tocar em qualquer linha. |
| 2 | teste | Que os valores esperados em `timers_dotclock_hblank.rs` (76 e 7224 nos segundos asserts de dois testes) estavam certos porque o teste já existia e passava. | R5 exige golden values derivados da spec, não do output da implementação. | Os próprios comentários do teste ("1100+30\*70=3200/70=45" e "2+floor((7218\*23891+10000\*11)/23891)=2+7222=7224") são a fórmula BUGADA por escrito — o teste foi calibrado para bater com o código errado, não com o hardware. Recalculado à mão: 47 e 6. |
| 3 | mutação | Que os testes existentes bastavam pra matar qualquer mutante razoável na fórmula. | Não é assunto de spec. | `scripts/mutantes.ps1 -Iter 0208`: m6 (troca `7 * vcs` por `8 * vcs` no denom do hblank) sobreviveu — os cycle counts dos testes existentes davam o mesmo resultado final com os dois denominadores por coincidência aritmética. Acrescentei um teste que cruza o limiar exato (`2172*11=23892` cruza `7*3413=23891` mas não `8*3413=27304`) pra fixar o denominador de verdade. |

## Bateria de mutação

Placar da bateria: 6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente —
docs/mutantes/0208-timer-acumulador-fracionario.mut

m1 reintroduz o bug original (escala o resto por `denom` de novo) — morto pelos golden values
corrigidos e pelo teste de duas-leituras. m2 soma +1 espúrio por chamada, m3 troca divisão por
resto no cálculo de pulsos efetivos, m4 troca resto por divisão no acumulador guardado, m5
troca a razão do dotclock (11/7→12/7) — todos mortos pelos valores exatos dos testes
existentes. m6 (razão do hblank 11/7→11/8) só morreu depois do teste de limiar acrescentado.

Reexecutada por âncora envelhecida (a fórmula mudou de linha): `0059-timers-sync` — 7/7
mutantes, 2/2 controles, sem regressão; só o controle K2 (comentário sobre renomear a
variável, que citava a linha antiga) precisou de ajuste de texto, sem mudança de
comportamento.

## Placar antes → depois

Workspace: **1264 → 1266** testes (2 novos em `timers_dotclock_hblank.rs`: o double-read real
e o teste de limiar do denom).

## Revisão cruzada (orquestrador)

Rodada de autoria do orquestrador; a revisão adversarial é do mesmo agente que escreveu — fica
registrado em vez de escondido, como em 0159/0206/0207. O que dá pra afirmar com medição
independente do julgamento de quem escreveu: os 3 testes reescritos falhavam contra o código
antes do fix (confirmado rodando `cargo test` antes de tocar em `timers.rs`) e passam depois;
a bateria de mutação (rodada via script, não por inspeção) confirma que cada linha tocada tem
pelo menos um teste que a mata.

**Não medi ainda se o fix realmente destrava Tekken 3/RE2 rodando os jogos de verdade** —
isso é o próximo passo lógico, fora do escopo desta rodada de código (que termina no fix +
teste + bateria, por protocolo). Ficou anotado no handoff do STATUS.md.

## Decisões e notas

O bug afeta só Timer 0 (clock_src∈{1,3}, dotclock) e Timer 1 (clock_src∈{1,3}, hblank), onde
`denom` é grande (na casa das centenas a dezenas de milhares, `7 * video_cycles_per_scanline`
ou `7 * gpu_cycles_per_pix`). Para o caso "system clock" (denom=1, a maioria dos usos de timer
em jogos que não mexem no `clock_src`), a fórmula bugada e a correta coincidem
(`prev_acc*1+cycles*numer == prev_acc+cycles*numer`) — isso explica por que a divergência de
"System Clock" já registrada em `docs/achados.md` (0203.3) nunca apontou pra cá: aquele
achado mede outro contador, sem essa fonte de clock.

**Os outros 3 casos investigados pelo workflow ficam abertos, com causas diferentes:**
- **FF7**: trava num laço de "espera contador atingir alvo" em `0x80059DFC-0x80059E10`, lendo
  RAM `0x80089D9C` (não é registrador de hardware). As únicas escritas observadas nesse
  endereço vêm de dentro da própria imagem da BIOS, não do jogo nem de um handler de IRQ
  identificável — quem deveria escrever ali além da BIOS não foi isolado.
- **Tomb Raider**: não trava num spin de PC único — o jogo roda ~900 iterações de um laço de
  frame completo procurando `\FMV\CORELOGO.FMV;1` (que existe no `.bin`, confirmado por
  grep), desiste e imprime "not found". Duas hipóteses não fechadas: bug na entrega de setor/
  diretório ISO9660 do CD-ROM, ou descasamento do modelo de ciclos (achados já abertos
  0193.4/10.102/10.114/10.116/0203.3) fazendo o contador de retentativas do próprio jogo
  estourar num número de instruções diferente do esperado.
- **CTR**: o IRQ0/VBlank de hardware dispara e é reconhecido corretamente (confirmado via
  `--watch-mem` em I_STAT/I_MASK, cadência estável de frame por >20M passos) — mas o contador
  de software de vsync do próprio jogo (RAM `0x8007DD9C`) nunca incrementa. O elo que falha
  (handler encadeado ou EvCB específico do jogo) não foi isolado.

Esses três — junto com os outros 8 jogos travados que o workflow não investigou
individualmente (FF8, FF9, GT2, MGS, RE3, Silent Hill, Tomb Raider II/III) — continuam
travados até serem investigados um a um. O fix desta rodada é sobre uma causa raiz
**confirmada e específica** (Tekken 3 + RE2), não uma correção geral de boot.
