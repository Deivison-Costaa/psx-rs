<!-- Custo, tokens e duracao ficam em docs/metricas.csv, medidos pelo runner. -->

# 0158 — rayman-exception-chain

- **Data:** 2026-08-02
- **Item do roadmap:** 10.83 (parte B)
- **Objetivo:** rastrear a caminhada da cadeia de excecao que separa a ativacao 0 das seguintes.

## Spec consultada

| Fonte | Secao | Arquivo local |
|---|---|---|
| psx-spx | § B(19h) - HookEntryInt(addr) (L1467-L1482) | docs/reference/13-kernel-bios.md |
| psx-spx | § Priority Chains (L1484-L1502) | docs/reference/13-kernel-bios.md |
| psx-spx | § C(02h) - SysEnqIntRP(priority,struc) (L1504-L1523) | docs/reference/13-kernel-bios.md |
| psx-spx | § C(03h) - SysDeqIntRP(priority,struc) (L1525-L1533) | docs/reference/13-kernel-bios.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | borrow-checker/API-Rust | `let mut active = None` teria o tipo inferido pelo primeiro `Some`. | A spec nao trata inferencia de tipos Rust. | A primeira compilacao da sonda exigiu `Option<Activation>`; a anotacao foi adicionada antes da medicao. |
| 2 | clippy | A sonda poderia manter a constante do vetor mesmo sem usa-la. | A spec nao trata lint Rust; o portao exige `-D warnings`. | O primeiro portao parou em `VECTOR` nao usado; o segundo commit removeu a constante. |
| 3 | hipotese | Os `C(03h)` imediatamente proximos de `HookEntryInt` poderiam ser a remocao do handler de VBlank. | § C(03h) - SysDeqIntRP(priority,struc) (L1525-L1533) em `docs/reference/13-kernel-bios.md` so documenta a remocao defeituosa; nao identifica o corpo de cada estrutura. | A sonda encontrou quatro chamadas de prioridade 0 em `0xA00091D0/0xA00091E0`, nenhuma de prioridade 1; o handler de VBlank nao foi atribuido a essas chamadas. |
| 4 | processo | O portao completo caberia no limite curto usado para o teste isolado. | A spec nao trata tempo de CI. | O limite de 120 s matou a primeira tentativa e o de 300 s encerrou a segunda durante `vsync_timeout_diag`; os resultados anteriores estavam verdes. |

## Bateria de mutação

Bateria de mutação: não se aplica — a iteracao altera apenas um teste de integracao e documentacao; nenhum arquivo em `crates/*/src/` foi modificado, portanto nao ha producao para mutar.

## Placar antes → depois

Workspace: **892 → 893** testes. `rayman_exception_chain.rs` executa a BIOS e o disco reais, deixa a CPU vetorizar quatro vezes e verifica por efeito a ordem dos handlers, o elemento `0x74A8` e as chamadas C proximas ao hook.

## Revisão cruzada (orquestrador)

Reproduzi a rodada com uma sonda independente que **não** infere a cadeia pelas chamadas de
`C(02h)`: ela lê a tabela de entrada em `[0x00000100]` e caminha os quatro `ExCB` conforme
§ Exception Control Blocks ExCB (L2885) de `docs/reference/13-kernel-bios.md`.
`ExCB` ficou em `0xA000E004` nas cinco ativações medidas.

**1. Os PCs e os passos reproduzem exatamente.** Mesma ordem, mesmos passos (`164111528`,
`164112358`, `164153882`, `164157984`) e mesmo `I_STAT` na entrada do hook. O achado central da
rodada — `0x4A1C` não executa na ativação 0 e executa na ativação 3 — está confirmado.

**2. A atribuição "o Rayman chamou `C(02h)`" está refutada; quem enfileira é o BIOS.** O teste
lê só `$a0`/`$a1`. Medindo `$ra` no mesmo ponto:

