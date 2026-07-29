# 0038 — vram-transfers

- **Data:** 2026-07-28
- **Item do roadmap:** 2.2
- **Objetivo:** VRAM de 1 MB e transfers CPU↔VRAM (fill GP0(02h), A0h copy, C0h copy).

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Quick Rectangle Fill (L217) | docs/reference/03-gpu.md |
| psx-spx | § GPU Memory Transfer Commands (L603) | docs/reference/03-gpu.md |
| psx-spx | § Masking and Rounding for FILL (L640) | docs/reference/03-gpu.md |
| psx-spx | § Masking for COPY Commands (L664) | docs/reference/03-gpu.md |
| psx-spx | § Wrapping (L697) | docs/reference/03-gpu.md |
| psx-spx | § GP1(00h) Reset GPU (L747) | docs/reference/03-gpu.md |
| psx-spx | § Ready Bits (bits 26/27) (L1041) | docs/reference/03-gpu.md |
| psx-spx | § GP1(01h) Reset Command Buffer (L767) | docs/reference/03-gpu.md |
| psx-spx | § Mask setting afeta CPU→VRAM (L590) | docs/reference/03-gpu.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| G1 | hardware | GP1(00h) limpa a VRAM (`self.vram.fill(0)`) | GP1(00h) não afeta VRAM; o reset limpa fifo, ack irq, display off, dma off, display address, x1/x2, y1/y2, display mode, GP0(E1h..E6h) — L747-765 — e nem GP1(09h) é afetado | Revisão do orquestrador após o commit inicial |
| G3 | protocolo | `top3 == 4` (GP0(80h)) é no-op e as 3 palavras de parâmetro viram comandos | GP0(80h) é VRAM→VRAM blit com 4 palavras totais (comando + 3 params); deve consumir e ignorar as 3 palavras seguintes | Revisão do orquestrador após o commit inicial |
| M5 | API | Campo `pub stat: Cell<u32>` expõe mutabilidade interior | Deveria ser privado com getter `pub fn stat(&self) -> u32` | Revisão do orquestrador (G4) |
| M6 | regressão | Leituras de byte/halfword do GPUREAD não consomem a transferência | `read32(0x0)` é chamado via `bus.rs` para byte/halfword, consumindo palavras indevidamente | Revisão do orquestrador (G2) |
| M7 | teste | `a6` verificava pixel(3,0) para o extra descartado, mas o extra vai para pixel(0,1) (wrap de linha) | O halfword extra em contagem ímpar é escrito no início da PRÓXIMA linha, não coluna+1 | Bateria de mutação — mutante (e) sobreviveu com a asserção original, corrigida a célula verificada |
| R1 | teste | Os 19 testes verdes mediam a implementação | Sete mutações aplicadas e **rodadas** sobreviveram — ver a seção da revisão delegada | Revisão delegada + execução do orquestrador |
| R2 | hardware | GP1(01h) podia continuar no-op | L767-771 "Resets the command buffer and CLUT cache"; L753 lista `GP1(01h) ;clear fifo` dentro do próprio GP1(00h) | Revisor de consequência cruzada |
| R3 | hardware | O bit 26 só precisava cair nos comandos 02h/A0h/C0h | L1051-1053: cai sempre que a GPU espera parâmetros — inclusive nos 3 do GP0(80h) | Revisor de fidelidade à spec |
| R4 | regressão | `bus.rs` byte/halfword estava corrigido pelo `peek32` | O braço da GPU do `region_read_byte` ignorava o parâmetro `offset`, único da função a fazer isso: `lhu` do GPUSTAT devolvia 0x8080 em vez de 0x1480 | Revisor de consequência cruzada |
| R5 | regressão | `swl`/`swr` eram inertes para I/O | Faziam `bus.read32` antes do `is_isc()`; com a GPU stateful, um STORE passou a consumir palavra do GPUREAD, e mesmo com cache isolada. Mesmo padrão de ordem da iter 0022 (F3) | Revisor de consequência cruzada |
| P1 | processo | Uma rodada de continuação seguiria o task file | A rodada 3 recriou a branch a partir da `main` e destruiu quatro commits, dois do orquestrador | `git log` na revisão pós-rodada |
| P2 | processo | Bastava dizer no prompt que a rodada era de continuação | As rodadas 4, 5 e 6 leram o `STATUS.md`/PR (passo 0 do protocolo), concluíram que o item estava pronto e pararam ou recomeçaram do zero | Três rodadas seguidas sem commit |

## Bateria de mutação

Placar da bateria: 6/6 mutantes mortos, 2/2 controles verdes, 1 equivalente — docs/mutantes/0038-vram-transfers.mut

