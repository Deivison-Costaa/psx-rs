# Orquestração — como este projeto é conduzido

> Diário de decisões sobre o PROCESSO (não sobre hardware). Entradas datadas, append-only.
> É insumo direto do relatório final (`docs/relatorio.md`).

## Papéis

- **Trabalhador** — opencode (`opencode-go/gpt-5.6-luna --variant max` desde 2026-08-01;
  `deepseek/deepseek-v4-pro` de 2026-07-27 a 2026-08-01; `deepseek-chat` nas iterações
  0009–0017), disparado por
  `scripts/oc-iter.ps1`. Escreve todo o código de emulação seguindo
  `.claude/skills/iterate/SKILL.md`. Custo por iteração ~duas ordens de grandeza menor que
  um modelo de fronteira (medido no gb-rs: US$ 0,11 vs US$ 5–8).
- **Orquestrador** — Claude (terminal interativo). Não escreve código de emulação. Faz:
  pré-voo do loop, monitoramento, **revisão adversarial de todo PR** (`docs/prompts/review.md`;
  modelo de vendor diferente do autor = revisão cruzada real), merge, métricas, fechamento de
  marco (checagem de premissa com o usuário + atualização do relatório).

## 2026-07-27 — bootstrap e o que herdamos do gb-rs

O projeto anterior (gb-rs, emulador de Game Boy, github.com/Deivison-Costaa/gb-rs) serviu de
experimento-piloto: 98 commits, 871 testes, US$ 145,46 em 55 execuções medidas. Este projeto
nasce corrigindo, por design, falhas que lá foram *medidas* (não opinadas):

1. **Protocolo com fonte única.** No gb-rs, CLAUDE.md (10 passos) e SKILL.md (12 passos)
   divergiam, e a bateria de mutação não estava em nenhum dos dois — só em prosa no STATUS.
   Um experimento controlado (Opus×Sonnet) mostrou o efeito: o agente que não inferia o rigor
   da prosa entregava bateria 0-de-2. Aqui o protocolo mora SÓ no SKILL; CLAUDE.md é índice.
2. **Contexto como recurso.** Medição do gb-rs: ~150k tokens/turno, sendo 41% arquivos de
   teste; cortar o STATUS não resolveu porque o custo é função do tamanho do repositório.
   Daí as regras R7/R8, o teto de 500 linhas por fonte, o mapa.md e os índices de seção nos
   docs de referência.
3. **Métricas sem memória humana.** O metricas.csv do gb-rs congelou duas vezes (26 PRs sem
   série numa delas) porque dependia de alguém rodar um script. Aqui o runner appenda a linha
   ao fim de cada execução e `metrics_freshness.rs` reprova lag > 1 iteração.
4. **Formato de commit blindado.** Lá o formato degradou na troca de agente (era convenção
   verbal); um commit ficou para sempre como `feat(cpu): CALL cc,u16 (    )` por aspas erradas.
   Aqui: template de PR + job `commit-lint` na CI.
5. **Checagem de premissa.** Um mal-entendido de enunciado (GBA vs DMG) sobreviveu 37
   iterações porque todos os controles olhavam para dentro do item. Aqui cada fechamento de
   marco relê o objetivo e pergunta ao usuário.
6. **Relatório incremental.** O marco de apresentação do gb-rs ficou 100% aberto até o fim
   ("o risco não é o emulador ficar incompleto — é ele ficar bom e o relatório não existir").
   Aqui fechar marco exige atualizar `docs/relatorio.md`.

Decisões de bootstrap: repo público `psx-rs`; **merge commit obrigatório** (squash/rebase
desabilitados) para preservar os commits test→feat→docs de cada PR — pedido explícito do
usuário após o histórico squashado do gb-rs; proteção de branch entra junto com a CI (exigir
check inexistente bloquearia os primeiros PRs). Iterações 0003–0005 executam os itens
0.5→0.4→0.3 nessa ordem (dependência: meta-testes exigem docs e CI existentes) — primeira
evidência de que iteração (cronológica) ≠ item (temático).

## 2026-07-27 — o smoke test pagou por si em uma hora

