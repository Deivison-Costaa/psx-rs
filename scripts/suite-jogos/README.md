# Suite de jogos (psx-rs x DuckStation)

Roda jogos comerciais no `psx-cli` com um roteiro de botões fixo, em **segundos emulados**, e
compara o que aparece na tela com o DuckStation (`duckstation-regtest`) rodando o mesmo roteiro.
Cada jogo é APROVADO ou REPROVADO. O código de saída é diferente de 0 se algum jogo reprovar.

## Como rodar

```bash
cargo build --release -p psx-cli
python3 scripts/suite-jogos/suite.py                       # todos os jogos de jogos/*.json
python3 scripts/suite-jogos/suite.py --jogos crash         # só alguns (ids separados por vírgula)
python3 scripts/suite-jogos/suite.py --jogos crash --refs  # (re)gera a referência do DuckStation e roda
python3 scripts/suite-jogos/suite.py --jogos crash --so-refs   # só gera a referência
```

| Opção | Efeito |
|---|---|
| `--jogos a,b` | ids dos jogos (nome do JSON sem `.json`); padrão: todos |
| `--refs` / `--so-refs` | gera as referências do DuckStation antes de rodar / e para |
| `--cli CAMINHO` | binário a testar (padrão `target/release/psx-cli`) |
| `--saida PASTA` | onde vai o relatório (padrão `logs/suite/<data-hora>`, fora do git) |
| `--jobs 1\|2` | jogos em paralelo; no máximo 2 (a máquina é compartilhada) |
| `--sabotagem sem-botoes\|atrasa-botoes` | quebra a execução de propósito (ver "As checagens não são vazias") |

Requisitos: `python3` com `numpy` e `Pillow`; BIOS em `bios/SCPH1001.BIN` (ou `PSX_BIOS`).
Variáveis de ambiente (os padrões são os caminhos desta máquina):

| Variável | Padrão |
|---|---|
| `PSX_ROMS` | `.../Programação com Agentes/roms/dados` (o `cue` do JSON é relativo a ela) |
| `PSX_REFERENCIAS` | `.../Programação com Agentes/tools/oraculos/referencias` |
| `PSX_DS_REGTEST` | `.../tools/oraculos/duckstation/build/bin/duckstation-regtest` (com o patch local de entrada/dump) |
| `PSX_BIOS` | `bios/SCPH1001.BIN` do repositório |

**Frames de jogo têm copyright e nunca vão para o git.** As referências ficam em
`$PSX_REFERENCIAS/<id>/` (`frames/frame_NNNNN.png`, `meta.json`, logs do DuckStation) e a
saída das execuções em `logs/suite/`, que está no `.gitignore`.

## Saída

Em `--saida`:

- `relatorio.md` e `relatorio.json`: por jogo, APROVADO/REPROVADO, a primeira falha e, por
  checkpoint, o score, o tempo do frame de referência escolhido, as resoluções e as falhas.
- `<id>-folha.png`: folha de contato, com o nosso frame à esquerda e o melhor frame do
  DuckStation à direita, em cada checkpoint.
- `<id>/`: `comando.txt` (linha de comando exata do psx-cli), `stderr.log`, `stdout.log`,
  `audio.raw` (s16le estéreo, 44,1 kHz), `card.mcd` e `frames/o-K-fb.png` (frame exibido em
  t = K x 0,5 s; os `.vram` crus são apagados).

## Base de tempo

- 1 segundo emulado = 33.868.800 ciclos de CPU, medidos no contador de ciclos do barramento.
  O `psx-cli` aceita o sufixo `s` em `--press BOTAO@12.5s:0.1s`, `--dump-vram-every 0.5s PREFIXO`,
  `--sample-pcs 0s:90s:0.01s`, `--swap-disc CUE@30s[:3s]`, `--open-lid`/`--close-lid 30s` e
  `--max-time 90s`. As formas antigas, em passos, continuam valendo.
- No DuckStation, t = quadro / 59,94. Os botões viram `REGTEST_INPUT` com
  `quadro = round(t x 59,94)`. A referência grava um PNG a cada 15 quadros (0,25 s).
- Os dois emuladores rodam com boot completo da BIOS (logo da Sony) e, se `memcard: true`, com
  um cartão novo e formatado no slot 1. Com `memcard: false`, os dois rodam sem cartão.

## Formato do JSON (`jogos/<id>.json`)

```json
{
  "id": "crash",
  "nome": "Crash Bandicoot (USA)",
  "cue": "Crash Bandicoot (USA)/Crash Bandicoot (USA).cue",
  "duracao_s": 92.0,
  "memcard": true,
  "botoes": [{"b": "start", "t": 55.0, "dur": 0.1}],
  "extra_cli": [],
  "checkpoints": [
    {"t": 74.0, "marco": "na fase, Crash andando", "checa": ["muda", "parecido_ds", "resolucao"]}
  ],
  "audio": {"min_nao_zero": 0.6}
}
```

