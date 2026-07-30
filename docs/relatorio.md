# Relatório final (rascunho incremental)

> Atualizado obrigatoriamente no fechamento de cada marco (regra do protocolo — lição do
> gb-rs, onde o relatório ficou para o fim e quase não existiu). Consolidação: item 11.1.

## 1. O que foi construído

Emulador de PlayStation 1 em Rust (psx-rs), três crates (core puro / CLI headless / app
desktop egui). Estado por marco: ver ROADMAP.md.

Em 28/07/2026, com o M1 fechado: o emulador carrega uma BIOS real, faz sideload de executáveis
PS-EXE, executa o R3000A completo (ALU, shifts, loads/stores com delay slot, branches, mult/div,
LWL/LWR/SWL/SWR, COP0 com mecanismo de exceção), tem scratchpad, cache isolation, hook de TTY
com `putchar`/`puts`/`printf`, e GPUSTAT com decodificação GP0/GP1. Roda suítes de teste de
hardware de terceiros e **produz veredito por teste**.

## 2. Como foi construído (o experimento)

Orquestrador (Claude) + trabalhador (opencode/DeepSeek); protocolo de fonte única com TDD e
bateria de mutação; meta-testes guardando o processo na CI. Detalhes e diário:
`docs/orquestracao.md`.

## 3. Métricas

Fonte: `docs/metricas.csv` (uma linha por execução, appendada pelo runner) e branch
`scoreboard-data`. Gráficos: item 11.2.

### 3.1 Custo e volume (fechamento do M1, 28/07/2026)

| | |
|---|---|
| Execuções do trabalhador | **59** |
| Custo total (trabalhador) | **US$ 1,87** |
| Tokens | 1,79 M entrada / 0,67 M saída |
| Iterações documentadas | 41 |
| Testes no workspace | 274 |
| PRs mergeados | 49 |

Custo por modelo:

| Modelo | Execuções | Custo | Média |
|---|---|---|---|
| `deepseek-v4-pro` | 24 | US$ 1,5607 | US$ 0,0650 |
| `deepseek-chat` | 12 | US$ 0,1779 | US$ 0,0148 |
| `deepseek-v4-flash` | 2 | US$ 0,0895 | US$ 0,0447 |
| `deepseek-reasoner` | 1 | US$ 0,0402 | US$ 0,0402 |

O `chat` é 4,4× mais barato por execução que o `v4-pro` e **nunca passou na revisão adversarial
num item de CPU** — as iterações 0009–0017 rodaram nele. Por execução aproveitada, o barato
custou infinito.

### 3.2 Retrabalho

| Resultado | Execuções |
|---|---|
| `ok` | 42 |
| `ok:correcao-pos-revisao` (todas as variantes) | 12 |
| `falha:sem-execucao` | 2 |
| `abortado` (handoff errado / troca de modelo) | 2 |
| `rejeitado:semantica` | 1 |

**Taxa de retrabalho: 20,3%** (12 rodadas de correção em 59 execuções). Nove itens exigiram
correção após revisão: 0018, 0020, 0021 (×2), 0025, 0027 (×2), 0029 (×2), 0031, 0035, 0036.

**30 das 41 iterações têm revisão cruzada preenchida** — a métrica que o gb-rs não tem, porque
lá a revisão adversarial nunca rodou.

### 3.3 Erros de primeira tentativa

**92 erros registrados** nos 41 docs de iteração. Por categoria:

| Categoria | N |
|---|---|
| API-Rust | 19 |
| endereçamento | 10 |
| processo | 7 |
| flags | 6 |
| *nenhum* (a suposição estava certa) | 6 |
| script / ambiente-host | 5 |
| hardware | 3 |
| bit-mapping | 2 |
| cobertura de teste | 4 |
| outros | 30 |

A concentração diz algo que não era óbvio no início: **o trabalhador erra mais em atrito com a
linguagem e em aritmética de endereço do que em entender o hardware.** A leitura de spec, quando
o handoff aponta arquivo e linha, raramente falha; o que falha é traduzi-la para Rust.