O 0008b falhou duas vezes antes de o modelo sequer ser chamado, e as duas falhas eram
exatamente da familia que o gb-rs documentou como "armadilhas do gh/aspas":

1. Shim npm: `Start-Process opencode` nao executa o .ps1 do npm; e o .cmd degrada aspas —
   um `--version` citado dentro do prompt virou flag do CLI, que imprimiu a versao e saiu
   com codigo 0. "ok" de exit code nao e "ok" de execucao (agora ha guarda: JSON < 1 KB =
   falha:sem-execucao).
2. Quoting do proprio orquestrador: `\"` (habito bash) nao escapa aspas em PowerShell; o
   argumento quebrou e fragmentos do prompt vazaram para outros parametros, corrompendo uma
   linha do metricas.csv (corrigida em 0008d, preservando a falha como dado).

Regra operacional fixada: prompts para oc-iter passam em string single-quoted do PowerShell
(escape de apostrofo = ''), e oc-iter chama o opencode.exe real, sem shims.

## 2026-07-27 — fechamento do M0

M0 completo no dia do bootstrap: 12 PRs mergeados, todos por merge commit com a trinca
test/feat/docs visivel na main, protecao de branch valendo para admin (push direto recusado,
GH006), CI verde em todos.

Numeros do pipeline ate aqui (docs/metricas.csv): 11 execucoes do orquestrador (custo em
assinatura, nao medido em dolar), 3 execucoes do trabalhador DeepSeek - 1 falha de infra
(falha:sem-execucao, 365 ms) e 2 iteracoes completas: 0008b US$ 0,0094 (49 steps, 3 min) e
0009 US$ 0,0145 (62 steps, 4 min). Ordem de grandeza confirmada: ~1 centavo por iteracao
nesta fase (gb-rs pagava US$ 5-8 no modelo de fronteira).

A revisao adversarial (papel novo, inexistente no gb-rs) achou defeito real em 2 de 2 PRs do
trabalhador: fmt fora de ordem derrubando a CI, regressao de comportamento sem-args, commit
sem escopo (pego pelo commit-lint - primeira captura real do guard), mensagem em ingles e
checkbox do ROADMAP nao marcado (virou regra explicita no SKILL). Nenhum achado de logica de
emulacao ainda - o codigo era trivial; o teste de fogo da revisao vem com delay slots (M1).

Correcao de rumo de processo: itens 0.3-0.5 executados fora de ordem por dependencia
(meta-testes exigem docs e CI); iteracao e cronologica, item e tematico.

## 2026-07-27 — decisao do usuario na checagem de premissa: revisao em lote

Premissa do projeto confirmada no fechamento do M0. Mudanca de cadencia decidida pelo
usuario: revisar cada PR do trabalhador custa caro no orquestrador (modelo de fronteira,
assinatura) enquanto o trabalhador custa ~US$ 0,01 - a economia inverteu o gargalo. Novo
regime a partir do M1:

- Trabalhador roda em lote: oc-loop -AutoMerge (merge com checks verdes, sem revisao previa).
- Guards sempre ativos por PR: CI (fmt/clippy/teste), meta-testes de processo, commit-lint,
  bateria de mutacao exigida pelo SKILL.
- Revisao adversarial do orquestrador passa a ser POR MARCO, em lote, sobre o diff acumulado;
  achados viram iteracoes de fix (sufixo letra). O campo "Revisao cruzada" dos docs de
  iteracao e preenchido nessa revisao de lote.
- Metricas: runner grava em logs/metrics-pending.csv (nao rastreado) e o worker incorpora no
  commit docs da iteracao seguinte - mantem arvore limpa entre iteracoes do loop e o lag <= 1
  do metrics_freshness.

Risco aceito e registrado: um erro de logica pode compor por algumas iteracoes ate a revisao
de marco; mitigacao real sao os testes de hardware (Amidog no 1.11) e a bateria de mutacao.

## 2026-07-27 — regra de tamanho estava invertida (iter 0015b)