**Nota histórica:** o placar original (antes do ferramental 0.10/0.11) era 13/14 mutantes pegos, 1 equivalente, 2/2 controles verdes, conferido manualmente. O manifesto `.mut` formal contém os 9 registros (6 mutantes + 2 controles + 1 equivalente) que sobreviveram à revisão adversarial e à conferência de âncoras. Cada linha foi
aplicada no fonte, `cargo test --all --no-fail-fast` foi **rodado**, e a árvore restaurada com
`git checkout --` (restaurar por cópia de arquivo desfez correções uma vez nesta iteração).
Saída bruta em `scratchpad/bateria-saida.txt`; script em `scratchpad/bateria.py`.

| # | Mutação | Pego? | Teste que pegou |
|---|---|---|---|
| (a) | `& 0x3F0` → `& 0x3FF` na máscara de Xpos do fill | Sim | `a2_fill_arredonda_xpos_e_xsiz` |
| (b) | Remover arredondamento `+ 0x0F) & !0x0F` do Xsiz | Sim | `a2_fill_arredonda_xpos_e_xsiz` |
| (c) | `& 0x1FF` → `& 0x3FF` na máscara de Ypos do fill | **Equivalente** | Nenhum, e nenhum é possível — ver abaixo |
| (d) | Inverter low/high na palavra de 32 bits, **simetricamente** (escrita e leitura) | Sim | 8 testes; `a8` e `peek32` **não** matam |
| (e) | Escrever halfword extra em contagem ímpar (remover guard `remaining > 0`) | Sim | `a6_a0h_impar_descarta_halfword_extra` |
| (f) | Não ligar bit 27 no cabeçalho do C0h | Sim | `a10_gpustat_bit27_c0h` |
| (g) | `((xsiz-1) & 0x3FF) + 1` → `xsiz & 0x3FF`, nos dois sentidos | Sim | `a7`, `a0h_xsiz_1024` — **não** `a10` |
| (C1) | Ypos ignorado nos três caminhos (`let ypos = 0`) | Sim | `a0h_com_ypos_nao_zero...`, `c0h_le_da_linha_absoluta...`, `gp1_00h_reset_preserva_vram` |
| (C2) | Fill abortado quando o mask-bit está ligado | Sim | `a4_fill_nao_respeita_mask_bit` |
| (C3) | Fórmula `Ysiz=((Ysiz-1)&1FFh)+1` do COPY removida | Sim | `a0h_ysiz_513_mascara_para_1_linha` |
| (C4) | Stride da VRAM 1024 → 512 só nos caminhos de cópia | Sim | `a0h_com_ypos_nao_zero...`, `a8`, `c0h_le_da_linha_absoluta...` |
| (C5) | A0h descarta em vez de dar wrap em X | Sim | `a9_wrap_copy...`, `c0h_le_da_linha_absoluta...` |
| (C7) | `GP1(00h)` enche a VRAM de `0xFFFF` (o G1 em outra roupa) | Sim | `gp1_00h_reset_preserva_vram` |
| (C9) | C0h sem wrap em X na leitura | Sim | `c0h_le_da_linha_absoluta_e_com_wrap_em_x` |
| (K1) | *controle*: renomear o local `raw_w` → `w_raw` | Sobreviveu ✔ | — |
| (K2) | *controle*: renomear a fn privada `gpuread_word` → `fetch_gpuread_word` | Sobreviveu ✔ | — |

**O mutante (c) é equivalente, não um gap de cobertura** — a primeira versão deste doc
registrou errado. `ypos = raw_y & 0x1FF` (gpu.rs) é subsumido por
`ypos.wrapping_add(row) & 0x1FF` no laço: como 512 divide 2¹⁶, os 9 bits baixos da soma são
idênticos com qualquer das duas máscaras. Nenhum teste pode matá-lo, nem com Ypos ≥ 512.
O buraco real em Y era outro (C1) e foi fechado.

**Dois créditos da versão anterior estavam inflados**, e a diferença foi medida:
- (g) era creditado a `a10_gpustat_bit27_c0h`. `a10` usa size word `0x0001_0004` → Xsiz bruto 4;
  `((4-1)&0x3FF)+1 = 4` e `4&0x3FF = 4`. O teste não toca o valor mutado.
- (d) era creditado a `a8` e `peek32`. Os dois são round-trips A0h→C0h: invertida
  simetricamente na escrita e na leitura, a palavra sai como entrou. Rodado: os dois **passam**
  com a mutação aplicada. O mutante morre pelos oito testes de `vram_pixel` absoluto.

## Placar antes → depois

