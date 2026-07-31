# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0116** — `DICR` do DMA: guardava o valor cru, entao nao havia flag de conclusao, nem bit 31
calculado, nem IRQ3. Agora os flags 24-30 sobem sob (mascara do canal AND master), o b31 e
recalculado a cada escrita e a borda 0->1 levanta `I_STAT.3`. O handler de DMA do kernel passou a
rodar (508 escritas no `DICR` contra 3). **O `GPU timeout` NAO parou** — o defeito era real, a
causa e outra (ROADMAP 4.4k; ver invariante 26).

## Próxima tarefa

**ROADMAP 4.4l — `GPUSTAT.26` preso em zero enquanto o kernel espera para enviar comando.**
Medido na 0116, ao fim de 400 M passos com disco: `GPUSTAT = 0x184E260A`, ou seja bit 28 (pronto
para bloco de DMA) = 1, bit 27 = 1 e **bit 26 (Ready to receive Cmd Word) = 0**. O `gpu.rs` abaixa
o bit 26 enquanto um comando GP0 espera parametros e o levanta ao completar (`grep -n "1 << 26"
crates/psx-core/src/gpu.rs`), entao um comando faminto por parametro que nunca chegou explica o
bit preso — e o driver da GPU do kernel espera esse bit antes de enviar, e desiste com
`GPU timeout:QUE=(n,n),CODE=(0,0,00FFFFFF)`.
**Hipotese NAO confirmada. Meça primeiro**: instrumente `gpu.write32(0, ...)` vindo do
linked-list (`dma.rs::execute_linked_list`) e registre o ULTIMO comando que abaixou o bit 26 e
quantos parametros ele ainda esperava. Nao conserte o parser por intuicao (R1).
Spec: `docs/reference/03-gpu.md` (§ GPU Status Register, § GP0 Render Commands) e
`docs/reference/04-dma.md` (§ Linked List DMA). Arquivos-alvo: `crates/psx-core/src/gpu.rs` e
`crates/psx-core/src/dma.rs`.
Armadilha conhecida: o no do linked-list carrega `word_count` no byte alto do header; um no com
contagem errada entrega comando pela metade sem erro visivel.
Critério de aceitação: o TTY para de repetir `GPU timeout` e o PC sai do laco em `0x80051200..`.
Invariantes relevantes: 24, 26.

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

Workspace: **775** testes.

## Bloqueios

- **4.4 Boot de jogo**: sem bloqueio conhecido; desde a 0115 o boot passa do handshake do
  controle e para no driver de GPU do kernel; a 0116 fechou o IRQ3 sem mover o sintoma (4.4l). Imagens de disco ficam fora do repositório, em
  `.../Programacao com agentes/roms/extraido/`. **Nunca commitar imagem de disco.**
