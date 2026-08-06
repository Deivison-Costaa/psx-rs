# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0208 — acumulador fracionário de timers (0208.1), branch `iter/0208-timer-acumulador-fracionario`,
PR não aberto.** Urgência do usuário: rodei os 14 jogos comerciais (`../roms/extraido/`); só
Crash funciona, os outros 13 travam (11 na tela SCEA, Tomb Raider I/II com tela preta),
confirmado até 1,6 bilhão de passos — não é boot lento. Workflow de 5 agentes (ff7, tekken3,
re2, tomb-raider, ctr) caçou o PC de cada travamento. **Achado e corrigido:** Tekken 3 e RE2
travam no double-read do Timer 1 (idiom PsyQ, `05-timers.md` L79-86) porque `Timers::tick()`
escalava o resto acumulado por `denom` de novo a cada chamada — Timer 0/1 em dotclock/hblank
saltava centenas/milhares de unidades por tick. Bateria 6/6+2/2 (0059 reexecutada 7/7+2/2,
sem regressão). **Não medi ainda se o fix destrava os jogos de verdade.** Causas diferentes
abertas: 0208.2 (FF7), 0208.3 (Tomb Raider), 0208.4 (CTR), 0208.5 (8 jogos não investigados).
PRs #218/#219/#220 de rodadas anteriores seguem abertos, não mesclados.

## Próxima tarefa

**Prioridade: compatibilidade de jogos, não a lista `10.x`.** Abrir o PR da 0208, depois
rodar Tekken 3/RE2 de novo pra confirmar que passam da tela SCEA (bateria verde não prova
isso). Investigar Achado 0208.2 (FF7, RAM 0x80089D9C só escrita pela BIOS), Achado 0208.3
(Tomb Raider, CD-ROM ISO9660 ou ciclos), Achado 0208.4 (CTR, elo IRQ→contador do jogo), depois
os 8 jogos do Achado 0208.5 um a um, mesmo método (`--sample-pcs`/`--watch-mem`/`--dump-mem`).
Lista legado `10.x`
(10.45/10.83/10.85/10.102/10.114/10.116) fica em segundo plano até isso avançar.

Rodar Crash: `--bios bios/SCPH1001.BIN --disc "../roms/extraido/Crash Bandicoot (USA).cue"
--max-steps 1200000000 --pad --press start@330000000 --press cross@700000000`.
**Rayman: sempre `--pad` e o `.cue` MULTI-TRILHA**, 1200000000.
Flags do runner: `--memcard <a.mcd>`, `--dump-audio <a.raw>` (PCM s16le 44100 Hz),
`--dump-vram-every N PREFIXO`, `--disc-info <cue>`.
App desktop: `./target/release/psx-desktop`, configurado por `psx-rs.toml` (ver
`docs/como-rodar.md`).

Achados abertos em `docs/achados.md`. Lotes do oráculo: tarefa-modelo em
`logs/orquestrador/task-lote-oraculo.txt`.

`K/M` no CSV é **K linhas divergentes de M**. `timers` tem jitter real e nunca dará
`identico`. **Antes de medir CD-ROM, monte disco** (10.108).

Invariantes relevantes: nenhuma.

## Repositório

- `main` protegida a partir da iter 0004; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- **Escopo de commit é UM único identificador `[a-z0-9-]`.** `feat(bus,cpu)` reprova no
  `commit-lint`; quando a mudança toca dois módulos, escolha o principal e cite o outro no
  resumo. Custou uma reescrita de 4 mensagens no PR #36 (ver `0022-scratchpad-isc.md`).
- **Use `cargo nextest run --workspace`, não `cargo test`**: 55 s contra vários minutos.
  A CI já usa nextest desde a 0072; a bancada local não estava usando.
- Iterações são cronológicas e nem sempre na ordem dos itens; o vínculo real está no
  título do PR e no doc da iteração.

## Placar de testes

Workspace: **1266** testes.
- **NUNCA rodar `cargo test`/`nextest` nem a bateria de mutação junto com o oráculo**: a
  disputa de CPU faz o `Start-Process` ler stdout antes do flush e reportar `sem-saida`
  falso. Derrubou 16/21 numa medição da 0170; rodada limpa deu 21/21.
- **GTE: 1100/1100 no `gte_valid_0xc0ffee_50.log`** (gitignored, em
  `tests/exes/ps1-tests/gte-fuzz/`). É o oráculo mais barato do projeto: 0,4 s e placar por
  registrador. Sem o arquivo o teste se ignora sozinho.
- **Crash e Rayman animam e soam** (medido na 0192): 8 dumps de VRAM cada, nenhum intervalo
  sem pixel mudando; 3,0 M e 3,4 M quadros de áudio, 94% e 78% de amostras não-zero.
- **Passo absoluto em teste de Rayman reprova por melhoria legítima (10.115).** Nesta rodada
  cinco deles andaram +6.372 porque o SPUSTAT passou a espelhar o SPUCNT e as esperas do
  kernel terminam. O `rayman_evcb_descritores` deixou de fixar passo: dispara no primeiro
  instante em que os dois descritores estão habilitados.
- **`mutantes.ps1` herda o último `teste:` visto (10.71)**: declare `teste:` em TODO
  registro do manifesto, não só no cabeçalho. Custou uma rodada de 9/18 falsos na 0187.
- **Ele maiúsculo seguido de dígito é lido como citação de spec** pelo `spec_citations`
  (é a forma de citar linha). Nomear os ombros do controle assim em doc reprova; escreva em
  minúscula. Custou duas correções: `docs/como-rodar.md` e o doc da 0196.
- **Lógica pura de frontend mora em `crates/psx-core/src/app/`** (biblioteca, saves, perfil
  de controle, config, sessão). Não é capricho: `mutantes.ps1` só roda `-p psx-core`, então
  código testável fora dele não teria bateria.
- Imagens de disco ficam fora do repositório, em `.../Programacao com agentes/roms/extraido/`.
  **Nunca commitar imagem de disco.**
- **Oraculo de hardware disponivel (0164)**: 51 EXEs em `tests/exes/` (gitignored). Amidog
  CPU em `Result: 00000101` (0166; era `00000109`).
- **Janela útil do Rayman: depois do passo 164.000.000** (`Execute !`); o executável ocupa
  `0x80125000..0x801CF800`.
