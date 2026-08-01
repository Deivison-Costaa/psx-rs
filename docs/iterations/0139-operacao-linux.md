# 0139 — operacao-linux

- **Data:** 2026-08-01
- **Item do roadmap:** 10.60
- **Objetivo:** a orquestração voltar a rodar depois da mudança de máquina (Windows → Fedora 44):
  achar o binário do `opencode` nos dois sistemas, parar de emitir flag de janela fora do Windows,
  e trocar o trabalhador para `opencode-go/gpt-5.6-luna --variant max`.

## Revisão do PR anterior

PR #155 (iter 0137), diagnóstico puro do congelamento pós-boot. Sem achados novos. O handoff dela
continua válido — só foi **adiado uma iteração**: ao conferir o orçamento do `ROADMAP.md` para
encaixar o item desta iteração, medi que 64 itens fechados ocupam 3906 dos 10000 bytes (39% da
escada é histórico), e sobraram 10 bytes livres. Marcar o checkbox do 4.5 quando ele fechar não
caberia. Então a 0140 é o arquivamento (ROADMAP 10.61) e a 4.5 vira 0141. Decisão do usuário,
consultado: a 0140 serve também de rodada de estreia do luna como trabalhador — tarefa de baixo
risco é melhor lugar para descobrir como o modelo se comporta do que o diagnóstico mais difícil do
projeto.

## Spec consultada

Nenhuma seção de spec de hardware — não há hardware nesta iteração. O comportamento autoritativo é
o do `opencode` e o da pilha PowerShell/.NET no Linux, ambos observados ao vivo (os três probes das
notas 1 a 3).

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição mostrou | Como foi pego |
|---|---|---|---|---|
| 1 | ambiente | Que o escape de aspas do `$promptArg` fosse workaround exclusivo do Windows e devesse virar condicional a `$IsWindows` — estava assim no plano aprovado | **Falso.** O `Start-Process` do PowerShell no Linux também achata o `-ArgumentList` num único `Arguments` reparseado com regras de aspas do Windows. Com escape: `["a b \"c\" -d --version"]` (1 argumento). Sem escape: `["a","b","c","-d","--version"]` (5 argumentos, e `--version` vira flag do CLI) | Probe B, antes de escrever o teste. O rótulo "armadilha do Windows" no comentário do script descrevia a **origem** do bug, não a plataforma do comportamento. Implementar sem medir teria reencenado a falha de 28/07 11:52, em que o opencode imprimiu o help e saiu 0 |
| 2 | processo | Que a bateria pudesse sair do `scripts/mutantes.ps1`, já que 0098 e 0101 têm `.resultado` gerado por ele sobre o mesmo alvo `.ps1` | O script **pula** alvo fora de `crates/psx-core/` (`mutantes.ps1:366-374`, invariante 29). Os dois `.resultado` antigos são legítimos porque rodaram **antes** da guarda existir — ela entrou na 0125 | Li a guarda ao conferir a premissa. Consequência: o job `mutantes` da CI sai **verde medindo zero** para alvo `.ps1`, e a justificativa da invariante 29 ("mutante fora do psx-core nunca é recompilado") **não se aplica** aqui — `ci_oc_iter.rs` lê o `.ps1` do disco em runtime. Dívida anotada abaixo |
| 3 | processo | Escrevi os 7 testes e implementei antes de rodar, isto é, nunca vi o vermelho — violação da R5 | — | Percebi ao ir rodar. Recuperado com `git stash push -- scripts/`: 5 testes novos falharam **por asserção** e os 8 antigos seguiram verdes; só então restaurei e confirmei 18/18 |

## Bateria de mutação

Placar da bateria: 7/7 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0139-operacao-linux.mut

| Registro | Rótulo | Testes que pegaram, conforme o `.resultado` |
|---|---|---|
| m1 | descoberta volta a assumir o layout do npm | `descoberta_do_binario_nao_e_so_o_caminho_do_npm_do_windows` |
| m2 | bifurca por plataforma mas os dois braços montam caminho de npm | `descoberta_do_binario_nao_e_so_o_caminho_do_npm_do_windows` |
| m3 | guarda da flag de janela some | `flag_de_janela_do_windows_nao_vaza_para_o_daemon_no_linux` |
| m4 | subida do daemon volta a parâmetro nomeado | `flag_de_janela_do_windows_nao_vaza_para_o_daemon_no_linux` |
| m5 | modelo volta ao provider sem autenticação | `modelo_padrao_e_o_trabalhador_em_vigor` |
| m6 | variante declarada e nunca repassada | `variante_do_modelo_e_repassada_ao_cli_e_nao_so_declarada`; `variante_vazia_nao_emite_flag_orfa` |
| m7 | flag da variante sem guarda, vazia vira flag órfã | `variante_vazia_nao_emite_flag_orfa` |
| c1 | texto da mensagem de erro reescrito | — (sobreviveu, como esperado) |
| c2 | `-m` vira `--model` | — (sobreviveu, como esperado) |

