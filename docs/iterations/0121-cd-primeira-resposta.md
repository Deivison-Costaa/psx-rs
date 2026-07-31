<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0121 — cd-primeira-resposta

- **Data:** 2026-07-31
- **Item do roadmap:** 4.4p
- **Objetivo:** entregar a primeira resposta do CD-ROM pelo `scheduler`, com o atraso da spec, em
  vez de dentro da escrita no porto — e medir se o `GetID` passa a aparecer.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § First Response (L2047) | docs/reference/06-cdrom.md |
| psx-spx | § First Response (INT3) (or INT5 if failed) (L1984) | docs/reference/06-cdrom.md |
| psx-spx | § SUB-CPU Mainloop (L1939) | docs/reference/06-cdrom.md |
| psx-spx | § BUSYSTS flag (L450) | docs/reference/06-cdrom.md |

Da § First Response saem os dois únicos números que a implementação usa:
*"Nop (normal) 000c4e1h 0004a73h..003115bh"* — média **0xC4E1 = 50 401** ciclos — e
*"Init 0013cceh 000f820h..00xxxxxh"* — **0x13CCE = 81 102**, porque *"the Init command ... is
doing some initialization before sending the 1st response"*. A mesma seção diz que
*"Timings for most other commands should be similar as above"*, o que justifica um só valor para
todo o resto.

## O que mudou

`cdrom.rs` era 100% atemporal: 22 campos `Cell`, todo método `&self`, e `send_command` publicava
`intsts` e a result FIFO dentro da própria escrita no porto. A correção **não reescreve
`send_command`** — ela adia a hora em que ele roda:

- a escrita na porta 1 / banco 0 passa a **latchar** o comando (`pending_cmd`) junto de um
  **snapshot da FIFO de parâmetros**, e a anunciar isso ao bus por `take_issued_command()`;
- `bus.rs` ganha o evento `CDROM_RESPONSE`, agendado em
  `total_cycles + Cdrom::first_response_cycles(cmd)`, e um braço em `tick_timers` que chama
  `deliver_first()` e levanta a IRQ2 pela borda;
- `scheduler.rs` ganha `cancel(EventId)`, de três linhas, para que exista **no máximo uma**
  resposta pendente por vez. Sem isso, um comando novo dentro da janela deixaria o evento antigo
  vivo, e ele venceria cedo entregando a resposta do comando errado.

## O resultado, que é o item inteiro

Harness `cdstate`, mesma BIOS e mesmo disco de sempre, 400 M passos:

```
  ANTES (iter 0120)                     DEPOIS
  Test (0x19)   x1                      Test (0x19)   x1
  GetStat(0x01) x1                      GetStat(0x01) x29
  — nada mais em 312 M passos —         GetID (0x1A)  x15

  passo 86989710   Test (0x19) params=[20]
  passo 87464254   GetStat (0x01)
  passo 87917831   GetID (0x1A)        <-- o comando que faltava
```

A sequência `GetStat → GetID` é exatamente a da referência do DuckStation preservada na 0120. O
critério de aceitação do item era o sintoma, não o relógio, e ele foi cumprido.

Na janela do `GetStat`, a interrupção deixou de pré-emptar o driver:

| | antes | depois |
|---|---|---|
| `sw` do comando | passo 87 464 254 | passo 87 464 254 |
| primeira leitura do HSTS pelo driver | passo 87 464 412 | passo 87 484 584 |
| entrada no handler com `I_STAT=4` | passo 87 464 **256** | fora da janela de 2 k passos |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | Que o BUSYSTS devia ficar alto durante toda a janela da resposta — é o desenho "óbvio" para um comando em voo, e foi o que a revisão adversarial do desenho também recomendou. | § SUB-CPU Mainloop (L1939) de `docs/reference/06-cdrom.md` separa as duas coisas: *"4. Command busy flag is unset and parameter fifo is cleared. 5. Shortly after (around 1000-6000 cycles later), CDROM IRQ is fired."* O busy cai **antes** da IRQ; ele é sobre a transmissão do comando (§ BUSYSTS, L450: *"1=Busy sending a command/parameters"*), não sobre a latência da resposta. | Leitura da spec antes de codificar, e o item 10.47 do backlog: laços de espera da BIOS têm orçamento de `0x8000 = 32 768` giros. Segurar o BUSYSTS por 50 401 ciclos seria entregar um timeout de presente. A semântica de `busy` ficou intacta, e o teste virou o contrário do que eu ia escrever: *não* pode ficar preso alto. |
| 2 | teste | Que `resposta_chega_no_prazo_medio_da_spec` media a IRQ2 saindo da entrega. | — | O mutante m7 (entrega não levanta IRQ2) **sobreviveu** à primeira bateria. A leitura do HINTSTS troca de banco, e essa escrita no porto chama `service_cdrom_irq` — era ela que levantava a IRQ, não o evento. Bastava afirmar o `I_STAT` antes de tocar nos portos de novo. Teste consertado, não mutante. |
| 3 | API-Rust | Que dava para o bus perguntar "houve comando?" com um `bool` e calcular o prazo com o comando. | — | O prazo depende do comando (`Init` é diferente), e um `bool` não carrega o byte. Colapsado em `take_issued_command() -> Option<u8>`, o que de quebra eliminou a trinca de flags que eu tinha desenhado (`pending_cmd` + `has_pending_cmd` + `command_issued`) e que era desincronização esperando para acontecer. |
| 4 | processo | Que o custo nos testes antigos seriam 55 edições, uma por teste que lê a resposta na instrução seguinte ao comando. | — | Três arquivos já tinham helper local `send_command`, o que cobriu 42 sítios com uma linha cada. O custo real foi 6 pontos de edição. A contagem de 55 estava no handoff como se fosse trabalho; era superfície. |

