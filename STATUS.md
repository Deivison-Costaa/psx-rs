# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0102** — `STATUS.md` vira handoff puro, invariantes saem para `docs/invariantes.md`, portão
novo reprova handoff que aponta para item inexistente (ROADMAP 10.41).

## Próxima tarefa

**ROADMAP 4.4f — o boot morre num `jr $ra` com `$ra = 3`.**
Medido pelo orquestrador em 30/07, BIOS real, 50M passos, commit `b6db863`. A sequência de PCs é
`8004A54C`, `8004A550`, `8004A554`, e daí direto para `00000003`. A instrução em `[0x8004A554]` é
`0x03E00008` (`jr $ra`) e **`$31` vale 3**; a anterior, `[0x8004A550] = 0x8FB40028`
(`lw $s4, 0x28($sp)`), é epílogo restaurando registrador da pilha. Buscar em PC=3 gera
`CAUSE=0x428` (excode 10, RI) com `EPC=0x00000003`; o kernel não resolve, chama
`A0(40h)` = `SystemErrorUnresolvedException` e fica nela para sempre (1 071 429 chamadas entre 35M
e 50M, `SR=0x410` com IEc=0 e IRQ0 pendente que ninguém mais reconhece).
Critério de aceitação: `psx-cli --bios <BIOS>` passa do passo 26 595 832 sem entrar em `A0(40h)`.
Arquivos-alvo: `crates/psx-core/src/cpu.rs`, `crates/psx-core/src/bus.rs`.
Invariantes relevantes: 2, 10, 15.

**Três hipóteses já medidas e descartadas — não repetir:**
1. *Dispatch de eventos do kernel não entrega o callback.* Falso: a trilha vai de `80000080` a
   `00000C80`, `00000CC0..00000E0C`, `00001A00..` e ao kernel em RAM (`8003FFB0..8003FFE8`,
   1 545 instruções). O dispatch funciona.
2. *Base de tempo insuficiente.* Falso: 89 entradas em vblank em 50M passos, 9 por janela de 5M, e
   a fila do scheduler nunca esvazia.
3. *bit10 (IRQ Request) do modo do timer restaurado cedo demais.* Falso: TTY byte a byte idêntico
   ao da `main`, 597 bytes e 8 `VSync: timeout` dos dois lados.

Os `VSync: timeout` (passos 19,7M a 21,4M) são sintoma anterior, não a causa do silêncio final.
**TTY idêntico ao da `main` significa que não consertou nada** — foi assim que os PRs #114 e #115
caíram, ambos alegando corrigir este item.

## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- **Escopo de commit é UM único identificador `[a-z0-9-]`.** `feat(bus,cpu)` reprova no
  `commit-lint`; quando a mudança toca dois módulos, escolha o principal e cite o outro no
  resumo. Custou uma reescrita de 4 mensagens no PR #36 (ver `0022-scratchpad-isc.md`).
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.

## Placar de testes

Workspace: **707** testes.

## Bloqueios

- **4.4 Boot de jogo**: depende de imagem BIN/CUE fornecida pelo usuário. Não inventar,
  não baixar, não marcar. Quando a imagem estiver disponível, desbloquear.