### 3.4 Placar de EXEs de teste

`scripts/scoreboard.ps1` sobre 51 arquivos em `tests/exes/` (ps1-tests build-158 + Amidog):

```
4 com veredito (1p/3f), 46 so com saida, 0 sem saida, 1 nao avaliados, de 51 arquivos
```

| Suíte | Status | Detalhe |
|---|---|---|
| `ps1-tests/cpu/cop` | pass | 2p/0f |
| `ps1-tests/cpu/code-in-io` | fail | 1p/2f |
| `ps1-tests/dma/otc-test` | fail | 3p/35f |
| `ps1-tests/gpu/gp0-e1` | fail | 5p/5f |

As 46 suítes em `tty` imprimem o banner da biblioteca do ps1-tests e param à espera de hardware
que ainda não existe (timers, vblank, DMA). O `1 nao avaliados` é um utilitário de host que veio
no zip e não é PS-EXE. Série histórica publicada pela CI na branch `scoreboard-data`.


### 3.5 Custo e volume do período M4→M6 (29–30/07/2026)

| | |
|---|---|
| Execuções do trabalhador | **58** |
| Custo do período | **US$ 12,74** |
| Tokens | 7,58 M entrada / 1,33 M saída |
| Tempo somado das execuções | 24,3 h |
| Custo total do projeto até aqui | **US$ 21,39** em 139 execuções |
| Iterações documentadas | 99 |
| Manifestos de mutação | 48 |
| Itens fechados | 81 de 131 |

| Resultado | Execuções |
|---|---|
| `ok` | 48 |
| `falha:timeout` | 6 |
| `ok:revisor-*` | 3 |
| `falha:exit--1` | 1 |

O período custou **6,8× o M1 inteiro** e produziu M4 quase fechado, M5 pela metade e M6 com
input funcionando. A linha que mais importa é `falha:timeout`: **6 de 58 execuções morreram no
teto de 45 min**, duas delas deixando trabalho pronto numa branch órfã que uma rodada seguinte
refez do zero. Nenhuma das seis foi erro de emulação.

## 4. Comparativo com o gb-rs

Baseline do projeto-piloto: US$ 145,46 / 55 execuções / 871 testes / DeepSeek US$ 0,11 por
iteração vs Claude US$ 5–8.

| | gb-rs | psx-rs (M1 fechado) |
|---|---|---|
| Execuções do trabalhador | 55 | 59 |
| Custo do trabalhador | US$ 145,46 | **US$ 1,87** |
| Testes | 871 | 274 (M1 de 11 marcos) |
| Revisão adversarial | nunca rodou | 30 de 41 iterações |
| Taxa de retrabalho medida | não medida | 20,3% |
| Erros de 1ª tentativa registrados | não registrados | 92 |

Com praticamente o mesmo número de execuções, o custo caiu **~78×**. A diferença não é o modelo
sozinho: no gb-rs o orquestrador escrevia o código; aqui ele revisa. O que o psx-rs gastou a
mais foi *disciplina de registro*, e é justamente o que produziu os números acima.

## 5. Padrões de falha medidos (o achado central até aqui)

Nenhuma das falhas caras deste projeto foi de emulação. As três que se repetiram:

1. **Afirmação sem execução.** Teste que retorna cedo por arquivo ausente e passa verde
   (0027, duas rodadas seguidas); bateria de mutação creditando mutantes a testes que não os
   pegavam (**três ocorrências**: 0027 M6 e C3, 0029 M8). Todas caíram do mesmo jeito: o
   revisor aplicar o mutante e rodar. A contramedida que entrou nos prompts — *"para cada
   afirmação, o comando que a prova, com a saída colada"* — produziu a primeira rodada sem
   nada a desfazer (1.11, terceira tentativa).
2. **Fechar um caminho de falha abre o vizinho.** A 0022 inventou mecanismo de hardware no
   texto do handoff; a regra criada na 0024 ("Spec tem que citar arquivo e seção") fez o texto
   ficar correto e a invenção migrou para as *armadilhas* (0025). Corrigir o nome de um arquivo
   de teste (0027) abriu espaço para errar o diretório dele na rodada seguinte.
