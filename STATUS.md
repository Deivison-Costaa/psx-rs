# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0118** — o shell nao pedia NADA ao disco (2 comandos em 400 M passos). A causa nao era o
CD-ROM: ele girava num laco esperando o contador do timer 2, e `lhu`/`sh` nas portas
`0x1F801100..2F` caiam no braco-sumidouro do `bus.rs` — os timers estavam certos e inalcancaveis
pela largura de acesso que o kernel usa (invariante 25, 2a vez). Corrigido; o contador agora
avanca e o laco sai (ROADMAP 4.4m).

## Próxima tarefa

**ROADMAP 4.4n — driver de CD-ROM do kernel nao conclui o `GetStat`.**
Medido na 0118: o laco quente agora e `0x8003D6FC`, decodificado a mao —
`lui $t7,0x8008 / lw $t7,0x3C58($t7) / slti $at,$t7,2 / beq` — ou seja
`while ([0x80083C58] >= 2) ;`. E o estado do driver de CD do kernel, e ele nunca cai abaixo de 2
depois do `GetStat` enviado no passo 87 464 254 (`pc=0x80057554`). Nenhum outro comando sai depois
disso, e `HINTSTS==INT1` nunca acontece.
**Meça primeiro** (invariante 26): instrumente TODA escrita em `0x80083C58` (qual PC escreve, com
que valor) e o caminho do handler de IRQ2 — o `psx-estado/instrumentacao/cdshell.rs` ja decodifica
os portos do CD e tem histograma de PC; acrescente o watch da variavel em vez de escrever outro
harness. Confira se o handler chega a rodar e se o ack do `HCLRCTL` sai na ordem certa (0114).
Spec: `docs/reference/06-cdrom.md` (§ GetStat, § HCLRCTL) e `docs/reference/13-kernel-bios.md`
(§ CdromDecodeIRQ, § callbacks do CD). Arquivos-alvo: `crates/psx-core/src/cdrom.rs`.
Armadilha conhecida: a resposta do `GetStat` e um INT3 de uma so entrega; se o driver espera um
evento de conclusao que so o segundo INT produz, o estado fica preso — nao "conserte" o cdrom.rs
sem antes ver QUEM escreve a variavel.
Critério de aceitação: `[0x80083C58]` cai abaixo de 2 e o shell emite um comando novo ao disco.
Invariantes relevantes: 24, 25, 26.

**Referencia externa (30/07):** captura canonica do DuckStation em
`psx-estado/referencias/tela-de-boot-duckstation.png`; fundo (180,180,180) e cores do losango
CONFIRMADOS iguais aos nossos; sem "®" na tela real. Diferenca visual restante no logo: costuras
de gouraud no losango (candidato 10.14).

## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- **Escopo de commit é UM único identificador `[a-z0-9-]`.** `feat(bus,cpu)` reprova no
  `commit-lint`; quando a mudança toca dois módulos, escolha o principal e cite o outro no
  resumo. Custou uma reescrita de 4 mensagens no PR #36 (ver `0022-scratchpad-isc.md`).
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.

## Placar de testes

Workspace: **790** testes.

## Bloqueios

- **4.4 Boot de jogo**: sem bloqueio conhecido; desde a 0115 o boot passa do handshake do
  controle e para no driver de GPU do kernel; a 0118 destravou o timer e o boot chega ao driver de CD do kernel (4.4n). Imagens de disco ficam fora do repositório, em
  `.../Programacao com agentes/roms/extraido/`. **Nunca commitar imagem de disco.**