- **Antes:** 274 testes (base do M1)
- **Depois:** 300 testes
- Scoreboard: **5 com veredito (1p/4f), 45 só com saída, 0 sem saída, 1 não avaliados, de 51**

Mudança de 4→5 vereditos: entrou `ps1-tests/gpu/mask-bit/mask-bit.exe` com `fail 3p/2f`.
A primeira versão deste doc atribuía a mudança ao fix G2 (`peek32` em `bus.rs`). **Isso é
falso e o próprio `logs/scoreboard.csv` do repositório desmente:** o `mask-bit.exe` já
aparecia com veredito em `c143e77`, commit anterior ao `f4cc25e` que introduziu o `peek32`.
A causa é a feature 2.2 como um todo — a VRAM passou a existir e o GPUREAD a devolver dado
real, então o EXE passou a produzir um padrão de pass/fail reconhecível. O `fail` é legítimo
e esperado: o `mask-bit.exe` exercita o mask setting do GP0(E6h) aplicado a cópias, que é
justamente o que **não** está implementado (item 10.7).

## Revisão delegada a três agentes `claude -p` — piloto

A revisão deste PR foi delegada a três agentes headless read-only
(`--allowedTools "Read,Grep,Glob"`), com briefs disjuntos: (a) fidelidade à spec,
(b) consequência cruzada em quem chama o que mudou, (c) "o teste mede ou afirma a
implementação?". Cada um devolveu `arquivo:linha` **e a mutação literal que provaria o
achado**; quem rodou os comandos foi o orquestrador. Delegar a busca, nunca a verificação.

**Custo: US$ 4,8978 somados, contra US$ 0,1653 da rodada de trabalhador que revisaram —
cerca de 30×.** As três linhas estão em `docs/metricas.csv` com `fonte=revisor`.

Acharam 7 testes que não mediam e 4 defeitos de código que a revisão centralizada não tinha
achado. As sete mutações foram aplicadas e rodadas pelo orquestrador: **as sete sobreviveram**,
nenhuma era falso positivo. O achado mais importante é o (C7): o G1 — `GP1(00h)` apagando a
VRAM — foi corrigido nesta mesma iteração e o teste renomeado para
`gp1_00h_reset_preserva_vram`, verde. Mas ele usava `assert_ne!(pixel, 0)`, e `assert_ne!`
contra zero é satisfeito por qualquer valor: com `vram.fill(0xFFFF)` no reset — o mesmo
defeito vestido de outro jeito — o teste continuava passando. **O teste que existia para
provar a correção não provava a correção.**

Segundo em importância: **round-trip não mede endereçamento.** `a8` é A0h→C0h e compara a
implementação com ela mesma; o stride da VRAM pôde ser trocado de 1024 para 512 nos caminhos
de cópia sem que nenhum dos 19 testes notasse.

## Revisão cruzada (orquestrador)

Revisão delegada a três `claude -p` (seção acima), com toda mutação e todo comando rodados
pelo orquestrador. Os quatro defeitos de código (R2-R5) e os sete testes que não mediam (R1)
foram corrigidos nesta branch; a bateria final saiu 13/14 + 1 equivalente, 2/2 controles,
refeita com a árvore livre de processos concorrentes (nota 12).

## Decisões e notas

1. **Primeiro timeout do projeto:** a rodada anterior (trabalhador DeepSeek) estourou o limite de 45 min. Quando estourou, existiam 291 testes verdes (17 novos para 2.2), mas `cargo fmt` e `cargo clippy` estavam vermelhos, e não havia doc, nem bateria de mutação, nem PR. Os dois commits (`test(gpu):` e `feat(gpu):`) foram feitos pelo ORQUESTRADOR para preservar o trabalho. Por isso, a ordem R5 (teste-antes-de-implementação) não pôde ser verificada nesta iteração.

2. **Não existe EXE de hardware que meça este item.** `vram-to-vram-overlap` e 80h estão fora de escopo; `bandwidth` depende de timers. O critério de aprovação foi não-regressão no scoreboard.

3. **Uso de `Cell<u32>` para `stat`:** consequência de `Bus::read32` ser `&self`. O campo era `pub` por engano (G4); foi tornado privado com getter `pub fn stat(&self) -> u32`.

4. **`peek32`:** criado para resolver G2 — leituras de byte/halfword do bus não devem consumir a transferência C0h. Só acessos de 32 bits ao GPUREAD consomem.

5. **GP0(80h) VRAM→VRAM:** não implementado (fora do escopo 2.2, R4). As 4 palavras do comando (1 comando + 3 params) são consumidas e ignoradas via estado `SkipParams`. Acrescentado item 10.6 no ROADMAP.

