# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0215 — Degrau 8 da escada de timing CPU/barramento (0215.1), branch
`iter/0215-dma-custo-palavra`, PR não aberto.** `Dma::word_cost_per_256`/`transfer_cost`:
tabela pura de custo por canal (`04-dma.md` L217-227) — MDEC.IN/OUT/GPU/OTC=17/16,
CDROM=24/1 (padrão BIOS), SPU=33/8, PIO=20/1. Confere contra a própria nota de DRAM Hyper
Page mode da spec (`transfer_cost(MDEC,16)==17`). Sem chamador ainda. Bateria 6/6+2/2.

## Próxima tarefa

Escada motivada pelo Achado 0193.4 (CPU sem custo de acesso a memória/periférico); 10
degraus, plano completo (números da spec de cada degrau) em
`~/.claude/plans/smooth-swimming-manatee.md`.

**Degrau 9: DMA cobra ciclos de verdade.** Acumular `Dma::transfer_cost` (Degrau 8) e somar
em `Bus::tick_timers` ANTES de drenar o scheduler e ANTES de `Timers::tick` (senão os
timers ficam artificialmente lentos durante DMA). Depende do Degrau 6 (scheduler, pronto —
sem ele um tick de 12288 ciclos de 1 setor de CD-ROM perderia eventos periódicos).
**Maior risco de todos**: testes do Rayman com passo absoluto (`rayman_autoack.rs`/
`rayman_exception_chain.rs`/`rayman_tty_boot.rs`, achado 10.115) vão quebrar; converter pra
condição-primeiro ANTES de tocar em `bus.rs`, não depois. Rodar oráculos `tests/exes/
ps1-tests/dma`/`.../spu` antes/depois (10.114). Depois, Degrau 7 sugere remedir Tekken3/
RE2/Tomb Raider de novo — 3/5 jogos travavam em algo CD-ROM-adjacente (0214.3-0214.5).

PRs #218/#219/#220 seguem abertos. Lista legado `10.x` em segundo plano até a escada avançar.

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

Invariantes relevantes: 17 (espera da BIOS cobre um frame — reconferir a cada degrau da
escada de timing), 34 (acumulador de ciclos extras é estado de pipeline).

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

Workspace: **1344** testes.
- **NUNCA rodar `cargo test`/`nextest` nem a bateria de mutação junto com o oráculo**: a
  disputa de CPU faz o `Start-Process` ler stdout antes do flush e reportar `sem-saida`
  falso. Derrubou 16/21 numa medição da 0170; rodada limpa deu 21/21.
- **GTE: 1100/1100 no `gte_valid_0xc0ffee_50.log`** (gitignored, em
  `tests/exes/ps1-tests/gte-fuzz/`). É o oráculo mais barato do projeto: 0,4 s e placar por
  registrador. Sem o arquivo o teste se ignora sozinho.
- **Crash e Rayman animam e soam** (medido na 0192): 8 dumps de VRAM cada, nenhum intervalo
  sem pixel mudando; 3,0 M e 3,4 M quadros de áudio, 94% e 78% de amostras não-zero.
- **Passo absoluto em teste de Rayman reprova por melhoria legítima (10.115).** `rayman_
  evcb_descritores` já foi convertido pra condição (dispara no 1º instante em que os dois
  descritores ligam, não passo fixo); os 3 que faltam são risco pro Degrau 9 (ver abaixo).
- **`mutantes.ps1` herda o último `teste:` visto (10.71)**: declare `teste:` em TODO
  registro do manifesto, não só no cabeçalho. Custou 9/18 falsos na 0187 e, na 0214, um
  mutante de scheduler rodando contra o alvo errado **travou ~520s de CPU num laço infinito**
  (mate o processo via `Get-Process`/`Stop-Process`, não só re-rode).
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
