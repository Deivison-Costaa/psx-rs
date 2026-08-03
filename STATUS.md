# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0176** — **lote E do oráculo de TTY: timers+GPU+GTE (R4 dobrado)**. Achada a causa única do
`gte/test-all` (997/999→17/19): `read_data`/`write_data` não tinham os formatos por registrador
da spec (sign-extend VZ/IR, máscara U16 OTZ/SZ, push de SXYP, IRGB/ORGB, LZCR, bug de H) — o
programa aborta os 1100 testes de opcode assim que o 1º teste de registro falha. RTPS ganhou 4
correções (FLAG.22 de IR3 sempre em faixa lm=0, overflow de MAC0/MAC1-3, SAR em vez de divisão
truncada): 71/1150 opcodes passam (era ~1). `timers`: GPU nunca propagava resolução real pros
timers (corrigido; não moveu o placar — HBLANK nunca é agendado de verdade, achado e não
corrigido). `gpu/bandwidth` e `timer-dump` seguem sem correção (10.104/10.105). Bateria 8/8,
controles 2/2.

## Próxima tarefa

**ROADMAP 10.100 e os lotes do oráculo de TTY.** `scripts/oraculo-tty.ps1` é confiável desde a
0171 e o placar em `logs/oraculo-tty.csv` é o alvo: fechar divergência por divergência, por
subsistema. Cinco lotes
— A CPU/kernel, B DMA, C MDEC+SPU, D CD-ROM, E timers+GPU+GTE. Tarefa-modelo pronta em
`logs/orquestrador/task-lote-oraculo.txt` (trocar `<<<LOTE>>>`).

`K/M` no CSV é **K linhas divergentes de M** — já foi lido ao contrário. `timers` tem jitter
real de hardware no gabarito e nunca dará `identico` por comparação exata.

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

Workspace: **969** testes.

## Bloqueios

- **Paridade com hardware real (0171)**: `gpu/gp0-e1`/`gpu/mask-bit` idênticos byte a byte;
  `cpu/cop` sobra `testCop0InvalidOpcode` (10.100).
- **Lote E (0176)**: `gte/test-all` trava no teste 72/1150 (RTPS), defeito novo não diagnosticado
  (SX2/SY2/IR0/SZ3). `timers`: `set_hblank_active` sem chamador real; "System Clock" diverge
  ~13-70x sem depender de GPU, raiz não encontrada. `timer-dump` parece exigir motherboard
  modificada (RTS→TCLK0), pré-requisito que o próprio psx.log documenta.
- **NUNCA rodar `cargo test` nem a bateria de mutação junto com o oráculo**: a disputa de CPU faz
  o `Start-Process` ler stdout antes do flush e reportar `sem-saida` falso. Derrubou 16/21 numa
  medição da 0170; rodada limpa deu 21/21.
- **4.4 Boot de jogo**: o motor 4.4ad agora avanca setores sequencialmente; a fronteira
  seguinte medida no Rayman foi o caminho hook -> incremento. Imagens de disco ficam fora do
  repositorio, em `.../Programacao com agentes/roms/extraido/`.
  **Nunca commitar imagem de disco.**
- **10.79/10.80/10.81 são diagnóstico, não correção**: `CAUSE.ExcCode=00h` em 1029 hooks e
  convergência em `0x801CF2CC`; `0xBFC00448` instala `0x4A1C` antes de `C(00h)`; nos 458
  intervalos sem ack o `I_STAT` só tinha bit 2 (CDROM) ou 3 (DMA) — não há defeito de VBlank aí.
- **10.85 (0159)**: o laço final do Rayman é `0x801B9574`, esperando `[0x801CF2CC] >= 2`. A espera
  do memory card NÃO é o bloqueio: termina sozinha em 166.321.383 com `F4000001h,0100h`.
- **Oraculo de hardware disponivel (0164)**: 51 EXEs em `tests/exes/` (gitignored). Amidog CPU
  em `Result: 00000101` (0166; era `00000109`). Depurar o CPU contra ele custa menos que
  inferir de jogo.
- **Rayman: a CPU nao era a causa (0167, medido)**: com o Amidog em 0 erros o jogo se comporta
  identico ao de antes das tres correcoes. A cadeia de auto-ack de 0158-0163 descreve a parada
  de ~166 M; em 590-600 M o jogo ja esta noutro laco (10.94). Nao retomar Rayman por inferencia:
  a proxima medida util e de tempo, nao de funcao.
- **Janela util do Rayman: depois do passo 164.000.000** (`Execute !`). Antes disso e boot do
  BIOS + BOOTSTRAP LOADER; `0x8003xxxx`/`0x8005xxxx` sao do carregador. O executavel do jogo ocupa
  `0x80125000..0x801CF800`.
- **10.88/10.89 fechados como premissa refutada (0162/0163)**: os descritores no momento da
  espera eram de CDROM (não card); o 2o `KERNEL SETUP` e do bootstrap.
- **10.87 fechado sem correção (0161)**: o auto-ack de IRQ0 no handler de Pad/Card é do BIOS, e
  quem religa depois do `ChangeClearPAD(0)` do jogo é o próprio `StartPAD2`. Não procurar defeito aí.
- **Duas correções de SIO0 (0159, 0160) são da spec e NÃO mexeram no boot** — o histograma de PC
  dos últimos 20 M passos é idêntico byte a byte. Não gastar rodada nova no SIO0 esperando boot.
- **10.83 diagnóstico (0158, já revisado)**: a ativação 0 não visita `0x4A1C`; a posterior visita
  depois do nó `0x74A8` de prioridade 2, inserido pelo BIOS (não pelo jogo). A caminhada da
  ativação 0 chega ao fim (prioridade 3, `0x2458`) — `0x4A1C` estava fora das cadeias, não pulado.
- **Premissa refutada:** o slot `$v1+0x18` não muda entre boots (0147). O defeito não está
  no valor do slot mas no encaixe temporal entre `SysInitMemory` e o enfileiramento dos
  handlers do jogo.
