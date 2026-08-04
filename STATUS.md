# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0187-0192** — **M4, M5, M6 e M7 fechados numa rodada só** (o usuário autorizou cruzar
itens; a regra de 1 item por PR ficou suspensa para este lote).

- **0187/0188 (7.1, 7.2, 7.4)** — SPU de verdade: 24 vozes com ADPCM, contador de pitch
  com interpolação gaussiana, envoltória ADSR, sweep de volume, mixer estéreo a 44,1 kHz
  pelo scheduler (768 ciclos), reverb completo a 22,05 kHz, ruído por voz (NON), EON e
  entrada de CD-DA/XA-ADPCM. Fechou 10.101 e 10.112.
- **0189 (7.3)** — anel puro no `psx-core` com conversão de taxa por acumulador de fase,
  e stream `cpal` no app desktop. Sem placa de som o app roda com vídeo e avisa na tela.
- **0190 (5.5, 5.6)** — GTE de **889/1100 para 1100/1100** contra o `gte-fuzz` do
  ps1-tests (22 comandos × 50 casos, os 64 registradores comparados). GPF e GPL **não
  existiam**; MVMVA usava o vetor de translação errado; SZ3 saía do MAC3 truncado.
- **0191 (6.3)** — memory card de 128 KiB no endereço 81h, comandos R/W/S, imagem `.mcd`
  crua carregada e regravada só quando o jogo escreve.
- **0192 (4.5)** — fechado **por medição**: o observável ("o Crash carrega 954 KB de WAD e
  não desenha frame") não existe mais. As duas hipóteses que nomearam o item (rollback do
  LIBSN, poll órfão do TMR2) seguem refutadas pelas 0141 e 0147.

## Próxima tarefa

**ROADMAP 9.2 — Snapshot do core (serde) → save states F5/F8 + slots.** É o item que
desbloqueia o resto do M9 (9.3 depende de save/estado por serial). `serde` vai precisar
entrar na allowlist de `purity.rs` no mesmo PR, com justificativa no doc da iteração — o
próprio teste diz isso na mensagem de falha.

Antes, um teste barato: **medir o Amidog `psxtest_cpu`** de novo. Ele parava em
`Result: 00000101` na 0166 e o GTE mudou muito desde então.

Rodar Crash: `--bios bios/SCPH1001.BIN --disc "../roms/extraido/Crash Bandicoot (USA).cue"
--max-steps 1200000000 --pad --press start@330000000 --press cross@700000000`.
**Rayman: sempre `--pad` e o `.cue` MULTI-TRILHA**, 1200000000.
Flags novas do runner: `--memcard <a.mcd>`, `--dump-audio <a.raw>` (PCM s16le 44100 Hz,
ouvir com `ffplay -f s16le -ar 44100 -ch_layout stereo`), `--dump-vram-every N PREFIXO`.

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

Workspace: **1192** testes.
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
- Imagens de disco ficam fora do repositório, em `.../Programacao com agentes/roms/extraido/`.
  **Nunca commitar imagem de disco.**
- **Oraculo de hardware disponivel (0164)**: 51 EXEs em `tests/exes/` (gitignored). Amidog
  CPU em `Result: 00000101` (0166; era `00000109`).
- **Janela útil do Rayman: depois do passo 164.000.000** (`Execute !`); o executável ocupa
  `0x80125000..0x801CF800`.
