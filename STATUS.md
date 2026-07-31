# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0121** — a primeira resposta do CD-ROM saia em ZERO ciclo; agora e evento do `scheduler` com o
atraso da spec (0xC4E1; 0x13CCE para Init/ReadTOC). **O `GetID` apareceu:** de 2 comandos em 400 M
passos para 45, na sequencia `GetStat -> GetID` da referencia. `scheduler.rs` ganhou `cancel`.

## Próxima tarefa

**ROADMAP 4.4q — `GetID` responde "sem disco" com disco dentro.**
`cdrom.rs` (`deliver_second`, caso 2) empilha sempre `08h, 40h, 00h x6` e `intsts = 5`. A spec
(§ GetID, `docs/reference/06-cdrom.md` L1139-1153) identifica essa linha como **No Disk**. Para um
disco licenciado Mode2 a linha certa e `INT2(02h,00h, 20h,00h, 53h,43h,45h,4xh)`, e L1170-1173 diz
que *"the PSX refuses to boot if it doesn't match up for the local region"*. BIOS `SCPH1001` e
NTSC-U, entao a quarta letra e `'A'` — `SCEA`.
Sintoma que confirma: o shell repete `GetStat, GetStat, GetID` a cada ~18,9 M passos, para sempre.
Alvo: `crates/psx-core/src/cdrom.rs` (so o caso 2 do `deliver_second`); a resposta passa a depender
de `disc_inserted`, mantendo a linha No Disk quando nao ha disco.
Armadilha conhecida: `cdrom_regs.rs::getid_sem_disco_retorna_int5` cobre o caminho SEM disco e tem
de continuar verde — o item so muda o caminho COM disco. Regiao fixada em `SCEA` e buraco assumido:
o certo e deriva-la do setor de licenca do `.bin` (abra item de backlog, nao implemente agora).
Critério de aceitação: o `cdstate.rs` mostra `Setloc`/`SeekL`/`ReadN` depois do `GetID`.
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

Workspace: **800** testes.

## Bloqueios

- **4.4 Boot de jogo**: sem bloqueio conhecido; o boot passa do handshake do controle, do logo
  SONY e agora pede o `GetID`, mas recebe "sem disco" e repete para sempre (4.4q). Imagens de disco
  ficam fora do repositório, em `.../Programacao com agentes/roms/extraido/`.
  **Nunca commitar imagem de disco.**
