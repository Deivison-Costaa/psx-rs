# 0141 — libsn-chain-walk

- **Data:** 2026-08-01
- **Item do roadmap:** 4.5
- **Objetivo:** confirmar por medição o gatilho do "rollback do init do LIBSN" que a 0137 nomeou —
  e, se possível, refutá-lo.

## Revisão do PR anterior

PR #157 (iter 0140). Revisão adversarial já registrada lá: achado de manifesto vivo arquivado
(5 de 7 registros ainda casavam), corrigido no próprio PR.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Priority Chains (L1484) | docs/reference/13-kernel-bios.md |
| psx-spx | § C(02h) - SysEnqIntRP(priority,struc)  ;bugged, use with care (L1504) | docs/reference/13-kernel-bios.md |
| psx-spx | § C(03h) - SysDeqIntRP(priority,struc)  ;bugged, use with care (L1525) | docs/reference/13-kernel-bios.md |
| psx-spx | § Exception Control Blocks (ExCB) (4 blocks of 8 bytes each) (L2883) | docs/reference/13-kernel-bios.md |
| psx-spx | § Table of Tables (see BIOS Control Blocks for details) (L438) | docs/reference/13-kernel-bios.md |

Estrutura autoritativa do elemento de cadeia (§ SysEnqIntRP):

```
  00h 4  pointer to next element    (0=none)  ;this pointer is inserted by BIOS
  04h 4  pointer to SECOND function (0=none)  ;executed if func1 returns r2<>0
  08h 4  pointer to FIRST  function (0=none)  ;executed first
  0Ch 4  Not used (usually zero)
```

E a cadeia é alcançada por `[0x100]` → base do array de ExCB (4 blocos de 8 bytes), cada bloco com
`00h 4 ptr to first element of exception chain`.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição mostrou | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | Que os três endereços da 0137 (`0x80140004/14/24`) fossem elementos de cadeia distintos, e que dumpá-los ao fim da execução mostraria os handlers do jogo | Ao fim da execução `0x80140000..3F` contém **lixo/dados do jogo** (`740FEED0`, `BDEEED04`, …), nenhum ponteiro plausível. Dump estático não serve para uma janela que já passou | `--dump-mem` no fim da corrida; precisei de sonda dinâmica no ponto de chamada |
| 2 | premissa herdada | Que o handoff da 0137 ("o jogo enfileirou 2 handlers e os REMOVEU — rollback de init falho") descrevesse o mecanismo, cabendo a esta iteração só achar o gatilho | **Falso.** O handler do jogo SOBREVIVE às duas chamadas de remoção. Quem o destrói é outra coisa | Sonda com `head_antes` em cada chamada de cadeia (ver abaixo) |

## Medição

Sonda descartável no gancho de C0 (mesmo padrão da 0137), registrando prioridade, ponteiro da
estrutura, as três palavras da estrutura, `$ra` e **a cabeça da cadeia antes da chamada**.
Revertida antes do commit; `git status` limpo confirmado.

14 chamadas de cadeia na execução inteira (250 M passos, release). As oito que importam:

```
5   ENQ prio=0 struc=80140004 func1=80058484 func2=800586E0 ra=80035C54 | head_antes=00006DA8
6   ENQ prio=0 struc=80140014 func1=80058628 func2=80058700 ra=80035CA4 | head_antes=80140004
7   DEQ prio=0 struc=80140014 next=80140004                 ra=800360DC | head_antes=80140014
8   DEQ prio=0 struc=80140024 next=FFFFFFFE func1=0 func2=0 ra=800360FC | head_antes=80140004
9   ENQ prio=0 struc=A00091D0 (BIOS)                        ra=BFC04890 | head_antes=80140004
10  ENQ prio=0 struc=A00091E0 (BIOS)                        ra=BFC048C0 | head_antes=A00091D0
11  ENQ prio=0 struc=A00091D0 (BIOS)                        ra=BFC04890 | head_antes=00006DA8
12  ENQ prio=0 struc=A00091E0 (BIOS)                        ra=BFC048C0 | head_antes=A00091D0
```

### O que isso estabelece

1. **O jogo instala dois handlers em prioridade 0** — que é a prioridade que a própria spec
   recomenda ("Using priority 0 and 3 should work"). Não há nada de errado no que ele faz.

2. **A chamada 7 é uma remoção correta.** O alvo `80140014` É a cabeça no momento, e a spec diz que
   `SysDeqIntRP` "can remove only the first element". Removeu; a cabeça volta a `80140004`.

3. **A chamada 8 remove um elemento que NUNCA foi enfileirado.** `80140024` tem conteúdo de lixo
   (`next=FFFFFFFE`, funções zeradas) e não está na cadeia. A spec marca a função como
   `;bugged, use with care` e exige "only if you are SURE that the element IS in the chain".
   **Medido: a chamada não remove nada** — a cabeça segue `80140004` na chamada 9.

4. **Portanto o handler do jogo continua instalado depois de todo o suposto "rollback".** A
   conclusão da 0137 está **refutada por medição**: não houve rollback do init pelo jogo.

