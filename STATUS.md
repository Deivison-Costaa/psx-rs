# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0113** — altura do display em 480i: `display_height` dobra o range com GPUSTAT.19/22; a janela
do app mostra a cena inteira e o **M2 fechou 100%** e foi para o arquivo (ROADMAP 2.11).

## Próxima tarefa

**ROADMAP 4.4 — Boot de jogo 2D/menu; primeiro degrau: a shell da BIOS.**
Referencia medida no DuckStation com a MESMA BIOS: apos o logo (~15 s) aparece a shell
(MAIN MENU / MEMORY CARD / CD PLAYER, bolas roxas). Nosso boot fica no laco de espera de VSync
(`PC=0x80059DCC`) com a tela do logo pronta e nunca navega para a shell — descobrir o que a BIOS
espera ali (candidatos, por ordem: resposta/IRQ do CD-ROM a GetStat/GetID sem disco; timer;
campo par/impar do GPUSTAT.31/13 que nunca alterna). Medir com os harnesses de
`psx-estado/instrumentacao/` (contador de comandos de CD + histograma de PC como na 0108).
Spec: `docs/reference/04-cdrom.md` (respostas sem disco) so se a medicao apontar para la.
Arquivos-alvo: a decidir pela medicao (`cdrom.rs`, `gpu.rs` ou `timers.rs`).
Critério de aceitação: despejo da VRAM em corrida longa mostra a shell (MAIN MENU), como no
DuckStation.
Invariantes relevantes: 22, 23.

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

Workspace: **750** testes.

## Bloqueios

- **4.4 Boot de jogo**: sem bloqueio conhecido desde a 0111 (o boot chega vivo ao laco de VSync
  pos-logo). Imagens de disco ficam fora do repositório, em
  `.../Programacao com agentes/roms/extraido/`. **Nunca commitar imagem de disco.**
