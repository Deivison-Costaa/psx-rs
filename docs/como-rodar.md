# Como rodar um jogo no psx-rs

Guia de uso, não de desenvolvimento. Para o processo de trabalho veja `CLAUDE.md`; para o
estado atual do emulador, `STATUS.md`.

## O que você precisa

1. **Rust estável.** A versão exata está fixada em `rust-toolchain.toml` e o `rustup` a
   instala sozinho na primeira compilação.
2. **Uma BIOS de PS1 extraída do seu próprio console.** Não vem no repositório e não será
   distribuída. O emulador foi medido contra a `SCPH1001.BIN` (América do Norte); coloque-a
   em `bios/`.
3. **A imagem do jogo em BIN/CUE.** Aponte sempre para o **`.cue`**, nunca para o `.bin`:
   é o `.cue` que descreve as trilhas. Jogos com trilhas de áudio (Rayman, por exemplo)
   **precisam do `.cue` multi-trilha** — usar o `.cue` só de dados faz o jogo travar
   esperando música que nunca chega.

Imagens de disco e BIOS ficam **fora** do repositório.

## Compilar

```
cargo build --release
```

Sempre em `--release`. Em `debug` o emulador roda cerca de 20 vezes mais devagar e nenhum
jogo fica jogável.

## Jogar (app desktop)

```
./target/release/psx-desktop
```

Na primeira vez ele abre na biblioteca sem nada configurado. Clique em **Ajustes**, aponte:

- **BIOS** — o arquivo da BIOS do seu console;
- **Pasta de jogos** — onde estão os `.cue`;

e clique em **Gravar**. Isso cria um `psx-rs.toml` na pasta de onde o app foi aberto. Da
próxima vez ele já sobe pronto. Para apontar outro arquivo de configuração use
`--config <arquivo>`; para sobrepor só a BIOS numa execução, `--bios <arquivo>`.

A biblioteca lista os `.cue` da pasta com **serial**, **região** e tempo já jogado. O serial
sai de dentro do disco (`SYSTEM.CNF`), lendo quatro setores por seek — a varredura não lê a
imagem inteira, então uma pasta com dezenas de jogos abre rápido.

Sem placa de som o app **não quebra**: escreve `audio desligado (sem dispositivo de saida)`
e segue com vídeo.

### Teclas

| Tecla | Botão | Tecla | Botão |
|---|---|---|---|
| Setas | direcional | `Enter` | Start |
| `Z` | X (cross) | `Tab` | Select |
| `Space` | O (circle) | `D` / `F` | `l1` / `r1` |
| `A` | quadrado | `E` / `R` | `l2` / `r2` |
| `S` | triângulo | | |

Os ombros aparecem em minúsculas porque é assim que o `--press` do runner os nomeia.

Controle de PlayStation, Xbox ou genérico é detectado sozinho (gilrs) e vale **junto** com o
teclado. Para remapear, tela de **Controles** (`F10`); o perfil grava em `controles.txt`.

### Teclas de função, durante o jogo

| Tecla | O que faz |
|---|---|
| `Esc` | volta para a biblioteca (grava o tempo jogado) |
| `F5` / `F8` | salva / carrega o save state do slot atual |
| `F6` / `F7` | slot anterior / próximo (0 a 9) |
| `F9` | tela do memory card |
| `F10` | tela de controles |
| `F11` | tela de ajustes |
| `F12` | velocidade: 1× → 2× → 4× → 8× → 1× |

**Save states** vão para `saves/<serial>-<slot>.state`, ~3,6 MB cada, e só carregam no jogo
de que saíram — o serial vai gravado no arquivo. Eles **não** substituem o memory card: são
coisas diferentes, e um save state de outra versão do emulador é recusado, não aplicado pela
metade.

**Memory card** é automático, um por jogo, em `cartoes/<serial>.mcd`. O arquivo só é criado
quando o jogo grava de verdade. `F9` mostra o que há dentro dele.

## Rodar um jogo de verdade (runner headless)

O `psx-cli` roda sem janela e grava o resultado em arquivo. É com ele que o projeto mede.

Antes disso, para saber quem é um disco sem bootá-lo:

```
./target/release/psx-cli --disc-info "../roms/extraido/Crash Bandicoot (USA).cue"
```

```
./target/release/psx-cli \
  --bios bios/SCPH1001.BIN \
  --disc "../roms/extraido/Crash Bandicoot (USA).cue" \
  --max-steps 1200000000 \
  --pad --press start@330000000 --press cross@700000000 \
  --memcard crash.mcd \
  --dump-audio crash.raw \
  --dump-vram-every 150000000 crash
```

