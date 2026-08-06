# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0207 — GetlocL/GetlocP (10.103), branch `iter/0207-cdrom-getloc`, PR ainda não aberto.**
Os dois comandos caíam no braço genérico de `send_command` (só stat byte); agora devolvem
cabeçalho+sub-header (10h) e trilha/index/posição relativa+absoluta (11h). Falha (80h) com
motor parado, durante play, e — medido no hardware real da 0175, não na spec — antes de
qualquer ReadN/ReadS ter completado. `send_command`/`deliver_first` passaram a receber
`disc_layout`. Extraído `sector_bin_offset` para não duplicar linha ancorada por
0132/0136 (as duas baterias antigas foram reexecutadas, sem regressão). Bateria 7/7+2/2.
**Limitação registrada, não testada:** não há checagem de `seeking` dedicada (só a de "nenhum
setor lido ainda", que cobre o caso comum por coincidência) — ver 0207-cdrom-getloc.md.
Parte de disc-swap do 10.103 (tray não scriptável) virou achado 0207.1, ainda aberto.
**PRs #218 (10.49+10.55+10.56) e #219 (10.52) seguem abertos, não mesclados** — ver seção
anterior do histórico se precisar do contexto de CI/admin-bypass dos merges #211-#217.

## Próxima tarefa

Continuar a lista de achados legado `10.x` em `docs/achados.md` (pedido do usuário: "vá
passando por essa lista... resolver esses bugs... faça o máximo possível", uma iteração por
achado). Candidatos que restam: 10.45 (load shadow — na verdade é custo de ciclo por
instrução/acesso, mesmo cluster arquitetural de baixo que 0193.4/10.102/10.114/10.116;
CUIDADO, R1, precisa de escopo próprio, não é micro-item), 10.83/10.85 (Rayman — investigação
multi-iteração sem causa raiz confirmada ainda, ver 0154/0157/0158/0159; não force fix sem
achar a causa), 10.102/10.114/10.116 (timing arquitetural, grandes, idem 10.45).
Achado 10.115 — âncoras relativas nos rayman_* segue pendente (manutenção de teste, não bug
de jogo).

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

Workspace: **1271** testes.
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
