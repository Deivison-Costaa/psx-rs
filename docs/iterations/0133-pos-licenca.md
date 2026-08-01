<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0133 — pos-licenca

- **Data:** 2026-08-01
- **Item do roadmap:** 4.4ab
- **Objetivo:** Decidir por medição se o hot em `0xBFC04xxx` pós-tela-de-licença é fluxo
  normal ou o próximo bloqueio — e, se bloqueio, nomear o mecanismo.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § HINTSTS (L313); fila de INTs em L333-337: a 2ª INT de resposta NÃO é entregue até a 1ª ser reconhecida | docs/reference/06-cdrom.md |
| psx-spx | § Setloc, Read, Pause (L969) — contexto dos comandos; Init = 0Ah | docs/reference/06-cdrom.md |

## Medições (nenhuma linha de código nova — só ferramentas existentes)

1. **É bloqueio:** janela 200M–400M steps (2000 amostras, passo primo 100003): **99,85%**
   num único laço em `0xBFC04A48–0xBFC04A90` (BIOS ROM) e a VRAM aos 400M é a MESMA tela de
   licença dos 200M. Janela além do horizonte com 200M de margem (invariante 30).
2. **O laço, desassemblado da própria SCPH1001** (offset = PC − 0xBFC00000): escreve
   **0x0A (Init)** numa porta via ponteiro (`BFC04A44: sb $t2=0x0A`), e faz poll do word de
   RAM **0xA00091C4** esperando 1 (completo) ou 2 (erro), com timeout de 0x7530 voltas;
   estourou → re-envia Init. Retry infinito.
3. **O drive RESPONDE:** 1317 IRQs de CD no run de 400M, em pares INT3→INT2 com cadência
   regular (~783k cycles) — as respostas do Init existem.
4. **Mas o INT2 chega 383 cycles depois do INT3** (ex.: steps 808715668 → 808716051) —
   antes de a ISR da BIOS sequer entrar (latência de exceção + dispatch do kernel é maior).
   O `intsts` salta de 3 para 2 SEM ack no meio: a 2ª resposta atropela a 1ª, violando a
   fila da spec (docs/reference/06-cdrom.md L333-337).
5. **A flag 0x91C4 fica em 0** (dump no estado travado): a ISR nunca fecha o protocolo de
   duas fases (nem 1=done nem 2=erro). Consistente com o atropelo: a ISR nunca observa a
   sequência INT3 → ack → INT2.

## Veredito

Bloqueio real, mecanismo único e nomeado: **nossa 2ª resposta de comando é agendada por
tempo, não pelo ack da 1ª** — exatamente a dívida 10.54 do ROADMAP. Correção é o item
4.4ac: enfileirar a 2ª resposta e só entregá-la quando o INT anterior for reconhecido
(HCLRCTL), conforme docs/reference/06-cdrom.md L333-337. As dívidas 10.53/10.54 podem ser
fechadas juntas se a implementação for a fila.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | nenhum | — | — | iteração de medição pura; as quatro medições confirmaram a primeira hipótese sem desvio |

## Bateria de mutação

Bateria de mutação: não se aplica — zero linhas de código nesta iteração; medições feitas
só com ferramentas existentes (--sample-pcs, --dump-mem, instrumentação de IRQ da 0129) e
desassemblador descartável no scratchpad; nada mutável foi introduzido.

## Placar antes → depois

847 → 847 (sem mudança; iteração só de diagnóstico).

## Revisão cruzada (orquestrador)

Sem achados. O risco de "medição sem código" é afirmar sem provar — cada afirmação acima
tem o comando/artefato correspondente (histograma, disasm da ROM real, contagem de IRQs,
dump da flag), e a citação da fila de INTs foi conferida na linha real da spec.

## Decisões e notas

- O gap INT3→INT2 de 383 cycles é o número a derrubar no 4.4ac: com a fila correta, o INT2
  só aparece depois do HCLRCTL do INT3.
- O timeout do laço da BIOS é 30000 leituras (~360k steps por tentativa) — a cadência de
  ~783k cycles por par INT3/INT2 observada bate com ~1 Init por estouro de timeout.
