# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0217 — Degrau 9 (ÚLTIMO da escada), branch `iter/0217-dma-cobra-ciclos`, PR não aberto.**
DMA cobra ciclos de verdade: `try_execute_*`/`execute_*` (dma.rs) devolvem as palavras
transferidas; `Bus::charge_dma` vira `Dma::transfer_cost` em `dma_extra_cycles`, drenado no
próximo `tick_timers` (padrão de `Cpu::extra_cycles`). Bateria 5/5+2/2; 7 manifestos antigos
reancorados (`()→usize`) sem regressão. **Rayman (0216) ainda sem disco real pra rerodar.**

## Próxima tarefa

**A escada de 9 degraus do achado 0193.4 está completa.** Falta só medir o resultado:

1. **Remedir os 5 jogos do Degrau 7** (FF7/Tekken3/RE2/Tomb Raider/CTR) contra a escada
   completa 1-9 — Tekken3/RE2/Tomb Raider travavam perto de CD-ROM (0214.3-0214.5), exatamente
   o que o Degrau 9 deveria mexer. CTR não deveria mudar (bug de cadeia software, não timing).
2. **Rerodar os 3 testes do Rayman convertidos na 0216** contra o disco real assim que
   `../roms/extraido/Rayman (USA) DADOS.cue` estiver disponível — a janela (140M-220M) não
   foi confirmada empiricamente ainda.
3. Rodar oráculos `tests/exes/ps1-tests/dma`/`.../spu` contra a escada completa (10.114).

Achado 0193.4 fica parcialmente aberto: GPU ainda desenha em 0 ciclos (tempo de desenho é
declarado desconhecido pela spec, 10.116) — fora do escopo desta escada por decisão do
Degrau 8. Degrau 10 (Load Shadow) segue **não recomendado** (dado insuficiente de hardware).

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

Workspace: **1369** testes.
- **NUNCA rodar `cargo test`/`nextest` nem a bateria de mutação junto com o oráculo**: a
  disputa de CPU faz o `Start-Process` ler stdout antes do flush e reportar `sem-saida`
  falso. Derrubou 16/21 numa medição da 0170; rodada limpa deu 21/21.
- **GTE: 1100/1100 no `gte_valid_0xc0ffee_50.log`** (gitignored, em
  `tests/exes/ps1-tests/gte-fuzz/`). É o oráculo mais barato do projeto: 0,4 s e placar por
  registrador. Sem o arquivo o teste se ignora sozinho.
- **Crash e Rayman animam e soam** (medido na 0192): 8 dumps de VRAM cada, nenhum intervalo
  sem pixel mudando; 3,0 M e 3,4 M quadros de áudio, 94% e 78% de amostras não-zero.
- **Passo absoluto em teste de Rayman reprova por melhoria legítima (10.115).** Os 4 testes
  (`evcb_descritores`, `autoack`, `exception_chain`, `tty_boot`) já usam janela/condição em
  vez de passo fixo (0216). **Disco do Rayman não está em `../roms/extraido/` nesta
  sessão** — os 4 arquivos rodam pelo caminho de skip gracioso, não testados contra dados
  reais aqui; rode numa máquina com a imagem antes de confiar no resultado.
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
