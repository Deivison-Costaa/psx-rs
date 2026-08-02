# 0150 — vsync-hook

- **Data:** 2026-08-02
- **Item do roadmap:** 10.74
- **Objetivo:** identificar a via usada pelo Rayman para instalar o caminho que deveria incrementar `0x801DF2CC`.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § B(19h) - HookEntryInt(addr) (L1467) | docs/reference/13-kernel-bios.md |
| psx-spx | § B(08h) - OpenEvent(class, spec, mode, func) (L1597) | docs/reference/13-kernel-bios.md |
| psx-spx | § Event Classes (L1656) | docs/reference/13-kernel-bios.md |
| psx-spx | § Priority Chains (L1484) | docs/reference/13-kernel-bios.md |
| psx-spx | § 1F801104h+N*10h - Timer 0..2 Counter Mode (R/W) (L30) | docs/reference/05-timers.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | O índice decimal `19` representaria `B(19h)` | Os nomes das funções BIOS usam índices hexadecimais | A saída registrou `B(13h)`; a sonda foi corrigida para `0x19` |
| 2 | diagnóstico | Qualquer store no endereço `0x80000080` seria substituição pelo jogo | O vetor pode ser inicializado pela BIOS; o PC físico `0x1FCxxxxx` identifica código da BIOS | A primeira asserção classificou stores `BFC019xx` como jogo; o filtro passou a excluir a região da BIOS |
| 3 | causa | A escrita do Timer 1 em `0x801B8BA0` bastaria para provar `SetRCnt` como fonte de VBlank | O bit 0 do modo controla sincronização; o valor escrito foi `0x0548`, com sync desabilitado | A sonda registrou valor, endereço e PC, e a asserção de Timer1-VBlank ficou negativa |

## Bateria de mutação

Bateria de mutação: não se aplica — nenhuma linha de produção foi alterada; esta iteração
acrescenta somente um diagnóstico de integração em `vsync_timeout_diag.rs`.

## Placar antes → depois

Workspace: **883** → **884** testes (+1: `diagnostico_instalacao_vsync_rayman`).

## Diagnóstico

O teste executa a BIOS SCPH1001 com `Rayman (USA) DADOS.cue`, observa as entradas nos wrappers
BIOS B/C, decodifica stores para Timer 1, vetor de exceção e contador, e para ao alcançar o
primeiro PC do spin `0x801B958C`, em vez de esperar a mensagem de timeout.

Resultado principal:

| Evidência | Medição | Veredito |
|---|---|---|
| `B(08h)` com classe `F0000001` | nenhuma chamada antes do spin | `VSyncCallback()` por EvCB não é a via observada |
| Timer 1 | jogo escreve target `0xFFFF` e modo `0x0548` em `0x801B8BA0`/`0x801B8BB4` | sync de VBlank desabilitado; `SetRCnt` não explica o contador |
| vetor `0x80000080` | stores observados pertencem à BIOS/kernel; nenhum store do PC do jogo | não há substituição direta pelo jogo |
| `B(19h) HookEntryInt` | step 164111334, estrutura `0x801D0F78`, PC `0x801B8E60` | via de instalação confirmada |
| execução do hook | `0x801B8E60` reaparece centenas de vezes antes do spin | hook é alcançado |
| código do incremento | `0x801B8C40` lê `0x801DF2CC`; `0x801B8C50` contém `sw ...,0xF2CC` | código existe, mas não é executado antes do spin |
| stores não-zero no contador | zero | causa ainda está depois do hook |

O contador **é** escrito uma vez, no passo 163.969.223, por `0x801ABCF0` — mas com valor zero:
é a inicialização, logo antes de o hook ser instalado (164.111.334). Vale registrar para que
"nenhum store não-zero" não seja lido como "o endereço nunca é tocado".

O resultado corrige a pergunta do handoff: as três alternativas originais não descrevem a via
real. O Rayman instala um hook de exceção por `HookEntryInt`; o próximo diagnóstico deve seguir
o fluxo entre `0x801B8E60` e `0x801B8C40`/`0x801B8C50`, sem implementar hardware por hipótese.

## Revisão cruzada (orquestrador)

**Aprovado. Reproduzi a medição inteira e a via de instalação se confirma.**

```
B(19h) step=19258130   pc=0x00000F20 a0=0x800DFF10  hook[0]=0x8005A1D8   ← do kernel
B(19h) step=164111334  pc=0x00000F20 a0=0x801D0F78  hook[0]=0x801B8E60   ← do jogo
hook entries: 1029
store op=0x2B step=163969223 pc=0x801ABCF0 addr=0x801DF2CC
```

Ponto que merecia ceticismo: existem **dois** `B(19h)`, e o primeiro (passo 19.258.130) instala
contexto do próprio kernel, com `hook[0]=0x8005A1D8`. Confundir os dois invalidaria a conclusão.
O teste desambigua corretamente, exigindo `words[0] == 0x801B_8E60` — asserção por **valor**, não
por presença de chamada.

As três alternativas do handoff anterior — que fui eu quem escreveu — **estavam todas erradas
sobre a via de instalação**. Não é `VSyncCallback()`/`OpenEvent`, não é `SetRCnt`, não é
substituição do vetor. É `HookEntryInt`. Terceira vez nesta sequência que uma premissa minha é
derrubada por medição do trabalhador (0142→0147, ExCB→0149, e agora as três hipóteses→0150), e
isso é o processo funcionando, não falhando.

**Um detalhe que faltava no doc e acrescentei:** o contador não é "nunca tocado" — ele recebe um
store de **zero** em 163.969.223, vindo de `0x801ABCF0`, imediatamente antes da instalação do
hook. A afirmação do teste é sobre stores *não-zero*, e está correta; sem essa nota, um leitor
concluiria que o endereço é inerte.

**Correção de processo (minha, não de luna):** o doc declarava item `10.73b`, que **não existe**
no ROADMAP — o 10.73 foi fechado na 0149. Criei os itens reais: **10.74** (fechado por esta
iteração: a via é `HookEntryInt`) e **10.75** (aberto: o hook roda mas não alcança o incremento).
É a mesma falha de item fantasma que eu cometi na 0148; vale registrar que ela reincide quando
uma rodada começa sem o item já existir na `main`.

**Ressalva de ambiente:** o teste faz `SKIP` sem BIOS e disco, então na CI não afirma nada — a
lição da 0146. Aqui é inerente à medição, e como a iteração é diagnóstico puro sem bateria,
nenhum mutante depende dele.

**Nota sobre a rodada:** executada por `gpt-5.6-luna`, que ficou sem limite antes de commitar. O
trabalho estava inteiro na árvore, sem nenhum commit. Commits, itens de ROADMAP, handoff e esta
revisão são do orquestrador.

## Decisões e notas

- A sonda é descartável e fica junto do diagnóstico anterior para manter um arquivo de teste por item.
- A busca estática localizou 14 instruções com imediato `0xF2CC`; a store relevante é `0x801B8C50` (`0xAC22F2CC`).
- O vínculo com o ROADMAP 4.5 continua não provado; não reutilizar a hipótese da ExCB como causa.
- Próximo handoff: tracear branches/calls de `0x801B8E60` até o bloco `0x801B8C40`, ou provar que o
  incremento depende de uma chamada que o hook deveria fazer e não faz.