O `file_size.rs`, escrito por mim no M0, reprovava **arquivo fonte** com mais de 500 linhas
e ignorava `tests/`. Ao ler o handoff da 0015 — que mandava fatiar o `cpu.rs` "porque o teto
manda" — o usuário corrigiu a premissa: a regra dele sempre foi a de **comentários** (≤5%,
reprova em 10%); o tamanho de arquivo fonte, "se fizer sentido, não tem problema", e o teto
"pros testes é uma boa, pros arquivos reais, não".

A correção não é cosmética, é o oposto do que estava valendo, e o motivo original ficava
melhor servido pela regra dele: no gb-rs quem comia contexto eram os testes (41% por turno),
não o `mcycle.rs`. O teto agora varre `crates/*/tests/`, e `src/` não tem teto — cortar
módulo por contagem de linha produz fronteira artificial, que é pior de ler que um arquivo
longo e coeso.

Efeito imediato: `cpu_branch_delay.rs` estava em 494 linhas, seis do limite. Virou
`cpu_jumps.rs` (113) e `cpu_branches.rs` (367), com os encoders de opcode extraídos para
`tests/support/asm.rs` — que é também o que a R8 pede, já que um item futuro sobre desvios
passa a pagar 113 ou 367 linhas de contexto, não 494.

Bateria de mutação do meta-teste: arquivo de teste plantado com 600 linhas → reprovado;
controles: fonte plantada com 900 linhas → verde (comportamento novo desejado), e
`comment_density.rs` intacto. 1/1 pego, 2/2 controles.

Registro honesto de onde o erro nasceu: o plano do M0 derivou "teto de 500 linhas por
arquivo fonte" do diagnóstico do gb-rs sem confirmar com o usuário, e a regra sobreviveu
15 iterações sem ser questionada porque nenhum arquivo tinha chegado perto do limite. Só
apareceu quando o `cpu.rs` chegou a 440 e o handoff começou a exigir refactor por causa
dela.

## 2026-07-27 — a CI reprovou por um lint que a máquina de desenvolvimento não conhece (iter 0016)

Primeira falha de CI da série que não é do emulador nem do processo, e sim do **ambiente**.
O `divu` da 0016 checava `rt_val == 0` antes de dividir; o clippy da CI reprovou com
`manual_checked_ops`. O trabalhador tinha rodado `cargo clippy -D warnings` local e visto
verde, e a remediação automática do `oc-loop` (`fmt` + `clippy --fix`) não achou nada para
corrigir — os dois porque o clippy local **não tem o lint**: stable local em 1.92.0
(2025-12-08), CI em `dtolnay/rust-toolchain@stable`, que instala a última (o log aponta a
doc do clippy 1.97.0).

O que isso diz sobre o protocolo: o passo 7 ("fmt + clippy + test antes de abrir o PR") é um
portão que só vale se o toolchain local for o mesmo da CI. Enquanto os dois lados
perseguirem "stable" de forma independente, toda stable nova é uma chance de PR vermelho por
motivo que o trabalhador não tinha como ver — e as três remediações mecânicas do loop não
cobrem isso, porque `clippy --fix` só conserta o que o clippy local enxerga.

Duas saídas, decisão adiada para uma iteração de infra:
1. `rustup update stable` antes de cada iteração (barato, mas depende de lembrar, e é
   justamente o tipo de dependência de memória humana que o gb-rs mostrou não funcionar);
2. `rust-toolchain.toml` **pinado**, local e CI na mesma versão fixa, bump deliberado como
   item do ROADMAP. Mais determinístico e mais alinhado à tese do projeto; o custo é que
   lints e correções novas do compilador só chegam quando alguém subir a versão.

Por ora: toolchain local atualizado, `divu` reescrito com `checked_div`/`checked_rem` (mesmo
comportamento, tabela de erros intacta) e o STATUS mandando sincronizar antes do clippy.

