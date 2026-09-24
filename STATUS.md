# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0240 — suíte automática de 12 jogos contra o DuckStation** (PR #256): 12/12 aprovados.
Rodar: `python3 scripts/suite-jogos/suite.py --jobs 2` (~15 min; README ao lado). Toda
integração passa por ela antes do merge. Estado de cada jogo: `docs/estado-dos-jogos.md`.

## Próxima tarefa

Os 12 jogos disponíveis rodam, jogam e batem com o DuckStation na imagem; save/load no
cartão confirmado em TR2, TR3 e Rayman. O que falta é timing fino e áudio:

1. **Achado 0241.1** — suíte converte quadro do DuckStation a 59,94 Hz (o PS1 roda ~59,82).
2. **Achado 0240.1** — CD-ROM rápido demais; antes, achar a deriva de ~0,5 s do Crash aos 17-21 s.
3. **Achado 0240.6** — integrar `onda6/dma-custo` sem quebrar `gpu/texture-overflow`.
4. **Achado 10.116** — GPU desenha em 0 ciclos (candidato às diferenças de ±1 quadro).
5. App desktop: teste com controle físico (só coberto por testes unitários) e o 0193.3.
6. **ROADMAP 11.3** — roteiro de demo e relatório final.

Oráculos: DuckStation regtest grava quadros e, com `REGTEST_AUDIO=<wav>`, áudio (guia em
`logs/onda/oraculos/COMO-USAR.md`, fora do git). **Medir travamento: histograma de PC
(`--sample-pcs`) decide melhor que hash de VRAM.** Tempo em segundos emulados (sufixo `s`
nas flags do `psx-cli`), nunca em passos.

Invariantes relevantes: 17 (espera da BIOS cobre um frame), 34 (acumulador de ciclos
extras é estado de pipeline).

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

Workspace: **1763** testes.
- **NUNCA rodar `nextest` nem a bateria de mutação junto com o oráculo**: a disputa de CPU
  faz o `Start-Process` ler stdout antes do flush e reportar `sem-saida` falso (0170).
- **GTE: 1100/1100 no `gte_valid_0xc0ffee_50.log`** (gitignored, em
  `tests/exes/ps1-tests/gte-fuzz/`). É o oráculo mais barato do projeto: 0,4 s e placar por
  registrador. Sem o arquivo o teste se ignora sozinho.
- **Crash e Rayman animam e soam** (medido na 0192): 8 dumps de VRAM cada, nenhum intervalo
  sem pixel mudando; 3,0 M e 3,4 M quadros de áudio, 94% e 78% de amostras não-zero.
- **Passo absoluto em teste reprova por melhoria legítima (10.115)**: use janela/condição.
  Os 4 testes de Rayman já foram convertidos (0216) e rodam por skip gracioso sem o disco.
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
