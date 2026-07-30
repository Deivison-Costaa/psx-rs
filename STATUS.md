# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0108** — diagnostico do segundo crash do boot: `$ra` restaurado da pilha vale 4 (ROADMAP 4.4h).

## Próxima tarefa

**ROADMAP 4.4h — o boot morre de novo, no passo 85 544 264, com `$ra = 4`.**
Medido pelo orquestrador em 30/07, BIOS real + disco, com watchpoint. A cadeia:

1. Passo 85 544 264, `PC=0x8003FA18`, instrucao `8FBF002C` = `lw $ra, 0x2C($sp)` com
   `$sp=0x801FFDA0` — le de **0x801FFDCC** e traz **4**.
2. `jr $ra` em `0x8003FA24` salta para `0x00000004`, que contem o stub
   `addiu $k0,$k1,0xC80 / jr $k0` e leva a `A0(15h)`.
3. Alguem executa `0x8005B6D0`, cujo conteudo e `0x77800000` — opcode primario `0x1D`, **que nao
   existe no MIPS I**. Levantamos RI (excode 10) corretamente; o kernel nao resolve e entra em
   `A0(40h)` = `SystemErrorUnresolvedException` para sempre (1,4 milhao de chamadas).
4. **Watchpoint em 0x801FFDCC: UMA unica escrita em 85 milhoes de passos**, no passo 133 574, de
   `PC=0xBFC018FC` (`sw $v1, 0x1C($sp)`), valor 4. Ou seja o prologo daquela funcao **nunca**
   salvou `$ra` nesse slot: o `$sp` do prologo e diferente do `$sp` do epilogo.

E a mesma familia do 4.4f (`$ra = 3`, tambem de `0x2C($sp)`), mas o conserto de la — interrupcao
no delay slot descartando o salto pendente — nao cobre este caso. Falta achar o segundo mecanismo
que desalinha o `$sp`.
Arquivos-alvo: `crates/psx-core/src/cpu.rs`.
Critério de aceitação: `psx-cli --bios <BIOS> --disc <CUE>` passa do passo 85 544 264 sem entrar em
`A0(40h)`; hoje entra e nunca sai.
Invariantes relevantes: 16.

**Primeiro passo:** watchpoint em `$sp` na janela do prologo — achar onde `$sp` muda sem um
`addiu $sp` correspondente, exatamente como se achou o 4.4f.

**Ja medido, nao repetir:** a BIOS **nao emite nenhum comando de CD-ROM** em 800 M passos (contador
no `send_command`), e a corrida de 3 bilhoes de passos (129 s emulados) nao produz TTY novo depois
de `ResetCallback`. Os dois sao consequencia deste crash, nao causas.

## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- **Escopo de commit é UM único identificador `[a-z0-9-]`.** `feat(bus,cpu)` reprova no
  `commit-lint`; quando a mudança toca dois módulos, escolha o principal e cite o outro no
  resumo. Custou uma reescrita de 4 mensagens no PR #36 (ver `0022-scratchpad-isc.md`).
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.

## Placar de testes

Workspace: **735** testes.

## Bloqueios

- **4.4 Boot de jogo**: DESBLOQUEADO em 30/07 — o usuário forneceu as imagens. Ficam fora do
  repositório, em `C:\psx-roms\` (extraídas dos zips em `.../roms`). **Nunca commitar imagem de
  disco.** Depende agora do 2.2b.
