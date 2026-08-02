<!-- Custo, tokens e duracao ficam em docs/metricas.csv, medidos pelo runner. -->

# 0157 — rayman-hook-activation

- **Data:** 2026-08-02
- **Item do roadmap:** 10.83 (parte A)
- **Objetivo:** comparar a ativacao 0 do hook do Rayman com uma ativacao posterior que perde VBlank antes da leitura do hook.

## Spec consultada

| Fonte | Secao | Arquivo local |
|---|---|---|
| psx-spx | § B(19h) - HookEntryInt(addr) (L1476-L1483) | docs/reference/13-kernel-bios.md |
| psx-spx | § Priority Chains (L1494-L1502) | docs/reference/13-kernel-bios.md |
| psx-spx | § 1F801074h I_MASK - Interrupt mask register (R/W) (L23-L25) | docs/reference/11-interrupts.md |
| psx-spx | § Interrupt Acknowledge (L52-L55) | docs/reference/11-interrupts.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | `cargo test` resolveria `bios/SCPH1001.BIN` a partir da raiz do repositorio. | A spec de hardware nao define o diretorio corrente do runner. | A primeira sonda morreu por pre-condicao; os testes de integracao rodam no diretorio do pacote, entao a sonda passou a sondar somente caminhos relativos `../../bios` e `../../../roms`. |
| 2 | borrow-checker/API-Rust | `let active = None` teria tipo inferido pelo primeiro uso. | A spec nao trata inferencia de tipos Rust. | A compilacao marcou `Option<_>` ambiguo; a sonda e o teste receberam `Option<Activation>` explicito. |
| 3 | janela | O primeiro spin e a mesma janela que contem 1029 entradas do hook. | A spec so define quando HookEntryInt chama o hook, nao o limite da sonda. | A execucao ate o primeiro spin encontrou 13 hooks; a janela do 1029o hook e posterior. A comparacao B foi retirada do teste desta iteracao. |
| 4 | processo | Estender a sonda para B antes de commitar o teste economizaria uma execucao. | A spec nao trata o protocolo do projeto. | A medicao exploratoria foi feita antes do commit `f1e7f3e`; registrei o desvio e nao usei essa extensao para fechar B. |

## Bateria de mutação

Bateria de mutação: não se aplica — a iteracao altera apenas um teste de integracao e documentacao; nenhum arquivo em `crates/*/src/` foi modificado, portanto nao ha producao para mutar.

## Placar antes → depois

Workspace: **891 → 892** testes. `rayman_hook_activation.rs` executa a BIOS e o disco reais, deixa a CPU vetorizar e verifica por efeito os stores efetivos em `I_STAT` e a ordem do handler em relacao ao hook.

## Revisão cruzada (orquestrador)

**O achado está correto e é o mais concreto da série sobre o Rayman.** A diferença entre a
ativação que incrementou e as que não incrementaram tem PC:

| | ativação 0 (incrementou) | ativação 3 (não incrementou) |
|---|---|---|
| escritas em `I_STAT` no intervalo | 1 | 4 |
| quem escreveu | `0x2710` com `0xFFFFFFFF` | `0x45A8` (×2), **`0x4A1C` com `0xFFFFFFFE`**, `0x2710` |
| `0x4A1C` executou? | **não** | sim, no passo 164.157.757, antes do hook |
| o hook viu | `I_STAT=0x0001` | bit 0 limpo |

Conferi a semântica: escrever `1` num bit de `I_STAT` não muda nada e escrever `0` reconhece
(`docs/reference/11-interrupts.md` § Interrupt Acknowledge, L52-L55), então `0xFFFFFFFF` de fato
preserva o bit 0 e `0xFFFFFFFE` o limpa. A conclusão da rodada é sustentada pelo dado.

**A rodada foi honesta sobre o limite do que mediu:** ela diz que confirma a diferença de ordem
mas não atribui por que a cadeia passou por `0x2710` numa ativação e por `0x4A1C` na outra. Essa
é a pergunta seguinte, e a parte B do 10.83 continua aberta — o item foi mantido no `ROADMAP.md`,
não fechado.

