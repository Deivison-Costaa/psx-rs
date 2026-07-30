# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0107** — o losango cortado do logo NAO e defeito nosso: a BIOS desenha 256 linhas numa area de
desenho de 240 que ela mesma programou (ROADMAP 2.2d, fica aberto por falta de referencia).

## Próxima tarefa

**ROADMAP 4.4 — o boot de jogo.** As imagens BIN/CUE chegaram (30/07) e ficam FORA do repositorio,
em `../roms/extraido/`. **Nunca commitar imagem de disco.**
O boot da BIOS ja funciona de ponta a ponta: 0 `VSync: timeout`, kernel inicializado, logo desenhado
com o texto "SONY" legivel. Falta medir se a BIOS chega a ler o disco e passar o controle ao jogo.
Arquivos-alvo: `crates/psx-core/src/cdrom.rs`, `crates/psx-core/src/bus.rs`.
Critério de aceitação: o TTY ou a VRAM mostram conteudo vindo do disco, nao mais so o logo.
Invariantes relevantes: nenhum.

**Primeiro passo:** rodar `psx-cli --bios <BIOS> --disc <CUE>` por alguns bilhoes de passos e ver
se o TTY muda depois de `ResetCallback`. Se nao mudar, medir se a BIOS chega a emitir comando de
leitura ao CDROM — o M4 esta 12/13 fechado, entao a maquina de estados existe.

**Erro que ja custou DUAS iteracoes seguidas — nao repetir:** nas 0105 e 0106 eu atribui um defeito
visivel a um componente antes de medir se ele participava. Na 0105 era o blit VRAM->VRAM (que a
BIOS nem emite); na 0107 era o rasterizador (que estava certo). MEÇA se o componente participa
antes de acusa-lo.

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