6. **Correção do teste `a6`:** a asserção que verificava `pixel(3,0) == 0` foi corrigida para `pixel(0,1) == 0` — o halfword extra, se escrito, cai no início da próxima linha (wrap), não na coluna seguinte. Descoberto durante a bateria de mutação.

7. **`#![rustfmt::skip]` não funciona em integration tests** (unstable `custom_inner_attributes`). Em vez disso, usamos `#[rustfmt::skip]` em funções específicas com asserts longos, e encurtamos as mensagens das demais para caber no limite de 100 colunas sem quebra.

8. **Scoreboard 4→5 vereditos:** ver a seção "Placar antes → depois". A primeira versão desta nota inventava a causa (creditava ao `peek32`); a medição no `logs/scoreboard.csv` mostra que o `mask-bit.exe` já tinha veredito no commit anterior ao `peek32`.

9. **Seis rodadas de trabalhador, e o item foi fechado pelo orquestrador.** A distribuição importa mais que o total: a rodada 1 estourou o timeout de 45 min (primeiro do projeto); a 2 produziu a implementação, que passou na conferência de spec de primeira; a 3 recriou a branch e destruiu quatro commits; a 4 e a 5 leram o `STATUS.md` e o PR aberto, concluíram que o item estava pronto e pararam sem commitar; a 6 leu o `STATUS.md` já corrigido, entendeu "2.2 é a próxima tarefa" e reimplementou o item do zero. As correções desta iteração (7 testes reforçados + 4 defeitos) foram feitas pelo orquestrador.

   **O diagnóstico que demorei a fazer:** eu tratava isso como falha de prompt e reescrevia o cabeçalho do task file mais alto a cada rodada. Não era. O passo 0 do protocolo manda ler o `STATUS.md`, e ele — mais o checkbox do `ROADMAP` e o corpo do PR — dizia que o item estava concluído. **Handoff só funciona se o estado do repositório concordar com ele**; enquanto não concordava, o trabalhador seguia o repositório, corretamente. Corrigido em `db2420f`, mas aí a correção criou o problema simétrico: marcar 2.2 como "próxima tarefa" fez a rodada 6 começar do zero. O protocolo descreve só a iteração normal; falta-lhe um estado de "item em revisão".

10. **Custo por papel nesta iteração** (`docs/metricas.csv`): trabalhador US$ 0,42 em 6 rodadas, das quais 1 aproveitada; revisores delegados US$ 4,90 em 3 execuções. Nenhuma das falhas caras foi de emulação — a implementação de hardware saiu certa na primeira tentativa e passou na conferência máscara por máscara. O que consumiu a iteração foi teste que não mede, doc que inventa causa, e protocolo que não previa continuação.

11. **Restaurar árvore por cópia de arquivo desfez correções.** Durante a bateria, um `cp backup gpu.rs` reverteu silenciosamente os fixes D1/D2 que já estavam na árvore, e os testes só voltaram a falhar duas etapas depois. A bateria passou a restaurar com `git checkout -- <arquivo>`, que não tem esse modo de falha.

12. **Dois escritores na mesma árvore, sem ninguém saber.** O `oc-iter.ps1` reportou o fim da rodada 6 e devolveu o controle, mas o processo `opencode.exe` continuou vivo e commitando: os commits `7ee357e` e `0e132a8` são dele, feitos enquanto o orquestrador já editava os mesmos arquivos, e havia uma edição pendente em `gpu_status_gp0_gp1.rs` duplicando um teste que o orquestrador acabara de escrever em outro arquivo. Pior: a primeira execução da bateria de mutação rodou nessa janela, e uma mutação (`vram.fill(0xFFFF)`) sobrou na árvore — o teste `gp1_00h_reset_preserva_vram` passou a falhar sem causa aparente. **A bateria foi refeita do zero com a árvore só do orquestrador, e o resultado saiu idêntico** (13/14 + 1 equivalente, 2/2 controles), mas o primeiro resultado não era confiável e não teria como ser distinguido do bom sem repetir. Antes de confiar em medição de árvore compartilhada, conferir que não há processo de trabalhador vivo.

13. **A rodada 6, mesmo descartada, achou uma causa que o orquestrador não tinha visto.** Os dois commits dela corrigem o texto envoltório do `oc-iter.ps1`, que dizia `"Ao abrir o PR, PARE - nao faca merge, nao comece outro item."` (linha 44, verificado em `git show 7ee357e^`). Com um PR já aberto, isso lê como "já acabou" — foi o que devolveu as rodadas 4 e 5 sem uma linha escrita. O orquestrador tinha atribuído a culpa só ao `STATUS.md`; eram as duas coisas. Os commits foram mantidos, e o `oc-iter.ps1` ganhou o modo `-ContinueBranch`.