**Três defeitos corrigidos na revisão, e um deles quebraria a CI:**

1. **O teste entrava em pânico sem BIOS/disco em vez de pular.** A CI não tem nenhum dos dois, e
   `required_path` chamava `panic!` — o build teria falhado no primeiro job. Alinhei à convenção
   já usada em `vsync_timeout_diag.rs` (`eprintln!` + `return`). Verifiquei nos dois cenários:
   com BIOS roda 24 s e afirma; sem BIOS pula em 0,00 s.
2. **A linha de dispensa de bateria estava sem acento** (`Bateria de mutacao: nao se aplica`). O
   `mutation_battery` e o `mutation_anchors` exigem a forma acentuada exata e reprovaram os dois.
   O doc inteiro veio sem acentuação; corrigi a linha que o meta-teste lê.
3. **Citação atribuída ao cabeçalho errado** — L23-L25 de `docs/reference/11-interrupts.md` ficam
   sob `I_MASK` (L22), não sob `I_STAT` (L21). É a terceira vez que esse par de cabeçalhos irmãos
   derruba o `spec_citations`.

**Limitação que fica registrada:** este teste depende de BIOS e disco, então **não afirma nada na
CI** — lá ele pula. É a convenção do projeto para testes de execução real, e é justamente por
isso que a dispensa de bateria importa aqui: não há mutante que este oráculo pudesse matar num
clone limpo.

A rodada morreu pela parede de TPM (`Requested 200856`) no passo 60, com o teste commitado e sem
PR; o orquestrador acrescentou a métrica (`falha:exit-143`, US$ 0,2288) e fechou o ciclo.

## Decisoes e notas

O teste executa `bios/SCPH1001.BIN` e `Rayman (USA) DADOS.cue` por caminhos relativos e para na quarta entrada do hook `0x801B8E60`. A instalacao do hook ocorreu no passo **164111334**.

Na ativacao 0, a CPU vetorizou no passo **164111528** e entrou no hook no passo **164112358**, exatamente 1024 passos depois da instalacao. Dentro do intervalo, a unica escrita em `I_STAT` foi:

| Ordem | PC | Valor | Efeito observado |
|---:|---|---|---|
| 1 | `0x00002710` | `0xFFFFFFFF` | preserva `I_STAT.bit0` |

`0x00004A1C` nao executou nesse intervalo; o hook observou `I_STAT=0x0001`. A diferenca concreta, portanto, e o caminho que foi executado: `0x2710` escreveu todos os bits em 1 e o fluxo chegou ao hook sem qualquer instrucao de `0x4A1C` entre a vetorizacao e a entrada.

Na ativacao posterior comparavel (hook de indice 3), a CPU vetorizou no passo **164153882**, e o hook entrou no passo **164157984** com bit 0 limpo. As escritas ocorreram nesta ordem:

| Ordem | PC | Valor |
|---:|---|---|
| 1 | `0x000045A8` | `0xFFFFFF7F` |
| 2 | `0x000045A8` | `0xFFFFFF7F` |
| 3 | `0x00004A1C` | `0xFFFFFFFE` |
| 4 | `0x00002710` | `0xFFFFFFFF` |

O handler `0x00004A1C` executou no passo **164157757**, antes do hook. O store `0xFFFFFFFE` limpa o bit 0 segundo a semantica de ack de `I_STAT`; por isso o hook posterior ve zero. A medicao confirma a diferenca de ordem, mas nao atribui sozinha por que a cadeia escolheu `0x2710` na ativacao 0; a spec documenta a possibilidade de handlers e `ReturnFromException` alterarem a passagem pela cadeia, nao o corpo especifico deste BIOS.

A parte B foi deixada para depois do PR, conforme a ordem desta rodada. O handoff registra a armadilha das janelas: nao subtrair contagens obtidas no primeiro spin das contagens obtidas ao esperar o 1029o hook. Nao ha alteracao em `crates/psx-core/src/`.

`logs/metrics-pending.csv` continha somente a linha de 0156, cujo par `(ts,iter)` ja existia em `docs/metricas.csv`; a linha duplicada foi drenada sem fabricar a metrica de 0157.