| Passo | Chamada | Prio | Estrutura | `$ra` |
|---:|---|---:|---|---|
| 164123817 | `C(03h)` | 2 | `0x000074A8` | `0x00004BB8` |
| 164123833 | `C(02h)` | 2 | `0x000074A8` | `0x00004BC8` |
| 164199669 | `C(03h)` | 2 | `0x000074A8` | `0x00004C98` |
| 164199689 | `C(02h)` | 2 | `0x000074A8` | `0x00004CA8` |

Os dois retornos ficam em RAM do kernel (`0x4BC8`, `0x4CA8`), não em RAM de usuário — o código do
Rayman está em `0x801Bxxxx`. E o par desenfileira-e-reenfileira a MESMA estrutura 16 passos
depois, duas vezes. É o comportamento descrito em
§ C(02h) - SysEnqIntRP (L1516) de `docs/reference/13-kernel-bios.md`:
*"The BIOS seems to be occassionally adding/removing the CardSpecificIrq and PadCardIrq (with
priority 1 and 2)"*.
O mesmo vale para os quatro `C(03h)` de prioridade 0 da
tabela acima: `$ra` = `0xBFC048EC`/`0xBFC04900`, ou seja, ROM do BIOS.

**3. O método do teste não distingue "ausente da cadeia" de "pulado por `ReturnFromException`".**
Ele monta o conjunto de nós a partir das chamadas de `C(02h)` que observou, então nós instalados
antes da sonda são invisíveis. Lendo o `ExCB` a ativação 0 tinha seis elementos que o teste nunca
viu:

| Prio | Elemento | first | second |
|---:|---|---|---|
| 0 | `0x6DA8` | `0x1A00` | — |
| 1 | `0x6D88` | `0x18BC` | `0x19C8` |
| 1 | `0x6D78` | `0x1858` | `0x1990` |
| 1 | `0x6D68` | `0x17F4` | `0x1958` |
| 1 | `0x6D58` | `0x1794` | `0x1920` |
| 3 | `0x6D98` | `0x2458` | — |

Com isso a hipótese alternativa cai: a caminhada da ativação 0 **chegou ao fim**, executando o
elemento de prioridade 3 (`0x2458`); ela não foi truncada por `ReturnFromException` na forma
permitida por § Priority Chains (L1498) de `docs/reference/13-kernel-bios.md`.
`0x4A1C` estava mesmo fora das cadeias, e a prioridade 2 estava vazia.

**4. Fato novo que a rodada não viu:** `0x4A1C` não é `first` nem `second` de nenhum elemento —
ele é alcançado de dentro de `0x49BC`, 3.407 passos depois. E o elemento de prioridade 1
`0x6D88` **executou o seu handler `0x19C8` na ativação 0** (só o verificador `0x18BC` roda nas
ativações com `I_STAT=0x08`, sem VBlank). Ou seja: o elemento de prioridade 1 que responde a
VBlank rodou e mesmo assim o bit 0 continuou pendente; quem reconhece VBlank neste BIOS é o
caminho de prioridade 2 (`0x4A4C` → `0x49BC` → `0x4A1C` com `0xFFFFFFFE`, medido em 0157). Essa é
a pergunta seguinte, e ela substitui a formulação original do item.

Sem defeitos de produção: a rodada não toca `crates/*/src/`. Portão reproduzido aqui —
`cargo fmt --check` limpo, `cargo clippy --all-targets -- -D warnings` limpo,
`cargo test --all --no-fail-fast` com 893 testes verdes.

## Decisões e notas

§ Priority Chains (L1484-L1502) em `docs/reference/13-kernel-bios.md` diz que a excecao percorre as quatro cadeias, que a ordem comeca no primeiro elemento da prioridade 0 e que um handler que reconhece a IRQ pode chamar `ReturnFromException`, pulando prioridades menores e o hook. § B(19h) - HookEntryInt(addr) (L1467-L1482) em `docs/reference/13-kernel-bios.md` diz que o hook so roda depois de a ExceptionHandler terminar.

