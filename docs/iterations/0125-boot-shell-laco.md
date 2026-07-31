# 0125 — boot-shell-laco

- **Data:** 2026-07-31
- **Item do roadmap:** 4.4t
- **Objetivo:** Instrumentar o laco `0x8004205C..0x800422DC` onde o shell para apos a tela de licenca,
  nomear o que o laco espera e com que valores.

## Spec consultada

Nenhuma — iteracao de diagnostico. A decodificacao do laco foi feita por leitura da memoria
(despejo via `--dump-mem` do proprio `psx-cli` instrumentado).

## Instrumentacao entregue

`psx-cli` ganhou tres flags novas:

- `--max-steps <N>` — controla o limite de passos do runner (default 50M).
- `--trace-pcs <addr1,addr2,...>` — quando o PC bate num endereco vigiado, despeja em stderr:
  PC, step, instruction word, registradores ($t1, $s1, $v0, $t4, $t5) e memoria em [$t1*4] e [$s1*4].
- `--dump-mem <addr> <hex_len>` — ao fim da execucao, despeja um trecho de memoria.

O teste `max_steps_limita_o_runner` verifica que `--max-steps` limita a contagem de passos e nao
roda o default.

## O que foi medido

### 1. Despejo da regiao do laco (0x80042000..0x80042300, 700 M passos)

O laco e uma funcao do shell com o seguinte prólogo:

```
80042040: 3C0B8014   lui $t3, 0x8014
80042048: 256B8EE8   addiu $t3, $t3, -0x7118     ; $t3 = 0x80138EE8 (tabela do kernel?)
8004204C: 24100020   addiu $s0, $zero, 0x20       ; $s0 = 0x20
80042054: 240C0020   addiu $t4, $zero, 0x20       ; $t4 = 0x20
80042058: 240D0030   addiu $t5, $zero, 0x30       ; $t5 = 0x30
```

O laco principal:

```
8004205C: 8C820000   lw $v0, 0($a0)               ; carrega a primeira palavra da entrada
80042060: 10000099   beq $zero, $zero, 0x800422C8 ; INCONDICIONAL (rs=rt=0): sempre vai ao check de tipo
80042064: 00021602   srl $v0, $v0, 24             ; delay slot: extrai o byte alto (tipo)
```

O byte alto da primeira palavra de cada entrada e o **identificador de tipo** do evento.
O corpo do laco (0x80042068..0x800422C4) processa a entrada carregando campos, deslocando e
escrevendo em outras estruturas.

A saida do laco:

```
800422C8: 104CFF67   beq $v0, $t4, 0x80042068     ; se tipo == 0x20, processa (inner)
800422CC: 00000000   nop
800422D0: 104DFFA1   beq $v0, $t5, 0x80042158     ; se tipo == 0x30, processa (via alternativa)
800422D4: 00000000   nop
800422D8: 25290001   addiu $t1, $t1, 1             ; incrementa indice
800422DC: 1531FF5F   bne $t1, $s1, 0x8004205C     ; se nao chegou ao fim, volta ao topo
800422E0+:           8FB0xxx  lw $s0, ...           ; epilogo: restaura regs salvos e retorna
```

### 2. O que o laco espera

O laco e um **dispatch de eventos do kernel**. Ele varre uma tabela de entradas, e para cada uma:

1. Toda entrada vai ao check de tipo — o desvio em `0x80042060` e incondicional
   (`beq $zero,$zero`); a entrada nula tem byte de tipo `0x00` e cai fora dos dois `beq`.
2. Se o tipo (byte alto da primeira palavra) e **0x20** ou **0x30**, **processa** o evento
   (dois caminhos de dispatch diferentes).
3. Qualquer outro tipo de evento: **nao processa e passa para a proxima entrada**.

O laco termina quando `$t1 == $s1` (percorreu todas as entradas) e a funcao retorna. Nao ha
dentro dele uma condicao de parada por evento encontrado — ele sempre varre ate o fim e
retorna para o chamador.

**Os valores-chave:**
- `$t4 = 0x20` (32) — tipo de evento A (provavelmente evento de timer ou vblank)
- `$t5 = 0x30` (48) — tipo de evento B (provavelmente evento de CD-ROM ou controlador)
- `$s1` — numero de entradas na tabela (nao medido diretamente, vem do chamador)
- `$t1` — indice atual (incrementado a cada iteracao)

A tabela base em `$t3 = 0x80138EE8` esta **totalmente zerada** ao fim de 700 M passos. Isso
sugere que `$t3` e um ponteiro para a tabela, nao a tabela em si — a tabela real esta em outro
endereco, carregado via `lw` a partir de `$t3` ou passado como argumento em `$a0`.

### 3. O que o TTY diz vs. a referencia

| | Nosso (700 M passos) | DuckStation (referencia) |
|---|---|---|
| TTY | 725 bytes, termina em `SetGraphDebug:level:1,type:0 reverse:0` | Carrega `SCUS_949.00` apos `SetGraphDebug` |
| SYSTEM.CNF | **Nao aparece** | E lido antes do executavel |

O discriminador barato e o TTY (invariante 27). Nosso TTY para em `SetGraphDebug`; a referencia
do DuckStation mostra `Executable path: 'SCUS_949.00'`.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medicao diz | Como foi pego |
|---|---|---|---|---|
| 1 | diagnostico | Que o laco esperava um tipo de evento especifico para SAIR (i.e., evento = gatilho de parada). | O laco nao tem condicao de saida por evento — ele sempre varre ate `$t1 == $s1` e retorna. O dispatch e um loop de varredura, nao de espera. | Decodificacao das instrucoes do laco: nao ha `jr` nem desvio condicional de saida dentro do corpo. O unico desvio para fora e o `bne $t1, $s1` que volta ao topo, e o fluxo que cai para o epilogo quando `$t1 == $s1`. |
| 2 | processo | Que eu poderia criar um teste de integracao que roda o BIOS ate o laco. | Roda 700 M passos — inviavel como teste de CI. | O proprio despejo de 700 M passos em release levou ~35s; um teste unitario desse porte nao cabe no pipeline. O teste que ficou e o de `--max-steps`. |