**Bateria MANUAL** (invariante 29): o `mutantes.ps1` pula este alvo. Aplicada por runner descartável
que casa por linha inteira, roda `cargo test -p psx-core --test ci_oc_iter` e restaura o arquivo num
`finally`; `git diff scripts/` saiu vazio ao fim. A prova para o revisor é o `.resultado` mais a
reaplicação de pelo menos m3 e m6, que são os dois mais sutis.

## Placar antes → depois

Workspace: 861 → 868 testes (+5 em `ci_oc_iter`, +2 em `ci_oc_loop`).

Primeira suíte verde desta máquina, e a primeira **com BIOS e disco presentes**: os testes de
`bios_boot`, `testevent_descritor`, `espera_tela_sce`, `evento_consumo_shell` e
`cdrom_evento_kernel`, que na CI pulam por ausência de imagem, executaram de verdade e passaram.

## Revisão cruzada (orquestrador)

<!-- Preenchido na revisão do PR. -->

## Decisões e notas

**1. Probe A — `Start-Process -WindowStyle` no Linux.** Lança:
`The parameter '-WindowStyle' is not supported for the cmdlet 'Start-Process' on this edition of PowerShell`.
Com `$ErrorActionPreference = "Stop"` (`oc-iter.ps1:31`) isso mata a rodada na subida do daemon,
antes do primeiro token. Por isso a flag passou a ir por splat, só sob `$IsWindows`.

**2. Probe C — `Test-Connection -TargetName 127.0.0.1 -TcpPort` no Linux.** Funciona: não lança e
devolve `False` para porta morta. O parameter set `-TcpPort` usa `TcpClient`, que é multiplataforma
— quem quebraria sem root é o ICMP do parameter set default, que o script não usa. **Nada mudou
aqui**, e era o desfecho que mais importava: `ci_oc_iter.rs:192` exige a string literal
`-TargetName 127.0.0.1 -TcpPort`, e a linha 54 é âncora viva de `0101-matar-sessao.mut`.

**3. Probe B.** Ver "Erros de primeira tentativa" nº 1. É o achado mais caro da iteração e o único
que mudou o escopo aprovado: a alteração do `$promptArg` foi **cortada**.

**4. Âncoras preservadas.** As 14 linhas de `oc-iter.ps1` ancoradas por `0098-oc-iter-travamento.mut`
e `0101-matar-sessao.mut` foram conferidas contra o diff antes do commit: nenhuma foi tocada. A
edição da flag de janela fica na linha imediatamente seguinte a uma delas
(`$up = Test-Connection ...`), que ficou intacta.

**5. Por que não portar os `.ps1` para bash.** A CI roda em `ubuntu-latest`, que já traz `pwsh`
pré-instalado, então ela nunca esteve quebrada — só a máquina local estava. Um porte para `.sh`
quebraria ~40 asserções literais de sintaxe PowerShell em `ci_oc_iter.rs`, `ci_oc_loop.rs`,
`ci_scoreboard.rs`, `gpu_scoreboard.rs` e `mutation_battery.rs`, mais 3 manifestos de mutação e o
`ci.yml`: várias iterações de trabalho de processo, sem uma linha de emulação, jogando fora as
travas que hoje existem. O pwsh foi instalado por `dotnet tool install --global powershell` (7.6.4,
sem sudo, com link em `~/.local/bin`).

**6. Dívidas encontradas, não consertadas aqui (R4).**
- `ROADMAP.md` tem **`10.42` duplicado**: dois itens diferentes com o mesmo número. `status_handoff.rs`
  indexa por conjunto, então a duplicata é invisível para a máquina e um handoff que citasse "10.42"
  apontaria para dois itens.
- A guarda de `mutantes.ps1:366` **super-bloqueia** alvo `.ps1`: a justificativa da invariante 29 é
  recompilação, que não se aplica a arquivo lido em runtime. Encosta em 10.58 e 10.33.
- `scripts/mutantes.ps1:528` usa `$statusFinal`, variável nunca atribuída — a "restauração camada 5"
  é um no-op silencioso.
- `scripts/fetch-test-exes.ps1` diz que confere o hash após o download; o hash é apenas impresso.
- `.claude/skills/iterate/SKILL.md:88-96` e `:101` afirmam que `mutantes.ps1` e `scoreboard.ps1`
  "ainda não existem".
- `STATUS.md` afirmava `reasoningEffort max ja configurado em ~/.config/opencode`; o arquivo desta
  máquina tem só o `$schema`. Corrigido nesta iteração — quem passa `max` agora é `--variant`.