## Bateria de mutação

Placar da bateria: 7/7 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0121-cd-primeira-resposta.mut

| # | Mutação | Teste que pegou |
|---|---|---|
| m1 | comando executado dentro da escrita no porto (o defeito original) | `nenhuma_resposta_na_mesma_instrucao_da_escrita_do_comando` |
| m2 | atraso genérico vai a zero | `nenhuma_resposta_um_ciclo_antes_do_prazo_da_spec` |
| m3 | `Init`/`ReadTOC` usam o atraso genérico | `init_espera_o_atraso_maior_da_spec` |
| m4 | entrega não restaura os parâmetros capturados | `limpar_a_fifo_de_parametros_na_janela_nao_altera_o_comando_em_voo` |
| m5 | emissão captura FIFO de parâmetros vazia | `limpar_a_fifo_de_parametros_na_janela_nao_altera_o_comando_em_voo` |
| m6 | agenda sem cancelar a pendência anterior | `comando_novo_na_janela_nao_faz_o_evento_velho_entregar_cedo` |
| m7 | entrega no prazo não levanta a IRQ2 | `resposta_chega_no_prazo_medio_da_spec` (só depois de consertado — ver erro 2) |
| c1 | ordem dos comandos no braço do atraso maior (cosmético) | verde |
| c2 | tipo do prazo anotado (cosmético) | verde |

## Placar antes → depois

Workspace: 790 → **800** testes (9 em `cdrom_primeira_resposta.rs`, 1 em `bus_scheduler.rs`;
6 arquivos de CD-ROM passaram a avançar o relógio), 0 falhas.

## Revisão cruzada (orquestrador)

O desenho foi submetido a uma revisão adversarial antes de virar código. Ela achou três coisas
reais, todas corrigidas antes do primeiro commit de implementação, e uma que foi recusada:

- **Aceito — a API não compunha** (erro 1ª tentativa nº 3).
- **Aceito — corrida nos parâmetros.** Com a execução adiada, um `HCLRCTL` bit 6 chegado dentro
  da janela limparia a FIFO e o `Test 20h` viraria `Test 00h` em silêncio. Resolvido com
  snapshot na emissão; teste e dois mutantes (m4, m5) cobrem.
- **Aceito — testes negativos vazios.** `dma3_nao_dispara_sem_bfrd` e três irmãos afirmam que a
  RAM não muda; um drive morto também não muda a RAM. As pré-condições de `read_n_and_int1` e
  `preparar_cdrom_para_dma3` passaram a afirmar que o INT1 chegou. É reforço, não relaxamento.
- **Recusado — BUSYSTS alto na janela** (erro 1ª tentativa nº 1).
- **Recusado por R4 — o portão de "no INT pending".** § First Response (INT3) (L1984), `docs/reference/06-cdrom.md`, diz que o
  sub-CPU só executa o comando *"AND, there is no INT pending"*. Nós executamos de qualquer
  jeito, e isso é divergência real — mas **pré-existente**, não introduzida aqui: hoje o
  `send_command` já pisa num INT não reconhecido. Vira item próprio.

Outras notas do diff:

- **Âncoras reancoradas.** O controle K2 de `0062-cdrom-regs.mut` e o mutante m2 de
  `0080-scheduler-vblank-irq0.mut` apontavam para linhas que esta iteração mudou. Ambos foram
  atualizados preservando o que mediam; sem isso o `mutation_anchors` reprova.
- **Gates:** `purity`, `file_size`, `comment_density`, `roadmap_size`, `status_size`,
  `spec_citations`, `mutation_manifest`, `mutation_anchors` e `mutation_battery` verdes.
- **Árvore limpa**, `crates/psx-core/src/bin` removido antes do commit.

## Decisões e notas

- **Hipótese plausível virou causa provada.** A 0120 registrou a entrega em zero ciclo como
  defeito real por três razões, mas escreveu explicitamente que plausível não é provado
  (invariante 26). Aqui o sintoma sumiu: o `GetID` aparece. É o caso raro em que a invariante 26
  fecha para o lado positivo, e vale registrar que ela levou **duas** iterações de diagnóstico
  para chegar num item de código de tamanho normal.
- **O boot ainda não passa daqui, e o próximo bloqueio já estava lido no código.** A cadeia
  `GetStat, GetStat, GetID` se repete a cada ~18,9 M passos, para sempre — assinatura de quem
  recebe resposta ruim e tenta de novo. `cdrom.rs` responde ao `GetID` sempre com
  `INT5(08h,40h,00h×6)`, que a § GetID (L1145) de `docs/reference/06-cdrom.md` identifica como a linha **No Disk**, mesmo com
  disco dentro. Item 4.4q.
- **Buracos conhecidos deixados abertos (R4), todos com número:** o portão de INT pendente
  (10.53); a segunda resposta ainda dirigida pelo ack do guest e não por tempo, apesar de a
  § Second Response (L2066) de `docs/reference/06-cdrom.md` dar os números (10.54); `Nop (when stopped) = 0x5CF4`, que faz o
  atraso depender do motor (10.55); e a result FIFO antiga, que continua legível durante a
  janela (10.56).
- **Unidades.** A spec mede em *"33MHz units on a PAL PSone"* e nós contamos ciclos de CPU em
  framing NTSC (33,8688 MHz). A diferença é de 2,6% sobre 50 401 ciclos e está bem dentro da
  faixa min/max da própria tabela.
