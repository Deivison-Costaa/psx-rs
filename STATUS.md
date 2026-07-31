# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0115** — portas do SIO0 em 16 bits: `write16`/`read16` quebravam a meia-palavra em dois bytes
e o braco do SIO0 ignorava o `offset`, batendo duas vezes no byte baixo. `JOY_CTRL=1003h` do
driver virava `0010h` e soltava o /CS. Corrigido no `bus.rs`; o boot sai do laco do controle e o
TTY passa do driver de pad (ROADMAP 4.4j).

## Próxima tarefa

**ROADMAP 4.4k — `GPU timeout` do kernel depois do driver de pad.**
Medido na 0115 com `psx-estado/instrumentacao/rodajogo.rs` (BIOS + disco, 400 M passos): depois
de `PS-X Control PAD Driver Ver 3.0` o TTY passa a repetir
`GPU timeout:QUE=( 5, 5),CODE=(0,0,00FFFFFF)` e depois `QUE=( 2, 2)`, e o PC circula pelo driver
de GPU do kernel (`0x800511DC`, `0x80051308`, `0x8005131C`) e por `0x00001C28`.
Candidato medido, NAO confirmado: nao existe um so `raise(3)` no repositorio — o IRQ do DMA
nunca chega ao `I_STAT` (mesma forma da invariante 24; `grep -rn "raise(" crates/psx-core/src`
devolve so os bits 0, 2, 7 e timers). **Meça antes de implementar**: instrumente os portos do
DMA2 (`0x1F8010A0..AF`), o `DICR` (`0x1F8010F4`) e o PC do laco, como o `padwait` fez com o SIO0
— registrando o TAMANHO do acesso junto com o endereco (invariante 25).
Spec: `docs/reference/04-dma.md` (§ DMA Interrupt Register, § DMA Channel Control) e
`docs/reference/11-interrupts.md` (§ Interrupt Request / Execution). Arquivos-alvo:
`crates/psx-core/src/dma.rs` e `crates/psx-core/src/bus.rs`.
Armadilha conhecida: `I_STAT` e de borda (invariante 24) e o `DICR` tem bit-31 calculado
(flag mestre) — nao e um bit gravavel comum.
Critério de aceitação: o TTY para de repetir `GPU timeout` e o PC sai do laco do driver de GPU.
Invariantes relevantes: 24, 25.

**Referencia externa (30/07):** captura canonica do DuckStation em
`psx-estado/referencias/tela-de-boot-duckstation.png`; fundo (180,180,180) e cores do losango
CONFIRMADOS iguais aos nossos; sem "®" na tela real. Diferenca visual restante no logo: costuras
de gouraud no losango (candidato 10.14).

## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- **Escopo de commit é UM único identificador `[a-z0-9-]`.** `feat(bus,cpu)` reprova no
  `commit-lint`; quando a mudança toca dois módulos, escolha o principal e cite o outro no
  resumo. Custou uma reescrita de 4 mensagens no PR #36 (ver `0022-scratchpad-isc.md`).
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.

## Placar de testes

Workspace: **766** testes.

## Bloqueios

- **4.4 Boot de jogo**: sem bloqueio conhecido; desde a 0115 o boot passa do handshake do
  controle e para no driver de GPU do kernel (4.4k). Imagens de disco ficam fora do repositório, em
  `.../Programacao com agentes/roms/extraido/`. **Nunca commitar imagem de disco.**
