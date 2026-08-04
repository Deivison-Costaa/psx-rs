# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0193 (duas, em paralelo — colisão de número registrada no diário)** e **0194-0198**.

- **0193-desktop-disco** (sessão de jogo com o usuário): `.cue` + `insert_disc` no app,
  pacing por tempo real, escala 4:3, slider. Abriu os achados **0193.1-0193.7**: jogo 2×
  rápido (1 ciclo/instrução sem custo de memória), HUD invisível (10.11), mixer sem
  headroom, display 24bpp ausente. **Exceção de executor autorizada pelo usuário: o
  orquestrador implementa a escada de correção diretamente** (`docs/orquestracao.md`).
- **0193-biblioteca a 0198 (M9 inteiro, rodada paralela)** — o app virou aplicativo:
  identidade do disco por ISO 9660 (Crash = SCUS-94900), save state serde/bincode (3,6 MB;
  `serde`/`bincode` na allowlist do `purity.rs`) com F5/F8 + 10 slots, `.mcd` por serial
  (F9), gilrs + perfis (F10), `psx-rs.toml` (F11), fast-forward 1/2/4/8× (F12), recentes e
  tempo de jogo emulado. **Na fusão, o loop do M9 (passos fixos por repaint) foi religado
  ao pacing por tempo real da 0193-desktop-disco.**

## Próxima tarefa

**Achado 10.23 — diffvram no scoreboard.** Conversor VRAM crua→PNG no `psx-cli` +
`scoreboard.ps1` compara com os `vram.png` de hardware via
`tests/exes/ps1-tests/tools/diffvram/`; coluna de veredito no CSV. Teste-antes:
`ci_diffvram.rs` + conversão sintética no psx-cli.

Depois, na ordem (escada da 0193, plano com o usuário): Achado 10.11 (retângulos
texturizados — HUD do Crash), Achado 10.115 (âncoras relativas nos rayman_*), Achado
0193.4 (custo de ciclo de memória; oráculo `cpu/access-time`), Achados 0193.5, 0193.3 e
0189.1 (áudio), Achado 0193.2 (display 24bpp), Achado 10.13 (modulação). Depois da
escada: ROADMAP 11.2 (gráficos), 11.3 (demo), remedir Amidog `psxtest_cpu` e o lote do
oráculo de TTY (parado desde a 0186).

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

Workspace: **1228** testes.
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
