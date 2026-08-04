# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0193** — primeira sessão de jogo real no app desktop (Crash, pelo usuário): o app ganhou
carga de `.cue` + `insert_disc` (semente do ROADMAP 9.1), pacing por tempo real
(`total_cycles` a 33,8688 MHz), escala 4:3 e slider de velocidade. A sessão diagnosticou
e registrou os achados **0193.1-0193.7**: jogo 2× rápido (1 ciclo/instrução sem custo de
memória), HUD invisível (retângulo texturizado nunca desenhado, 10.11), mixer sem
headroom, filas de áudio com descarte silencioso, display 24bpp ausente.
**Exceção de executor autorizada pelo usuário para a escada 0194+: o orquestrador
implementa diretamente** (registrada em `docs/orquestracao.md`).

## Próxima tarefa

**Achado 10.23 — diffvram no scoreboard.** Conversor VRAM crua→PNG no `psx-cli` +
`scoreboard.ps1` compara com os `vram.png` de hardware via
`tests/exes/ps1-tests/tools/diffvram/`; coluna de veredito no CSV. Teste-antes:
`ci_diffvram.rs` + conversão sintética no psx-cli.

Depois, na ordem (escada da 0193, plano com o usuário): Achado 10.11 (retângulos
texturizados — HUD do Crash), Achado 10.115 (âncoras relativas nos rayman_*), Achado
0193.4 (custo de ciclo de memória; oráculo `cpu/access-time`), Achados 0193.5, 0193.3 e
0189.1 (áudio), Achado 0193.2 (display 24bpp), Achado 10.13 (modulação). **ROADMAP 9.2
(save states) fica para depois da escada.**

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

Workspace: **1137** testes.
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
