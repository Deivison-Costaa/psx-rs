# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0122** — o `GetID` respondia a linha **No Disk** da spec (`INT5 08h,40h`) mesmo com disco dentro.
Agora responde Licensed:Mode2 (`INT2 02h,00h,20h,00h,'S','C','E','A'`) quando ha disco. **O laco de
retentativa acabou:** de 45 comandos repetindo para 4 lineares, e o shell pediu o proximo comando.

## Próxima tarefa

**ROADMAP 4.4r — `GetTOC` (1Eh) nunca arma a segunda resposta.**
Medido na 0122: depois do `GetID` o shell emite `GetTOC` no passo 88 380 174 e nao pede mais nada.
`send_command` nao tem braco para `0x1E`; ele cai no `_ =>`, que empilha o stat, seta `intsts = 3`
e faz `busy = false` **sem tocar em `pending_second`**. A spec (§ Second Responses,
`docs/reference/06-cdrom.md` L2002) exige `1Eh ReadTOC — INT3(late-stat), INT2(stat)`, e a
§ ReadTOC (L961) avisa: *"rather slow, the second response appears after about 1 second delay"*.
Alvo: `crates/psx-core/src/cdrom.rs`, braco novo `0x1E` no `send_command`. O caso 1 do
`deliver_second` (Init) ja faz exatamente `INT2(stat)` — reusar `pending_second = 1`.
Armadilha conhecida: o `first_response_cycles` ja trata `0x1E` como o atraso longo (0x13CCE), entao
NAO mexa nele. E a segunda resposta continua saindo no ack do guest, nao por tempo (buraco 10.54):
o "1 second delay" da spec nao e modelado, e isso e assumido, nao esquecido.
Critério de aceitação: o `cdstate.rs` mostra um comando novo depois do `GetTOC` — a referencia do
DuckStation em `psx-estado/referencias/` continua sendo o gabarito da cadeia.
Invariantes relevantes: 26, 28.

**Meta em vigor (ordem do usuario, 31/07):** emendar as iteracoes ate o M4 fechar, sem parar entre
PRs. Pronto = **menu navegavel no `psx-desktop`**. Parada: 5 iteracoes fechadas sem o jogo bootar,
ou falha 3x no mesmo passo. Risco anotado: o unico disco disponivel e o Crash Bandicoot, que e 3D —
5.4b/5.4c/5.4d e 5.5 (GTE) estao abertos e podem entrar na conta.

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
- **`ROADMAP.md` estava a 3 bytes do teto na 0121.** As linhas ja fechadas do 4.4 foram
  comprimidas (o contexto mora em `docs/iterations/`), sobrando ~470 bytes. Encurtar, nunca apagar.

## Placar de testes

Workspace: **808** testes.

## Bloqueios

- **4.4 Boot de jogo**: sem bloqueio conhecido; o boot passa do handshake do controle, do logo
  SONY e agora pede o `GetID`, mas recebe "sem disco" e repete para sempre (4.4q). Imagens de disco
  ficam fora do repositório, em `.../Programacao com agentes/roms/extraido/`.
  **Nunca commitar imagem de disco.**