A revisão da mesma PR achou o defeito mais caro, e esse é de teste, não de código: **os 18
testes não distinguiam DIVU de DIV**. Trocando o corpo do `divu` por divisão com sinal, a
suíte inteira passava — os dois testes de DIVU usavam `rs=100`, valor onde as duas
interpretações coincidem. O contraste é útil: o `multu` **tem** teste com bit alto
(`rs=0x8000_0000`). O trabalhador lembrou do sinal na multiplicação e esqueceu na divisão,
e a bateria de mutação dele (7 mutantes) atacou divisão por zero, overflow e parte alta —
nunca a assinatura. É o terceiro caso da série em que o defeito escapa exatamente onde o
autor do teste não pensou em errar; a revisão adversarial por um modelo de outro vendor
continua sendo o que pega isso.

## 2026-07-27 — a primeira iteração rejeitada, e ela veio verde (iter 0017)

A PR #27 chegou à revisão com CI verde, 20 testes passando e bateria de mutação 7/7. Os
quatro opcodes estavam errados. O diagnóstico está em
`docs/iterations/0017-cpu-unaligned-load-store.md`; aqui fica o que ele diz sobre o
**processo**.

O defeito não é de disciplina: a R5 foi cumprida à risca. Teste antes de implementar protege
o código contra divergir do que o autor entendeu — e não protege contra o autor ter
entendido errado. Quando o mesmo agente escreve o teste e a implementação na mesma sessão, o
teste herda o modelo mental, e a bateria de mutação só confirma que o código concorda
consigo mesmo. Placar 7/7 com valor zero: os mutantes foram conferidos contra expectativas
erradas.

É a diferença entre os dois eixos de verificação que o projeto usa. O eixo *interno*
(teste-primeiro, mutação, clippy, CI) mede consistência e já estava saturado. O eixo
*externo* — revisão adversarial por um modelo de outro vendor, lendo a spec de novo — é o
único que estava olhando para fora do modelo mental do autor, e foi o que pegou. Vale para o
relatório: das quatro últimas iterações, três tiveram defeito real achado só na revisão
cruzada (0014 no bus, 0015 no overflow do link, 0016 na cobertura do DIVU) e a quarta foi
reprovada por inteiro. Nenhum deles apareceu na CI.

Mudança de protocolo, e é a primeira que sai de uma rejeição: quando a spec do item traz um
**idioma canônico**, o handoff passa a incluir um **teste de aceitação com valores literais**
derivado da spec pelo orquestrador, obrigatório no PR. A assimetria é o ponto — uma asserção
com bytes concretos (`r2 = 0x44AABBCC` depois do par lwl/lwr) não pode ser satisfeita por um
modelo errado, enquanto uma asserção que o próprio autor deriva sempre pode.