- `id` tem de ser igual ao nome do arquivo. `cue` é relativo a `PSX_ROMS`.
- `botoes`: nomes como no `--press` (`select l3 r3 start up right down left l2 r2 l1 r1
  triangle circle cross square`). `t` e `dur` estão em segundos emulados (`dur` padrão 0,1).
- `extra_cli`: argumentos a mais para o psx-cli, por exemplo `["--swap-disc", "<cue>@300s"]`.
  O regtest não troca disco, então a referência só cobre o primeiro disco (a suite avisa).
- `checkpoints[]`: `t`, `marco` (texto livre, aparece no relatório), `checa` (padrão: as três),
  e opcionalmente `limiar` (padrão 0,6) e `janela` (padrão 3 s).
- `audio.min_nao_zero`: fração mínima de blocos de 10 ms não silenciosos na execução inteira.

A referência guarda uma assinatura do roteiro (disco, duração, botões e cartão). Se você mudar
algo disso no JSON, o jogo reprova com "referencia desatualizada" até rodar com `--refs`. Mudar só
os checkpoints não exige regerar.

## O que cada checagem significa

Por checkpoint (tempo `t`):

| Checagem | Passa quando |
|---|---|
| `muda` | Algum par de frames nossos consecutivos em `[t - janela, t]` difere (diferença média de cinza > 1,0 em 160x120) **e** as amostras de PC (`--sample-pcs`, a cada 10 ms) nessa janela não ficam todas dentro de 256 bytes (CPU presa num laço pequeno). |
| `parecido_ds` | O nosso frame em `t` (o último dump até `t`, no máximo 1 s antes), comparado com **o melhor** frame do DuckStation em `[t - janela, t + janela]`, tem score >= `limiar`. Score = média de correlação normalizada e SSIM (blocos 8x8), com as duas imagens em cinza 160x120. |
| `resolucao` | O tamanho exibido do nosso frame é igual ao do frame de referência escolhido. |

Por jogo: o psx-cli sai com código 0, sem `panicked`, sem timeout (8x a duração, no mínimo
600 s) e chega ao fim da duração. A fração de áudio não silencioso é >= `min_nao_zero`, e a
saturação (amostras em +-32767) fica abaixo de 0,1%. A referência existe, está atualizada e
cobre a duração.

Escala do score, medida no Crash: a mesma tela dá 0,98 a 1,00 em telas estáticas e de 0,7 a
0,98 no gameplay. Telas diferentes dão de 0,0 a 0,43. Duas telas pretas dão 1,0. Por isso, se a
janela da referência tem algum frame com conteúdo, os frames quase uniformes dela (fades, telas
pretas) são ignorados na busca do melhor frame: uma tela preta nossa não "acerta" um fade da
referência. Se a janela inteira da referência for uniforme, o relatório avisa ("parecido_ds pouco
informativo"). Não ponha checkpoints em fades ou telas pretas.

## Como adicionar um jogo

1. Reaproveite um roteiro que já funcionou (`logs/onda/comparativo*/scripts.sh` e os
   `relatorio.md` no repositório principal) e escreva `jogos/<id>.json` com os `botoes` e
   `checkpoints: [{"t": 5, "checa": ["resolucao"]}]` provisórios.
2. `python3 scripts/suite-jogos/suite.py --jogos <id> --so-refs` gera os frames em
   `$PSX_REFERENCIAS/<id>/frames/` em segundos. Olhe os frames, ajuste os botões até o
   DuckStation chegar aonde você quer, sempre regerando a referência.
3. Escolha os checkpoints em telas estáveis e características (logo, menu, mapa, gameplay),
   longe de fades e de cortes, e escreva no `marco` o que deve aparecer.
4. Rode a suite e confira a folha de contato. Se o jogo estiver certo e só a sincronia variar
   (gameplay com câmera livre, por exemplo), baixe o `limiar` daquele checkpoint e registre
   o motivo no `marco`.

## As checagens não são vazias

Medido com o Crash em 2026-09-23. O mesmo `crash.json` aprova com o binário normal (scores de
0,70 a 1,00) e reprova nas três sabotagens:

| Execução | Resultado | Checkpoints reprovados (score) |
|---|---|---|
| normal | APROVADO 10/10 | nenhum |
| `--sabotagem sem-botoes` (fica no título) | REPROVADO | 62 s a 91 s (0,005 a 0,17); em 91 s, também "imagem congelada" |
| `--sabotagem atrasa-botoes` (todos os botões 5 s depois) | REPROVADO | 62, 66, 74, 80 e 91 s (0,09 a 0,31); 86 s passa por coincidência de cena (0,81) |
| `--cli` = wrapper que troca o disco pelo Rayman ("binário modificado") | REPROVADO | 38 s a 91 s (0,002 a 0,27), além de resolução 320x224 contra 512x224 |

Os checkpoints da BIOS (10 s e 16 s) passam nas sabotagens, como devem: até ali a execução é a mesma.