A cadeia observada na ativacao 0 foi:

`0x1A00 -> 0x18BC -> 0x19C8 -> 0x1858 -> 0x17F4 -> 0x1794 -> 0x2458`

Ela ocorreu depois da vetorizacao no passo **164111528** e antes da entrada no hook no passo **164112358**. O bit 0 de `I_STAT` estava pendente na entrada e continuou pendente no hook. O PC `0x00004A1C` nao foi visitado.

Na ativacao 3, a ordem foi:

`0x1A00 -> 0x18BC -> 0x19C8 -> 0x1858 -> 0x17F4 -> 0x1794 -> 0x4A4C -> 0x49BC -> 0x4A1C -> 0x2458`

A vetorizacao ocorreu no passo **164153882**, `0x4A1C` foi visitado no passo **164157757**, e o hook entrou no passo **164157984** com o bit 0 limpo. A medicao e de PCs efetivamente executados; nao atribui nomes sem evidencia ao corpo especifico do BIOS, que a spec nao descreve.

A divergencia estrutural foi o elemento em `0x000074A8`, de prioridade 2, com `next=0`, `first=0x00004A4C` e `second=0x000049BC`. Ele nao estava presente no snapshot da ativacao 0. Uma chamada de `C(02h)` ocorreu no passo **164123833**, entre a ativacao 0 e a ativacao 3, com prioridade 2 e essa estrutura. Depois disso, os dois PCs novos apareceram na caminhada e `0x4A1C` foi visitado antes do handler final `0x2458`. O teste **nao registra o chamador** — a atribuicao dessa chamada ao codigo do jogo nao esta medida aqui; a revisao cruzada mediu `$ra` e a atribui ao proprio BIOS.

§ C(02h) - SysEnqIntRP(priority,struc) (L1504-L1523) em `docs/reference/13-kernel-bios.md` documenta que o novo elemento entra no inicio da cadeia e fornece os quatro words da estrutura. A estrutura medida antes da chamada era `[0, 0x49BC, 0x4A4C, 0]`; o teste permanente confirma esses words e o efeito posterior, sem declarar que a spec documenta a semantica interna de `0x4A4C`, `0x49BC` ou `0x4A1C`.

§ C(03h) - SysDeqIntRP(priority,struc) (L1525-L1533) em `docs/reference/13-kernel-bios.md` documenta que a funcao so remove de forma confiavel o primeiro elemento e pode agir de forma imprevisivel no resto. Perto da instalacao de `HookEntryInt` no passo **164111334**, as chamadas foram:

| Passo | Funcao | Prioridade | Estrutura |
|---:|---|---:|---|
| 164111239 | `C(03h)` | 0 | `0xA00091D0` |
| 164111288 | `C(03h)` | 0 | `0xA00091E0` |
| 164113116 | `C(03h)` | 0 | `0xA00091D0` |
| 164113154 | `C(03h)` | 0 | `0xA00091E0` |

Nao houve chamada de `C(02h)` ou `C(03h)` com prioridade 1 nessa janela. Portanto, a medicao refuta a explicacao de que o jogo tentou remover diretamente o handler de VBlank da cadeia de prioridade 1 perto da instalacao e falhou. Ela encontra, em vez disso, uma insercao posterior de prioridade 2 que muda a caminhada antes da ativacao 3. A spec cobre a estrutura e o defeito generico das funcoes, mas nao o corpo especifico deste BIOS nem por que esses dois handlers acabam levando a `0x4A1C`.

O teste pula com `eprintln!` e `return` se BIOS ou disco faltarem, como exige a convencao de testes dependentes de artefatos locais. Nao houve alteracao em `crates/*/src/`. A linha pendente de metricas de 0157 ja existia em `docs/metricas.csv` pelo par `(ts, iter)` e foi removida de `logs/metrics-pending.csv` sem fabricar a metrica desta rodada.