(Este parágrafo dizia `0x44DDCCBB` quando foi escrito. O valor estava errado — meu, não do
trabalhador. Ver a entrada de 2026-07-27 mais abaixo, "o teste de aceitação obrigatório
estava errado".)

Registro honesto do que **eu** errei: o handoff do 1.7 foi escrito por mim na revisão da
0016 e avisava que "o endereço define qual fragmento é transferido (tabelado na spec)" —
verdadeiro e inútil. Não nomeou a armadilha que importava (`[N*4+0]` é endereço de byte, não
a parte alta do valor da palavra) porque eu também não tinha derivado a tabela na hora de
escrever o handoff — só na hora de revisar. Handoff escrito sem passar pela spec vale menos
do que aparenta.

Custo da rejeição: US$ 0,0192 e 5 minutos do trabalhador. Reexecutar é mais barato que
corrigir na branch, e mantém a separação de papéis — o orquestrador não escreve o emulador.

## 2026-07-27 — decisão do usuário: toolchain pinado (iter 0017c)

Das duas saídas registradas na entrada anterior, o usuário escolheu a segunda: pinar. Entrou
`rust-toolchain.toml` com `channel = "1.97.1"`, e o `ci.yml` deixou de escolher canal — a
versão vem de um lugar só.

O que faz o pin valer é o segundo meta-teste, não o arquivo: `toolchain_pin.rs` reprova
`ci.yml` que instale toolchain por conta própria. Sem isso, bastaria alguém reintroduzir um
`@stable` no workflow para os dois lados divergirem outra vez sem ninguém perceber — que é
o padrão que este projeto vem corrigindo desde o M0: regra que só existe em prosa degrada,
regra com meta-teste na CI não.

Custo assumido: lint e correção novos do compilador só chegam quando alguém subir o
`channel`, e isso vira item de ROADMAP com iteração própria. É o preço de o portão local
significar o que a CI mede.

## 2026-07-27 — o default que nunca foi uma decisão (troca para deepseek-v4-pro)

O usuário perguntou se o trabalhador estava rodando no "flash" quando deveria estar no "pro".
Respondi que esses nomes eram de outro fornecedor. Errado: `opencode models` lista
`deepseek/deepseek-v4-flash` e `deepseek/deepseek-v4-pro` ao lado de `deepseek-chat` e
`deepseek-reasoner`. Um comando decidiu o que eu tinha respondido de memória.

O achado de processo não é o modelo — é que da iteração 0009 à 0017 o trabalhador rodou na
geração anterior porque esse era o default escrito no `oc-iter.ps1` no dia 0008 e nunca mais
olhado. Nenhuma iteração escolheu esse modelo; todas herdaram a escolha. É a mesma classe de
falha do toolchain flutuante da 0017c, com o sinal trocado: lá um valor mudava sozinho, aqui
um valor nunca mudava — e nos dois casos ninguém estava decidindo.

A regra R1 ("não implemente hardware de memória, leia a spec") vale para o ferramental. O
ambiente é verificável em um comando; responder de memória sobre ele é o mesmo erro, num
lugar onde ele é ainda mais barato de evitar.

Preço pago: a segunda tentativa da 1.7 estava em voo com `deepseek-chat` e foi morta aos
18min36 (contra ~5min das iterações anteriores). Com isso perdi a comparação limpa que eu
tinha planejado — repetir o item com o mesmo modelo para medir só o efeito do handoff
corrigido. Se a próxima tentativa passar, não saberei se foi o handoff ou o modelo. Registrado
como perda, não escondido: a decisão do dono do projeto sobre a ferramenta vale mais que a
limpeza do meu experimento.

O eixo de comparação passa a ser **v4-pro (padrão) × v4-flash (barato)**, a pedido do usuário,
medido no `metricas.csv` por custo/iteração e por reprovação na revisão adversarial — não por
impressão de qualidade. Substitui o `chat × reasoner` que estava reservado para o item 1.8.

## 2026-07-27 — o teste de aceitação obrigatório estava errado

Na revisão que reprovou a PR #27 eu fixei uma regra: item cuja spec traz idioma canônico
ganha, no handoff, um teste de aceitação com valores literais derivados **pelo orquestrador**,
obrigatório no PR. A assimetria era o argumento — asserção com bytes concretos não pode ser
satisfeita por modelo mental errado.

O valor que eu escrevi (`r2 = 0x44DDCCBB`) estava errado; o correto é `0x44AABBCC`. Usei
`mem[0]`, que não pertence à palavra que começa em 1, e descartei `mem[3]`: indexação
misturada, base-0 no byte do topo e base-1 nos três de baixo.

Peguei carregando as seções da spec **antes** de o PR existir, para ter a revisão pronta na
chegada. Derivei a tabela por conta e não bati com o meu próprio handoff; conferi pela forma
algébrica e o segundo caminho concordou com o primeiro, não comigo. O trabalhador estava a
~6 minutos de execução, sem branch criada. Morto. Segunda rodada abortada na mesma noite.

Três consequências, e nenhuma delas é "abandonar a regra":

1. Valor literal imposto pelo orquestrador passa a ser **derivado duas vezes por caminhos
   diferentes**, e o handoff carrega a derivação junto com o resultado — para que o
   trabalhador tenha como reprovar o orquestrador. Handoff também é código: precisa de
   controle, não de confiança.
2. **Ler a spec do item enquanto a iteração roda** vira prática do orquestrador. Foi o que
   converteu um defeito entregue num defeito abortado, e custa espera que já existe.
3. Reprovei a 0017 por confundir via de byte e cometi o simétrico ao compor os bytes. A
   diferença não foi competência, foi ter quem revisasse: o trabalhador não tinha ninguém por
   cima, eu tinha a spec. É argumento a favor da revisão cruzada — não a favor de o revisor
   ser confiável.

Custo da noite até aqui, sem uma linha de emulador entregue: duas rodadas do trabalhador
abortadas (~24 min de execução) e três PRs de processo (#28, #29, #30). Registrado como está:
o projeto mede o processo, e o processo hoje gastou mais do que produziu.

## 2026-07-27 (noite, fechamento) — o primeiro PR de CPU que passou, e o que ele custou

O item 1.7 fechou na **terceira** tentativa, a primeira em `deepseek/deepseek-v4-pro`. É o
primeiro PR de CPU que passa na revisão adversarial sem defeito de emulação.

| | `deepseek-chat` (PR #27) | `deepseek-v4-pro` (PR #32) |
|---|---|---|
| Custo | US$ 0,0192 | US$ 0,1595 |
| Tempo / steps | 5min17 / 48 | 23min / 106 |
| Testes entregues | 20 | 25 (27 após correção) |
| Usou `tests/support/asm.rs` | não | sim |
| Nomes de teste em português | não | sim |
| Implementação | errada nos 4 opcodes | correta nos 16 casos |

**8,3× o custo por rodada. Mas a comparação por rodada é a errada:** o `chat` produziu duas
rodadas descartadas (uma reprovada, uma abortada), então o custo por iteração *aproveitada*
dele até aqui é infinito. Somando a rodada de correção (US$ 0,0524), o 1.7 custou **US$ 0,2119**
no `pro` contra **US$ 0,0192 jogados fora** no `chat`. Ainda é a fração de uma iteração do
Claude como trabalhador (US$ 5–8 medidos no gb-rs). A troca de modelo se paga.

O que o `pro` fez diferente e não foi sorte: seguiu as duas instruções explícitas do handoff
que o `chat` ignorou (usar os helpers existentes, nomear testes em português), e escreveu um
teste para o caso que a spec documenta como idioma — que era o defeito 2 da rejeição anterior.

### O código estava certo; o registro, não

Reprovei o registro e mandei corrigir na mesma branch. Três achados:

1. **Bateria de mutação irreproduzível.** Os 7 nomes de teste da tabela não existiam no
   arquivo. O placar `7/7` provavelmente estava certo — mas ninguém consegue reproduzi-lo
   pelos nomes publicados. Num projeto cuja segunda entrega é o registro empírico, isso é
   defeito de primeira classe, não detalhe de formatação.
2. **Lacuna de cobertura que a bateria não podia achar.** Ao re-executar os 7 mutantes, apliquei
   o mutante 5 (`reg_with_pending` → `reg`) só dentro de `fn lwl` — e nenhum dos 25 testes
   quebrou. Nenhum caso cobria o LWL lendo um load pendente; só o LWR era exercitado.
   Comportamento certo, não testado. Daí sai regra permanente: **helper compartilhado por N
   chamadores rende N mutantes independentes; mutar a definição testa 1 deles.** A bateria
   muta cada ponto de chamada.
3. **Teste de aceitação obrigatório ausente**, e o doc afirmando que existia. O trabalhador,
   ao corrigir, diagnosticou sozinho por que a soma fechava em 25 com um teste inexistente
   (tinha contado dois testes como um). Auto-auditoria de verdade.

### Três retratações do revisor no mesmo PR

Na sequência 0017→0018 o orquestrador errou três vezes e as três estão registradas:

1. **0017e** — o literal do teste de aceitação obrigatório estava errado (`0x44DDCCBB` por
   `0x44AABBCC`). Custou uma rodada abortada.
2. **A6 da revisão da #32** — acusei a citação de spec do PR (`L240`) de estar errada. `L235`
   é a seção-pai; `L240` é a seção certa. A citação do trabalhador era **mais precisa que a do
   meu próprio doc da 0017**.
3. **A2 da revisão da #32** — escrevi que o mutante 5 da tabela escapava. Não escapava: o
   mutante que escapava era o meu, mais fino. A acusação estava mal formulada e a lição, uma
   vez corrigida, ficou melhor (ver item 2 acima).

Taxa de erro do revisor nesta janela: 3 achados retratados. Vale dizer sem rodeio — o
orquestrador não é uma instância confiável, é só uma instância **independente**. O valor da
revisão cruzada vem da independência, não da autoridade.

### A tensão que este PR expôs: correção com rastro × limpeza de handoff

O teste de aceitação do trabalhador asseria `0x44DD_CCBB` — exatamente o literal errado que eu
publiquei e corrigi na 0017e. A asserção dele estava **certa** (o setup de memória era
invertido, e sob ele o valor bate), então não houve defeito. Mas o valor errado continuava
visível 6× no repositório porque corrigi "com rastro", deixando o erro registrado ao lado da
correção. Ancoragem plausível.

Os dois objetivos do projeto colidem aqui: manter o erro visível serve ao registro empírico;
tirá-lo de circulação serve à qualidade do handoff. Não resolvi por decreto. O que passa a
valer é uma separação de lugar: **o erro fica no doc da iteração (memória longa, que o
trabalhador não lê por R8); o STATUS (memória curta, que ele lê sempre) carrega só o valor
correto.** Isso preserva o registro sem deixar o literal errado no caminho de quem trabalha.

### Papercuts de infraestrutura pagos nesta sessão

- `Test-Connection -TargetName localhost -TcpPort 4096 -Quiet` devolve `False` mesmo com o
  `opencode serve` escutando (confirmado por `Get-NetTCPConnection`). O `oc-iter.ps1` tenta
  subir um segundo servidor toda rodada e perde ~60s. **Não corrigido** — vira iteração de
  infra própria, deliberadamente não mexido no meio de execução.
- Arquivo de tarefa escrito pelo Bash em `/tmp/...` e lido pelo PowerShell como `C:\tmp\...`:
  a rodada morreu na largada sem gastar token. Caminho absoluto do Windows resolve. Custo real:
  uma rodada perdida e a descoberta de que `oc-iter.ps1` faz `git checkout main` incondicional
  na linha 19 — o que só não quebrou a correção porque a tarefa mandava o trabalhador fazer
  checkout da branch do PR.

## 2026-08-01 — a orquestração muda de máquina (Windows → Linux) e de trabalhador

O repositório foi clonado limpo num Fedora 44. Três coisas quebraram, nenhuma de emulação: não
havia `pwsh`; o `oc-iter.ps1` procurava o opencode no layout do npm global do Windows
(`node_modules\opencode-ai\bin\opencode.exe`), e aqui a instalação é nativa; e o provider
`deepseek/` das 110 iterações anteriores não está autenticado nesta máquina — só `opencode-go`.

**Não portamos os `.ps1` para bash.** A CI roda em `ubuntu-latest`, que já traz `pwsh`, então ela
nunca esteve quebrada: só a máquina local estava. Um porte quebraria ~40 asserções literais de
sintaxe PowerShell espalhadas por `ci_oc_iter.rs`, `ci_oc_loop.rs`, `ci_scoreboard.rs`,
`gpu_scoreboard.rs` e `mutation_battery.rs`, mais 3 manifestos de mutação e o `ci.yml` — várias
iterações de trabalho de processo, sem uma linha de emulação, jogando fora as travas existentes.
Instalamos o pwsh (`dotnet tool install --global powershell`, 7.6.4, sem sudo).

**Troca de modelo.** `opencode-go/gpt-5.6-luna` com `--variant max`. A tabela do gateway em
2026-08-01: luna in US$ 0,10/M · out US$ 0,60/M · ctx 1,05 M, contra deepseek-v4-pro in US$ 0,435/M
· out US$ 0,87/M. Pela mediana das rodadas da `metricas.csv` (~100 k in / 30 k out), a iteração cai
de ~US$ 0,15 para ~US$ 0,03. Luna nunca rodou como trabalhador — só uma vez como orquestrador
(0136) — então a 0140 é o primeiro dado real desse papel. `opencode models --refresh` foi
necessário: o cache local estava velho e escondia o modelo.

**Três medições que decidiram o escopo, em vez de suposição.**

1. `Start-Process -WindowStyle Hidden` **lança** no pwsh Linux. Com `$ErrorActionPreference = "Stop"`
   isso mata a rodada na subida do daemon. Vira splat condicional a `$IsWindows`.
2. `Test-Connection -TargetName 127.0.0.1 -TcpPort` **funciona** (usa `TcpClient`, multiplataforma).
   Nada mudou — o que é sorte, porque `ci_oc_iter.rs:192` exige essa string literal e a linha 54 é
   âncora viva de `0101-matar-sessao.mut`.
3. O escape de aspas do `$promptArg` **não** é workaround exclusivo do Windows. O pwsh no Linux
   também achata o `-ArgumentList` num único `Arguments` reparseado com regras de aspas do Windows:
   com escape sai 1 argumento, sem escape saem 5 e `--version` vira flag do CLI. O plano aprovado
   mandava tornar isso condicional; a medição **refutou** e a mudança foi cortada. O rótulo
   "armadilha do Windows" no comentário descrevia a ORIGEM do bug, não a plataforma do comportamento
   — e essa distinção é o achado transferível desta iteração.

**Achado de processo: o job `mutantes` da CI sai verde medindo zero para alvo `.ps1`.** A guarda de
`mutantes.ps1:366` pula manifesto cuja `alvo:` não casa `^crates/psx-core/` (invariante 29). Os
`.resultado` de 0098 e 0101, sobre o mesmo alvo, são legítimos porque rodaram ANTES da guarda, que
entrou na 0125. Mas a justificativa da invariante — "mutante fora do psx-core nunca é recompilado" —
**não se aplica** a `.ps1`: `ci_oc_iter.rs` lê o arquivo do disco em runtime, então mutá-lo afeta o
teste sem recompilação nenhuma. A guarda super-bloqueia este caso. A bateria da 0139 foi manual;
a dívida encosta em 10.58 e 10.33 e não foi consertada aqui (R4).

## 2026-08-02 — nova categoria de erro: `[AllowNull()]` não impede a coerção de tipo do PowerShell

Na 0168 (`scripts/lib/tty-veredito.ps1`), um parâmetro `[AllowNull()][string]$Gabarito` deveria
aceitar `$null` para distinguir "gabarito ausente" de "gabarito vazio". Não aceitou: o PowerShell
converte `$null` em `""` ao vincular o argumento ao tipo `[string]` — a conversão de tipo roda
antes da validação de `AllowNull()`, então o atributo nunca chega a ser consultado para esse caso.
O teste sintético pego (`gabarito_ausente_nao_e_confundido_com_diferenca`) devolveu `difere` em
vez de `sem-gabarito`. Correção: remover a anotação de tipo do parâmetro (deixar sem tipo) — sem
conversão declarada, `$null` chega como `$null` de verdade. Categoria nova para o campo "Erros de
primeira tentativa" dos docs de iteração: **API-PowerShell**, ao lado de API-Rust.

## 2026-08-03 — exceção de executor: o orquestrador implementa a escada da 0193

Primeira sessão de jogo real do projeto (Crash Bandicoot no `psx-desktop`, jogado pelo
usuário) produziu quatro sintomas: jogo ~2× rápido, áudio estourado, HUD/sprites
invisíveis e artefatos. A exploração fechou as causas (achados 0193.1-0193.7) e derrubou
uma hipótese no caminho: não há auto-ack nem hack de vblank no core — o vblank dispara a
59,82 Hz; o acelerador real é a CPU cobrar 1 ciclo/instrução sem custo de memória, com a
GPU desenhando em 0 ciclos (0193.4).

**Decisão do usuário, perguntada explicitamente:** a escada de correção (texturas →
velocidade → áudio → 24bpp/modulação) será implementada **diretamente pelo orquestrador**,
não pelo trabalhador. É exceção à divisão de papéis deste documento e vale só para a
escada; o protocolo (teste-antes, 1 item por PR, spec antes de hardware, bateria de
mutação no psx-core) continua o mesmo. Registrado porque o dado empírico "quem codificou"
muda a leitura das métricas dessas iterações no relatório final. Prioridade escolhida
pelo usuário: texturas primeiro.
