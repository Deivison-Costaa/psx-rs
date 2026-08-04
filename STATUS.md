# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0199 — diffvram no scoreboard (fechou 10.23).** `psx-cli --vram-to-png` + comparação
com os `vram.png` de hardware; scoreboard foi de 2 para **20 suítes com veredito**.
`clipping` deu **vram-ok 0px** (pixel-perfeito, valida a conversão `<<3`); `rectangles`
diverge 11.560px (é o 10.11), `lines` 518px; `clut-cache`/`texture-flip`/`mdec` 524.288px
(VRAM inteira). Bateria manual do conversor: 5/5 + 2/2. Antes dela: as duas 0193 em
paralelo (colisão no diário) e o M9 inteiro (0194-0198) fundido com revisão adversarial
(3 defeitos consertados na fusão, achados 0198.1-0198.6). **Exceção de executor vigente:
o orquestrador implementa a escada diretamente** (`docs/orquestracao.md`).

## Próxima tarefa

**Achado 10.11 — retângulos texturizados na GPU (o HUD do Crash).** `gpu.rs` descarta
todo rect com textura (dispatch ~966-1016 vira `vram_state=Idle` sem desenhar;
`render_rect` só pinta cor chapada). Escopo: raw 15bpp + CLUT 4/8bpp com texpage E1,
texture window E2 e mask bits; **modulação fica para o 10.13**. Ler `03-gpu.md` (seção de
retângulos/sprites) ANTES (R1). Teste-antes: `gpu_rect_textured.rs` (textura via GP0(A0),
sprite desenhado, halfwords conferidos). Régua: suíte `rectangles` hoje em **11.560px**
no scoreboard — tem de cair; `clipping` tem de seguir 0px.

Depois, na ordem: Achado 10.115 (âncoras relativas nos rayman_*), 0193.4 (custo de ciclo
de memória; oráculo `cpu/access-time`), 0193.5/0193.3/0189.1 (áudio), 0193.2 (24bpp),
10.13 (modulação). Depois: ROADMAP 11.2, 11.3, remedir Amidog e lote do oráculo TTY.

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

Workspace: **1235** testes.
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
