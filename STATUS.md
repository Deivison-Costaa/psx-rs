# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0119** — diagnostico, sem codigo de producao. A troca do `GetStat` esta correta porto a porto
(comando, poll do HSTS, leitura do HINTSTS, leitura da resposta, ack `HCLRCTL=07h`) e
`[0x80083C58]` NAO esta travada: ela cicla posta/expira/retenta com cadencia de quadro, chegando a
zero uma vez. Quatro hipoteses refutadas, incluindo duas do handoff anterior (ROADMAP 4.4n).

## Próxima tarefa

**ROADMAP 4.4o — comparar a sequencia de comandos do CD contra emulador de referencia.**
Depois do `GetStat` do passo 87 464 254 o shell nao toca mais no drive em 312 M passos; ele fica
num laco por quadro lendo o controle (endereco `0x01`, ~1 por quadro) e ciclando o contador
interno. Nenhum `Setloc`/`ReadN`/`GetID`, `HINTSTS==INT1` zero, e 417 entradas no handler depois
do `GetStat` (IRQs correm).
**Nao escreva outro harness cego** (invariante 27): rode a MESMA BIOS (`bios/SCPH1001.BIN`) com o
MESMO disco (`../roms/extraido/Crash Bandicoot (USA).cue`) no DuckStation de
`psx-estado/referencias/`, ligue o log de CD-ROM dele e compare a sequencia de comandos com a
nossa (`Test(20h)`, `GetStat`, e nada). Duas saidas possiveis, as duas uteis: (a) no real sai um
`GetID` depois do `GetStat` — entao o alvo e o que provoca esse `GetID`; (b) no real tambem nao
sai — entao o shell espera outra coisa e o alvo muda de subsistema.
Harness ja pronto: `psx-estado/instrumentacao/cdstate.rs` (watch de variavel, janela de portos do
CD, contagem do handler, bytes de endereco do SIO0).
Spec: `docs/reference/06-cdrom.md` (§ GetID, § GetStat) e `docs/reference/13-kernel-bios.md`.
Armadilha conhecida: `[0x80083C58]` NAO e comprovadamente o estado do driver de CD — foi um chute
meu na 0118. Nao construa o proximo item sobre esse nome.
Critério de aceitação: a diferenca entre as duas sequencias esta medida e escrita, e o proximo
alvo sai dela.
Invariantes relevantes: 26, 27.

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
  controle e para no driver de GPU do kernel; desde a 0118 o shell roda e nao pede nada ao disco; proximo passo e comparar com referencia (4.4o). Imagens de disco ficam fora do repositório, em
  `.../Programacao com agentes/roms/extraido/`. **Nunca commitar imagem de disco.**