3. **O placar mede o que é fácil, não o que importa.** Com o `printf` implementado, o critério
   "TTY não vazio" fez o placar saltar para 50/51 "passando" — sendo que os 50 só imprimiam o
   banner da biblioteca e paravam. E quando o veredito real entrou (1.13), a linha de resumo
   passou a *excluir* da manchete justamente as suítes que melhoraram: caiu de 50/51 para
   46/51 porque quatro suítes ganharam veredito. Nenhum teste pegaria isso — o CSV estava
   certo, a bateria estava certa; errado estava o que um humano lê.

4. **A bateria de mutação pegou o que quatro checks verdes não pegaram.** O conserto do
   `oc-loop.ps1` (item 10.37) tinha teste escrito antes da implementação, passava `cargo test
   --all`, e passaria os quatro checks da CI. A bateria saiu **2 de 5**: o teste procurava
   `check-runs` como substring — e `check-runs-old` contém `check-runs` — e afirmava a presença
   da *palavra* `PENDENTE`, que continua no `jq` mesmo quando a comparação que faz o laço
   esperar é removida. Precisou de dois fortalecimentos até 5/5. É o argumento empírico mais
   forte do projeto a favor do passo 6 ser obrigatório: sem ele, o código entrava verde.

5. **Um portão do projeto reprovou trabalho correto e travou o início de um marco.** O
   `spec_citations.rs` casava título de seção por substring e recusou a citação
   `Coprocessor Opcode/Parameter Encoding (L207)` de `docs/reference/02-cpu.md` exigindo L179, porque `Opcode/Parameter
   Encoding` é prefixo do título citado. A citação estava certa: o índice tem as duas seções,
   L99→179 e L127→**207** em `docs/reference/02-cpu.md`. O defeito já estava registrado como item 10.16 e ninguém havia
   pago. **O custo de um portão não é só o que ele pega — é também o que ele barra por engano,
   e uma rodada perdida por falso positivo custa o mesmo que uma perdida por defeito.**

6. **A ordem dos passos do protocolo impedia pegar uma classe inteira de erro.** O passo 7 roda
   `cargo test --all` e o passo 8 escreve o doc da iteração. Nessa ordem, citação errada **nunca**
   pode ser pega localmente: o documento que a contém só existe depois do único momento em que a
   suíte é executada. Três rodadas do período morreram assim, nenhuma com defeito de emulação.
   O conserto é um comando — rodar o portão de novo depois de escrever o doc.

7. **Ferramenta de diagnóstico rendeu mais que feature.** O item do `printf` da BIOS (1.11c) não
   toca em nenhuma funcionalidade de emulação: só implementa largura e zero-pad. Ele transformou
   858 linhas de `%2d. 0x%08x - 0x%08x` em valores legíveis, e com isso "50 testes falham" virou
   "estes valores, nestes índices, com dois padrões" — recebido[n] = esperado[n−1] em dois
   índices, e valor na metade errada da palavra em três. Num marco inteiro de aritmética com
   saturação, ver o valor recebido *é* o diagnóstico.

8. **Teste sem asserção passa por todos os portões.** O critério de aceitação do item da base de
   tempo era um `#[test]` cujo corpo é um `eprintln!` mandando conferir à mão. Passa sempre.
   Existem meta-testes para tamanho de arquivo, teto de bytes do ROADMAP, citação de spec,
   frescor de métricas, âncora de manifesto e placar da CI — e **nenhum** que reprove um teste
   que não afirma nada. Virou item 10.34.

9. **Silêncio não é defeito.** Foram lidos `0p/0f` de várias suítes como sintoma, quando 45 das
   51 suítes **renderizam na VRAM** em vez de imprimir veredito, e o `input/pad` é interativo por
   design — `while(1)` imprimindo botões, sem PASS/FAIL. A rodada que respondeu isso foi ler o
   **fonte do teste**; o orquestrador havia inferido da ausência de saída. Regra que sai daí:
   antes de tratar suíte silenciosa como defeito, conferir se ela emite veredito por design.