Isso leva cerca de um minuto e produz `crash.raw` (áudio) e `crash-1.vram` … `crash-8.vram`
(vídeo). O Crash chega ao menu por volta de 330 M passos, à ilha em 600 M e ao nível em
720 M — daí os `--press` acima.

Para o Rayman, troque o disco, mantenha `--pad` e **tire os `--press`**:

```
./target/release/psx-cli --bios bios/SCPH1001.BIN \
  --disc "../roms/extraido/Rayman (USA).cue" --max-steps 1200000000 --pad
```

### Flags que interessam a quem só quer rodar

| Flag | O que faz |
|---|---|
| `--bios <arquivo>` | BIOS do console (obrigatória) |
| `--disc <arquivo.cue>` | imagem do jogo |
| `--exe <arquivo.exe>` | carrega um PS-EXE direto, sem disco |
| `--max-steps <N>` | quantas instruções executar antes de parar |
| `--pad` | liga um controle digital no slot 1 |
| `--press BOTAO@PASSO[:DURACAO]` | aperta um botão num passo dado; pode repetir |
| `--memcard <arquivo.mcd>` | memory card de 128 KiB, criado se faltar (no app é automático) |
| `--disc-info <arquivo.cue>` | imprime serial, região e trilhas, sem bootar |
| `--dump-audio <arquivo.raw>` | grava o som produzido |
| `--dump-vram <arquivo>` | um retrato da VRAM no fim |
| `--dump-vram-every N <prefixo>` | uma linha do tempo de retratos |

Nomes de botão aceitos por `--press`: `select`, `l3`, `r3`, `start`, `up`, `right`, `down`,
`left`, `l2`, `r2`, `l1`, `r1`, `triangle`, `circle`, `cross`, `square`.

O TTY do jogo e do kernel sai na saída padrão; as sondas de diagnóstico saem na saída de
erro. Para ver só o que o jogo imprimiu, redirecione: `2>/dev/null`.

## Ver o resultado

### Áudio

`--dump-audio` grava **PCM cru**, sem cabeçalho: 16 bits com sinal, dois canais
intercalados, 44100 Hz.

```
ffplay -f s16le -ar 44100 -ch_layout stereo crash.raw
ffmpeg -f s16le -ar 44100 -ac 2 -i crash.raw crash.wav   # para converter
```

### Vídeo

Os dumps de VRAM também são crus: 1024 × 512 pixels de 16 bits, formato BGR555 do
PlayStation — **não são PNG**, nenhum visualizador comum abre direto. São feitos para
comparação automática, não para olhar. O jeito rápido de saber se o jogo desenhou é contar
quantos pixels mudaram entre dois retratos:

```
python3 - <<'EOF'
a = open('crash-3.vram','rb').read()
b = open('crash-4.vram','rb').read()
print(sum(1 for i in range(0, len(a), 2) if a[i:i+2] != b[i:i+2]), 'pixels mudaram')
EOF
```

Zero pixels mudados entre todos os retratos significa jogo travado. Qualquer número acima
de alguns milhares significa que ele está desenhando.

## Quando alguma coisa dá errado

| Sintoma | Causa provável |
|---|---|
| Tela preta e nada acontece | `.cue` de dados em jogo com trilhas de áudio; use o multi-trilha |
| O jogo ignora o controle | faltou `--pad` (ou, no desktop, a janela não está em foco) |
| `BIOS invalida` | arquivo truncado ou não é uma BIOS de PS1 |
| Save não persiste | faltou `--memcard`; sem ele o cartão nem é conectado |
| Som picotado sob carga | esperado: o anel de áudio ainda não tem controle de fluxo |
| Som picotado em fast-forward | esperado: em 8× o SPU produz 8× mais do que a placa consome |
| Save state não carrega | é de outro jogo ou de outra versão do formato; a mensagem diz qual |
| Voz de cutscene áspera | esperado: XA reamostrado por vizinho mais próximo |

Os dois últimos estão registrados como achados abertos (`0189.1` e `0188.1`) em
`docs/achados.md` — são limitação conhecida, não regressão.

Se travar, o que ajuda a diagnosticar não é a descrição do sintoma: é o `--dump-audio` mais
os dumps de `--dump-vram-every` da mesma corrida, e a saída de erro completa.