5. **O que destrói o handler é uma SEGUNDA execução da sequência de boot.** Entre as chamadas 10 e
   11 a cabeça pula de `A00091E0` para `00006DA8`: a cadeia inteira é resetada, levando junto o
   `80140004`. As chamadas 9-10 se repetem literalmente como 11-12. Isso é estado de memória, não
   impressão duplicada. O TTY corrobora: `reading file system` ×2, `Inited and Allocated 20 pages`
   ×2, `ResetGraph:jtb=...` ×2.

### Estado final das quatro cadeias (dump estático, para referência)

`[0x100] = A000E004`, size `0x20` (= 4×8, bate com a spec).

| prio | head | next | func1 | func2 |
|---|---|---|---|---|
| 0 | 0x6DA8 | 0 | 0x1A00 | 0 |
| 1 | 0x6D88 | 0x6D78 | 0x18BC | 0x19C8 |
| 2 | 0x74A8 | 0 | 0x4A4C | 0x49BC |
| 3 | 0x6D98 | 0 | 0x2458 | 0 |

Nenhuma aponta para `0x80140004`: ao fim, o handler do jogo não está em cadeia nenhuma.

## Bateria de mutação

Bateria de mutação: não se aplica — diagnóstico puro, nenhuma linha de código de produção no diff; a sonda foi descartável e revertida antes do commit.

## Placar antes → depois

Workspace: **870** → **870** testes. Nenhum código de produção mudou, por desenho.

## Revisão cruzada (orquestrador)

**Tentativa de revisão cruzada falhou, e a causa é do ferramental.** Como o diff é do próprio
orquestrador, mandei o `opencode-go/gpt-5.6-luna` revisar (foi o que funcionou na 0139). Rodei do
scratchpad para que ele não pudesse escrever no repositório — e isso bloqueou a **leitura**: ele
tentou abrir `docs/reference/13-kernel-bios.md` para conferir as citações de spec e foi barrado por
permissão de diretório externo. Revisor sem acesso de leitura ao repo não consegue verificar
afirmação nenhuma; o isolamento anulou a revisão. Fica como dívida.

**Auto-ataque por medição, no lugar.** A objeção mais forte contra a conclusão desta iteração é:
*"as chamadas 9-10 não se repetem como 11-12; o gancho `pc & 0x1FFFFFFF == 0xC0` é que dispara duas
vezes por chamada"*. Se fosse verdade, a conclusão cairia — e ainda explicaria o item 10.43 sem
boot duplo nenhum. Testei com segunda sonda, carimbando cada disparo com `bus.total_cycles()`:

```
chamadas  9-10 → ciclo 342.532.701 / 342.532.755
chamadas 11-12 → ciclo 354.273.747 / 354.273.801
```

**11.741.046 ciclos de distância** (~25 frames). Os 14 disparos têm 14 carimbos distintos, nenhum
par com ciclo idêntico. Não é gancho disparando duas vezes: é re-execução real. A objeção está
refutada e a conclusão sai mais forte.

A mesma sonda mostrou que os pares do BIOS se repetem ao longo de toda a execução — `DEQ D0/E0` em
14,06 M e de novo em 53,34 M; `ENQ D0/E0` em 342,53 M e de novo em 354,27 M — o que é consistente
com várias reinicializações de kernel, não com uma só.

## Decisões e notas

**1. Por que isto não vira conserto nesta iteração.** A hipótese que o painel da 0137 deixou
armada — laço de espera perdendo corrida por ciclos subcustados em `cpu.rs:187` — **não foi
testada aqui e continua aberta**, mas deixou de ser a explicação do sintoma: o handler do jogo não
some por corrida de timing, some porque a cadeia é resetada por um segundo boot. Implementar
goldens de custo agora seria consertar o que não está provado quebrado (o erro que a R1 e o
histórico da 0104 existem para evitar).

**2. Conexão com o item 10.43, que muda o diagnóstico dele.** O ROADMAP tem
`10.43 Todo texto do TTY sai duplicado (2 linhas 'System ROM' na main e depois do 0103)`,
catalogado como defeito **de TTY**. A evidência desta iteração sugere que não é: se a sequência de
boot roda duas vezes de fato, o TTY duplicado é **sintoma**, não causa. Nada aqui prova o vínculo —
mas é a primeira medição que dá outra leitura para aquele item, e ela é barata de testar.

**3. Próximo passo natural.** Descobrir **o que dispara o segundo boot**: contar entradas em
`0xBFC00000` (reset vector) e em `0x80030000`, e sondar quem escreve na cabeça da cadeia entre as
chamadas 10 e 11. Se o segundo boot for espúrio (nosso), some com ele e o handler do jogo
sobrevive; se for legítimo (o jogo pede reinit), o defeito está em o kernel não repor os handlers.

**4. Método.** Oito hipóteses foram refutadas na 0137 e uma conclusão dela foi refutada aqui. Vale
registrar o padrão: as duas refutações vieram de **instrumentar o ponto de decisão** (o `head_antes`
em cada chamada), não de olhar o estado final. Dump estático descreve o resultado; sonda no ponto
de chamada descreve o mecanismo. É a invariante 27 ("depois de N hipóteses refutadas por
instrumentação, troque de instrumento") aplicada uma camada acima: trocar o instrumento incluiu
trocar o *momento* da medição.