10. **O checkbox é marcado a partir do título do PR, não da entrega.** A remediação automática do
    encadeamento marca o item lendo o título, e o próprio comentário do script avisa disso. No
    período foram cinco ocorrências de item `[x]` com o critério não atingido — incluindo o app
    desktop marcado como pronto sendo uma casca que criava um `Gpu` solto, sem `Cpu` nem `Bus`, e
    imprimia "Display desligado" para sempre.

11. **Erros do orquestrador, medidos e do mesmo tipo.** Três blocos de instrução injetados no
    encadeamento tinham portão que testava um *rótulo* e não a *coisa*: um sem portão nenhum
    (custou uma rodada inteira refazendo item já fechado), um gateado por número de item que
    colidiu com item alheio, e um gateado por um padrão de código genérico que casava linha
    correta. Além disso, uma melhora real (timeouts de VSync de 104 para 8) foi atribuída a uma
    causa falsa até uma segunda medição mostrar que o registrador em questão nunca é escrito.
    **Regra: o portão tem de casar algo que só existe se o defeito existir.**

Corolário para o método: **conferir o handoff antes de despachar virou o passo de maior
retorno.** Três defeitos distintos foram pegos assim, cada um valendo uma rodada inteira:
afirmação de hardware inventada (0032), handoff que não existia e era um ponteiro para outro
documento descrito como "revisado e aprovado" (0034), e esquema de dados que não representava
o caso real (0035).

## 6. Fechamentos de marco

| Marco | Data | Premissa reconferida | Observações |
|---|---|---|---|
| M0 | 2026-07-27 | sim — rumo confirmado; cadência alterada: revisão adversarial passa a ser em lote por marco (custo do orquestrador ≫ custo do trabalhador) | 12 PRs, 19 testes, pipeline validado: 2 iterações DeepSeek (US$ 0,0094 + US$ 0,0145), revisão adversarial achou 5 defeitos reais (fmt fora de ordem, regressão sem-args, commit sem escopo pego pelo commit-lint, commit em inglês, checkbox não marcado); 3 falhas de infra registradas como dado (shim npm ×2, quoting) |
| M1 | 2026-07-28 | sim — e a cadência voltou atrás: revisão **por PR**, não em lote (decisão do usuário em 27/07, após o lote parar em clippy e um handoff fundir 4 itens). Confirmada pelos números: 20,3% de retrabalho pego na revisão | 17 itens (1.1–1.14), 274 testes, 59 execuções, US$ 1,87. O emulador roda EXEs de hardware reais e produz veredito: `pass - testCop0Disabled` / `pass - testCop0Enabled`. Duas iterações consertaram o **ferramental** e não o emulador (0030: lançador quebrava com aspas no prompt e perdia a métrica da própria falha; commit-lint não revalidava título editado). Três achados viraram itens 10.3–10.5 em vez de virar nota perdida |
| M4 | 2026-07-30 | parcial — 4.1 a 4.3c e 4.4a/b/c/e fechados; **4.4d é o bloqueio medido** (I_MASK fica 0x0000 do primeiro ao último passo do boot, logo `CAUSE.IP` nunca acende e o handler de exceção nunca executa) e o 4.4 depende de imagem BIN/CUE que só o usuário fornece | O scheduler foi ligado ao `Bus` na 0080 — **R2 declarado inviolável desde o dia 1 e ligado a nada por 79 iterações**, escondido porque todo teste chamava `enter_vblank()` à mão. A BIOS passou a desenhar: `framebuffer_for_display()` devolve `Some(640x239)` e a VRAM contém "SONY COMPUTER ENTERTAINMENT" legível |
| M6 | 2026-07-30 | parcial — 6.1 (SIO0 + pad digital) e 6.2 (teclado no desktop) fechados, 6.3 aberto | Os 14 bits do pad conferidos um a um contra a spec. A suíte `input/pad` **não** confirma nada: é interativa por design. A correção do pad se fecha por teste de unidade e observação à mão, e isso está dito no registro em vez de ser mascarado por um número |
