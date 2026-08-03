# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0184** — **O Rayman roda.** Ele carrega, toca a intro em MDEC, aceita o controle e chega ao
primeiro nivel, com HUD e sprites animados nos dois framebuffers (153 600 pixels redesenhados por
quadro). Quatro defeitos, nenhum deles especifico deste jogo:
**(1)** o MDEC nao decodificava em cor (`run_decode` saia vazio em 15/24bpp);
**(2)** o DMA1 desistia quando encontrava a fifo vazia — o jogo arma o canal 1 ANTES de alimentar
o MDEC pelo canal 0, entao o canal armado tem de retomar quando o MDEC produz;
**(3)** o DMA1 nao costurava os quatro blocos 8x8 no macrobloco 16x16 (quadro saia em faixas);
**(4)** o /ACK do SIO0 chegava no meio do byte em vez de depois do ultimo SCK, e o driver do
kernel o apagava na limpeza de IRQ7 — 483 reinicios do PAD driver viraram 5.
**A reducao de profundidade do MDEC ARREDONDA, nao trunca** (774 de 3072 canais saiam um passo
abaixo do console). Os testes de `mdec/4bit` e `mdec/8bit` eram verdes contra constantes
derivadas da spec que erravam 16 de 64 e 16 de 32 bytes do gabarito de hardware; agora usam o
gabarito. Bateria 12/12 e 2/2; a 0174 foi reexecutada (6/6, 2/2).

## Próxima tarefa

**ROADMAP 4.4ae — Crash Bandicoot ate o primeiro nivel.** O Rayman fechou o 4.4; o Crash e 3D e
vai bater no GTE (5.4b-5.6) e no SPU (M7), que continuam abertos. Imagem em
`../roms/extraido/Crash Bandicoot (USA).cue`.

**Rodar o Rayman sempre com `--pad` e com o `.cue` MULTI-TRILHA** (`Rayman (USA).cue`).
Reproduzir o estado de jogo: `--max-steps 1200000000 --pad --dump-vram <arquivo>`; o dump e VRAM
crua 1024x512x16bpp, nao PNG.

Achados abertos em `docs/achados.md`. Pendencias do lote A: 10.112 (SPU sem estado) e o resto de
`cpu/io-access-bitwidth`. Lotes do oraculo: tarefa-modelo em
`logs/orquestrador/task-lote-oraculo.txt`.

`K/M` no CSV e **K linhas divergentes de M**. `timers` tem jitter real e nunca dara `identico`.
**Antes de medir CD-ROM, monte disco** (10.108).

Invariantes relevantes: nenhuma.

## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- **Escopo de commit é UM único identificador `[a-z0-9-]`.** `feat(bus,cpu)` reprova no
  `commit-lint`; quando a mudança toca dois módulos, escolha o principal e cite o outro no
  resumo. Custou uma reescrita de 4 mensagens no PR #36 (ver `0022-scratchpad-isc.md`).
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.
- **`ROADMAP.md` estava a 3 bytes do teto na 0121.** As linhas ja fechadas do 4.4 foram
  comprimidas (o contexto mora em `docs/iterations/`), sobrando ~470 bytes. Encurtar, nunca apagar.

## Placar de testes

Workspace: **1039** testes.
- **NUNCA rodar `cargo test` nem a bateria de mutação junto com o oráculo**: a disputa de CPU faz
  o `Start-Process` ler stdout antes do flush e reportar `sem-saida` falso. Derrubou 16/21 numa
  medição da 0170; rodada limpa deu 21/21.
- **4.4 Boot de jogo**: o motor 4.4ad agora avanca setores sequencialmente; a fronteira
  seguinte medida no Rayman foi o caminho hook -> incremento. Imagens de disco ficam fora do
  repositorio, em `.../Programacao com agentes/roms/extraido/`.
  **Nunca commitar imagem de disco.**
- **10.79/10.80/10.81 são diagnóstico, não correção**: `CAUSE.ExcCode=00h` em 1029 hooks;
  `0xBFC00448` instala `0x4A1C` antes de `C(00h)`; nos 458 intervalos sem ack o `I_STAT` só tinha
  bit 2 (CDROM) ou 3 (DMA).
- **10.85 (0159)**: o laço final do Rayman é `0x801B9574`, esperando `[0x801CF2CC] >= 2`. A espera
  do memory card NÃO é o bloqueio: termina sozinha em 166.321.383 com `F4000001h,0100h`.
- **Oraculo de hardware disponivel (0164)**: 51 EXEs em `tests/exes/` (gitignored). Amidog CPU
  em `Result: 00000101` (0166; era `00000109`). Depurar o CPU contra ele custa menos que
  inferir de jogo.
- **Rayman: a CPU nao era a causa (0167) e a BIOS tambem nao (0178)**. O jogo tem driver de CD
  proprio, falando direto com `0x1F801800..03`. Toda investigacao de handler de BIOS (10.79-10.87)
  olhava para o lado errado desta parte.
- **Janela util do Rayman: depois do passo 164.000.000** (`Execute !`); o executavel ocupa
  `0x80125000..0x801CF800`.
- **10.88/10.89 fechados como premissa refutada (0162/0163)**: os descritores no momento da
  espera eram de CDROM (não card); o 2o `KERNEL SETUP` e do bootstrap.
- **10.87 fechado sem correção (0161)**: o auto-ack de IRQ0 no handler de Pad/Card é do BIOS, e
  quem religa depois do `ChangeClearPAD(0)` do jogo é o próprio `StartPAD2`. Não procurar defeito aí.
- **Duas correções de SIO0 (0159, 0160) são da spec e NÃO mexeram no boot** — o histograma de PC
  dos últimos 20 M passos é idêntico byte a byte. Não gastar rodada nova no SIO0 esperando boot.
- **10.83 diagnóstico (0158)**: a ativação 0 não visita `0x4A1C`; ele estava fora das cadeias,
  não pulado.
- **Premissa refutada:** o slot `$v1+0x18` não muda entre boots (0147). O defeito não está
  no valor do slot mas no encaixe temporal entre `SysInitMemory` e o enfileiramento dos
  handlers do jogo.