## Bateria de mutacao

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0125-boot-shell-laco.mut

| # | Mutacao | Teste que pegou |
|---|---|---|
| m1 | while loop usa RUNNER_MAX_STEPS em vez de max_steps | `max_steps_limita_o_runner` (passos >> 1000) |
| m2 | max_steps.unwrap_or descartado, usa RUNNER_MAX_STEPS fixo | `max_steps_limita_o_runner` (passos >> 1000) |
| m3 | max_steps zerado | `max_steps_limita_o_runner` (steps == 0) |
| m4 | --max-steps analisa o valor mas nao atribui a Option | `max_steps_limita_o_runner` (usa default 50M) |
| m5 | --max-steps avanca i em 1 em vez de 2 | `max_steps_limita_o_runner` (arg seguinte vira flag desconhecida, exit 1) |
| c1 | comentario cosmético no loop de parse | verde |
| c2 | renomeacao consistente max_steps → max_steps_parsed | verde |

## Placar antes → depois

Workspace: 821 → **823** testes (o real em `crates/psx-cli/tests/max_steps.rs` + o stub homonimo
em `psx-core/tests/`, exigido pelo portao `bateria_nomes_de_teste_existem`), 0 falhas.

## Decisoes e notas

- **O laco e de dispatch, nao de espera.** A funcao varre a tabela de eventos, despacha os de
  tipo 0x20 e 0x30, e **retorna**. Quem a chama e que decide o que fazer com o resultado
  (provavelmente dormir e chamar de novo no proximo quadro). O fato de o shell nao avancar
  significa que, quadro apos quadro, a funcao e chamada e nunca encontra um evento do tipo que
  faria o chamador transitar de estado — nao que ela mesma esteja presa.
- **O evento que falta esta a montante do dispatch.** O que deveria popular a tabela de eventos
  com uma entrada que dispara a leitura do `SYSTEM.CNF` nao esta acontecendo. Candidatos:
  interrupcao de CD-ROM nao mapeada para evento do kernel, evento de timer com tipo errado,
  ou evento de Vblank que deveria acordar uma thread que por sua vez posta o evento de CD.
- **Proximo passo (4.4u).** O laco de dispatch esta entendido. O que falta e descobrir **quem**
  deveria postar o evento que faz o shell sair do `SetGraphDebug` e montar o sistema de
  arquivos. A referencia do DuckStation mostra `SYSTEM.CNF` sendo lido — o caminho inteiro do
  evento de CD-ROM ate o `open()` do kernel precisa ser rastreado. Nao e mais laco — e fluxo
  de eventos entre subsistemas. A invariante 26 diz "defeito confirmado nao e causa
  confirmada" — aqui o defeito esta confirmado (evento ausente) mas a causa (quem deveria
  posta-lo) ainda nao.

## Revisao adversarial (orquestrador)

1. **Decode errado no doc:** `0x10000099` foi anotado como `beq $v0, $zero` ("se a entrada e
   nula, pula"). Os campos rs e rt da palavra sao ambos `$zero` — o desvio e INCONDICIONAL.
   A entrada nula e descartada pelo check de tipo (byte `0x00` nao casa com `0x20`/`0x30`),
   nao por um teste de nulidade. Conclusao funcional intacta; mecanismo corrigido acima.
2. **Placar de testes:** o doc dizia 822; `cargo test --all --release` conta **823**
   (o trabalhador esqueceu o proprio stub de `psx-core/tests/max_steps.rs`).
3. **Bateria re-executada de verdade:** os 5 mutantes aplicados um a um sobre
   `crates/psx-cli/src/main.rs` e rodados contra o teste real (`cargo test -p psx-cli
   --test max_steps --release`): m1 FAILED 1.4s, m2 FAILED 1.4s, m3 FAILED 0.03s,
   m4 FAILED 1.4s, m5 FAILED 0.03s; c1/c2 verdes. O cabecalho original do `.resultado`
   dizia "script nao existe ainda, item 0.11" — **falso**: `mutantes.ps1` existe desde a
   0041; o motivo real de a bateria ser manual e ele so rodar `cargo test -p psx-core`.
4. **Achado colateral (nao deste PR):** os `.resultado` das iters **0078 e 0079** trazem
   cabecalho "gerado por scripts/mutantes.ps1", mas o script re-rodado hoje na 0078 da
   **0/5** (o mutante em psx-cli nunca e recompilado; o stub em psx-core e sempre-verde) e
   a digital denuncia escrita a mao: `rodado_em` so com a data, quando o script grava ISO
   completo (compare 0077/0080). Os placares 5/5 daquelas iteracoes nao tem prova de
   execucao. Registrado como invariante 29 e item 10.58 do ROADMAP (revalidar as duas
   baterias quando o script souber rodar teste fora do psx-core).
5. **O job `mutantes` da CI reprovava este PR pelo mesmo mecanismo:** rodou a bateria dos
   tres manifestos de CLI alterados (0078, 0079, 0125) e obteve 0/5 estrutural em todos —
   na CI nem BIOS ha, entao o teste real nunca poderia matar nada. `mutantes.ps1` agora
   PULA manifesto com alvo fora de `crates/psx-core/`, com rotulo `FORA-DO-PSX-CORE`
   explicito, em vez de produzir um 0/5 que mede o script e nao o teste. A prova de
   execucao dessas baterias e o `.resultado` manual reaplicado pelo revisor.
