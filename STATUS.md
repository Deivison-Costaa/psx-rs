# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0202 — poligono modulado (fechou 10.13).** `render_triangle` sempre desenhava o texel cru;
GP0(24h) e variantes (bit24=0) exigem `finalChannel=texel*cor/128` (03-gpu.md L1604).
`modulate_texel` nova, com `raw_texture` viajando por `PolygonRender`. 5 testes em
`gpu_poligono_modulacao.rs`; migrados 21 usos de GP0(24h) com cor preta (só testavam
fetch de texel) para GP0(25h) raw em 5 arquivos. Scoreboard local não mudou (rectangles
é RECT, não polígono — achado 10.13 não cobre isso; oráculos de polígono já divergem por
outros defeitos maiores). Evidência real: dump do Crash em 900M passos, **200.749/524.288
px (38%) mudaram** entre antes/depois. Bateria 5/5 + 2/2.
**PR #211 (0201, achado 10.53) ainda aberto sem merge** — STATUS/achados.md/ROADMAP-fechado
vão divergir até lá reconciliar; revisar os dois na hora do merge.
**Exceção de executor vigente: o orquestrador implementa a escada** (`docs/orquestracao.md`).

## Próxima tarefa

Seguir a lista de achados legado `10.x` em `docs/achados.md` (pedido do usuário: "vá passando
por essa lista enorme... resolver esses bugs... faça o máximo possível", uma iteração por
achado, PR sem merge, reportar e continuar). Candidatos bem escopados e ainda não tentados
(citação exata de spec de cada um mora em `docs/achados.md`, não repetida aqui): 10.4
(CAUSE.CE), 10.6 (GP0 80h vram→vram), 10.7 (mask GP0 E6h), 10.8 (SWL/SWR porta destrutiva),
10.14 (gouraud/UV sobre span recortado), 10.45 (load shadow), 10.50 (GP0 C0h sem
transferência), 10.52 (timer lhu/lbu bits 11/12), 10.55/10.56/10.57 (CD-ROM).
**Achado 10.115 — âncoras relativas nos rayman_*** segue pendente (não é bug de jogo, é
manutenção de teste): ~97 literais de passo absoluto nos 10 `rayman_*.rs` → gatilhos por
evento, pré-requisito de 0193.4.

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

Workspace: **1246** testes.
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
