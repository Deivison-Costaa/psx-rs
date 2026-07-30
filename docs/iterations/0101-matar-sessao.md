# 0101 — matar-sessao

- **Data:** 2026-07-30
- **Item do roadmap:** 10.31
- **Objetivo:** rodada morta parar de escrever. Encerrar a **sessão** no daemon `opencode serve`,
  e não só o processo cliente, sempre que a rodada terminar em falha.

## Revisão do PR anterior

PR #116 (iter 0100), do próprio orquestrador: quatro checks verdes, `headRefOid` conferido antes do
merge, bateria 5/5 depois de um mutante sobrevivente corrigido. Sem achados novos.

## Spec consultada

Nenhuma seção de spec de hardware. O comportamento autoritativo é o do `opencode` e o da pilha de
rede do Windows, ambos observados ao vivo (medições nas notas 1 e 2).

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que o comportamento real é | Como foi pego |
|---|---|---|---|---|
| 1 | processo | Que `Stop-Process` no processo do `opencode run --attach` encerrasse a rodada | Mata o **cliente**. A sessão vive no daemon e continua escrevendo: depois de a rodada ser declarada morta, o reflog registrou commit, `reset --hard HEAD~1`, `checkout main`, **3 commits direto na main local** e rename de branch, ao longo de ~3 min | Guarda de árvore suja do `oc-loop` parou o loop; fui ler o reflog para entender e achei as escritas pós-morte |
| 2 | ambiente | Que `Test-Connection -TargetName localhost -TcpPort 4096` detectasse o daemon | `localhost` resolve para **::1 (IPv6)** e o `opencode serve` escuta em **127.0.0.1 (IPv4)**. Com o daemon vivo e confirmado por `netstat`, `localhost` responde **False** e `127.0.0.1` responde **True** | Provei a função contra um listener falso: com a porta ocupada ela não esperou. Achei que o teste falso é que estava errado, medi contra o daemon real, e era o script |
| 3 | teste | Que procurar `Get-Process`/`opencode` "depois do início da função" provasse que a função os usa | O resto do arquivo também cita `opencode`, então a busca ficaria verde com o corpo da função esvaziado — a mesma família dos mutantes sobreviventes das iterações 0094, 0098 e 0100 | Percebi antes de rodar a bateria e recortei o corpo da função (`corpo_da_funcao`). Foi a primeira vez no dia que peguei esse padrão **antes** de a bateria pegar |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0101-matar-sessao.mut

| Registro | Rótulo | Testes que pegaram, conforme o `.resultado` |
|---|---|---|
| m1 | função de encerramento renomeada | `rodada_morta_encerra_a_sessao_no_daemon_e_nao_so_o_cliente` |
| m2 | volta a alcançar só o cliente | `rodada_morta_encerra_a_sessao_no_daemon_e_nao_so_o_cliente` |
| m3 | encerramento amarrado a um rótulo só | `encerramento_da_sessao_esta_no_caminho_de_falha` |
| m4 | não espera a porta liberar | `apos_matar_o_daemon_espera_a_porta_liberar` |
| m5 | detecção volta para `localhost` | `deteccao_da_porta_usa_o_endereco_em_que_o_daemon_escuta` |
| c1 | teto de espera de 30 para 40 s | sobreviveu |
| c2 | sondagem da porta de 1 para 2 s | sobreviveu |

## Placar antes → depois

Workspace: **699** → **703** testes (+4 em `ci_oc_iter`).

O efeito é operacional e já foi medido em produção antes do conserto: uma rodada morta escreveu
sete vezes na árvore compartilhada, três delas commits na `main` local, num intervalo de 2 min 25 s.

## Revisão cruzada (orquestrador)

Iteração inteira do orquestrador, durante a parada deliberada do loop.

## Decisões e notas

1. **Verificação de comportamento contra o daemon real, não só contra o texto do script.** Subi um
   `opencode serve` de verdade e rodei a função: `127.0.0.1:4096` respondia **True** antes,
   **False** depois, com **zero** processos `opencode` restantes, em 2 s.
2. **O conserto do endereço entrou junto, e isso é desvio de escopo assumido (R4).** São a mesma
   chamada de uma palavra em três lugares, e a espera nova depende dela: deixar a detecção quebrada
   ao lado do conserto seria entregar algo que não funciona. O efeito colateral bom é que a subida
   do daemon deixa de ser espúria em toda rodada.
3. **Matar o daemon inteiro em vez de abortar só a sessão.** É a opção grossa; a fina exigiria
   falar a API do `opencode`. O desenho do loop é de uma rodada por vez, então não há sessão
   legítima a preservar, e o próprio `oc-iter.ps1` sobe um daemon novo quando a porta não responde.
4. **O que este item NÃO cobre:** rodada que termina em `ok` não passa pelo encerramento. Se uma
   sessão sobreviver a um término bem-sucedido, o zumbi volta — a guarda de árvore suja do
   `oc-loop` continua sendo a segunda linha de defesa.
