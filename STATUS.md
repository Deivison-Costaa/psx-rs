# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0180** — **o laço do Rayman quebrou.** `[0x801CEEBC]` foi de `0` para **`1`** e o jogo saiu de
`0x8019FA1C`, onde estava desde a 0167. Três mudanças: (a) `.cue` com um arquivo por trilha —
`parse_cue` guardava um `bin_path` só e sobrava o **último**, então a trilha de dados era lida do
arquivo da trilha 6 e o boot morria em `boot file`; (b) TOC absoluto — o `INDEX 01` de um rip por
trilha é relativo ao próprio arquivo, e sem somar os anteriores a fronteira entre trilhas some;
(c) **AutoPause** — o jogo manda `Setmode=07h` (medido), e o INT4 no fim da trilha é o que arma o
modo que escreve a flag. Bateria 6/6, controles 2/2; âncora da 0064 reparada e reexecutada 7/7.

## Próxima tarefa

**ROADMAP 10.94 (nova parada do Rayman) e 10.90 (VSync).** O jogo agora carrega
`cdrom:SLUS-000.05;1`, imprime `Execute !` e o `PS-X Control PAD Driver Ver 3.0`, e para em
`0x80132BF0`: `while (*(u8*)$s0 == 0) {}`, logo depois que `0x80133F40` retorna. Continuam **296**
`VSync: timeout` — o 10.90 nunca foi tocado e e o proximo obstaculo provavel.

**Rodar sempre com o `.cue` MULTI-TRILHA** (`Rayman (USA).cue`): o `DADOS` nao tem trilha de audio
e o autopause nao dispara nele.

Pendencias do lote A: 10.112 (SPU sem estado) e o resto de `cpu/io-access-bitwidth` (I_MASK ecoa
bruto, SIO/JOY largura, `Dma::write_dicr` deixa passar o bit 6). Lotes do oraculo: tarefa-modelo
em `logs/orquestrador/task-lote-oraculo.txt`.

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

Workspace: **1019** testes.
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
