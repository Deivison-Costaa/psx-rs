# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0221 — escrita de byte nos offsets 1-3 dos registradores de DMA.** O jogo liga/desliga a
máscara do canal no DICR por read-modify-write de 1 byte em `1F8010F6h`; a leitura devolvia
zero fixo e a escrita era descartada. **Destravou Tomb Raider I e III e Silent Hill.**
Estado de cada jogo em `docs/estado-dos-jogos.md` (leia antes de investigar travamento).

## Próxima tarefa

**13 de 15 títulos rodam.** Faltam Tomb Raider II e Final Fantasy IX. Leia
`docs/estado-dos-jogos.md` ANTES de investigar qualquer travamento: ele traz o ponto exato
de congelamento de cada jogo, as hipóteses **já refutadas por medição** e duas armadilhas de
medição que já fizeram relatório mentir.

1. **FMV sai granulada** (defeito exposto ao destravar: antes nenhuma FMV decodificava).
   Logos legíveis sobre fundo ruidoso. Suspeito: MDEC (IDCT/zigzag/quantização). Ataque
   pelos oráculos de hardware em `tests/exes/` antes de olhar pixel.
2. **Custo de DMA conta a lentidão do drive duas vezes.** `Dma::word_cost_per_256` cobra a
   tabela "DMA Transfer Rates" (`04-dma.md` L217-226) como stall da CPU, mas aquela tabela é
   a vazão do DISPOSITIVO. O stall deve ser o da seção "DRAM Hyper Page mode" (~17 ciclos
   por 16 palavras). Para o CD-ROM já modelamos a lentidão do drive na cadência de setor.
3. **Final Fantasy IX**: gira em `0x800A9A6C` esperando o bit1 do byte em `0x80076B14`. Esse
   byte é campo do próprio jogo (não é stat do CD-ROM — já verifiquei), sobrescrito por um
   memcpy em `pc=0x800226CC` cuja origem passou a conter `0x80015509` no passo 408.411.249.
4. **Tomb Raider II**: mudou com o fix do DICR, não confirmado visualmente.

Achado 0193.4 pode ser **fechado**: o custo por instrução da CPU foi medido contra o modelo
da spec no laço do decoder do TR1 e bate com 0,04% de erro (49,019 contra 49,0). A suspeita
de "CPU rápida demais" está refutada por medição.

Flags do runner e como rodar cada jogo: `docs/como-rodar.md` e
`docs/estado-dos-jogos.md`. **Medir travamento: histograma de PC (`--sample-pcs`, passo
PRIMO) decide melhor que hash de VRAM** — hash congelado não separa "travou" de "menu
parado". Ordene os `.vram` NUMERICAMENTE e compile sempre um binário baseline pra A/B.

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

Workspace: **1413** testes.
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
