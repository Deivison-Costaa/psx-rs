# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0120** — diagnostico com oraculo externo. DuckStation com a MESMA BIOS e o MESMO disco
(`PatchFastBoot=false`) responde `Getstat Stat=0x02` — identico ao nosso — e emite `GetID` logo em
seguida, seguindo para `Setloc`/`SeekL`/`Setmode`/`ReadN` e carregando `SCUS_949.00`. Falta
exatamente um comando. Achado a caminho: a nossa primeira resposta sai em ZERO ciclo (ROADMAP 4.4o).

## Próxima tarefa

**ROADMAP 4.4p — primeira resposta do CD-ROM pelo `scheduler`, com o atraso da spec.**
Medido na 0120: o `sw` do comando no passo 87 464 254 e a entrada no handler `0x80000080` com
`I_STAT=0x00000004` no passo 87 464 **256** — dois passos. A spec (§ First Response, 06-cdrom.md)
da `Nop (normal) 000c4e1h  0004a73h..003115bh`: media **0xC4E1 = 50 401 ciclos**, minimo 0x4A73.
Hoje o `send_command` seta `intsts` dentro da escrita no porto, e o `service_cdrom_irq` levanta a
IRQ na mesma instrucao — a interrupcao pre-empta o proprio codigo que acabou de escrever o comando.
Isso tambem viola o R2: resposta de dispositivo e evento de `scheduler`, nao efeito colateral de
escrita.
Alvo: `crates/psx-core/src/cdrom.rs` (fila de resposta pendente com prazo) e
`crates/psx-core/src/bus.rs` (tick que entrega no prazo, ao lado do VBLANK).
Armadilha conhecida: ha 13 testes de CD-ROM que hoje leem a resposta na instrucao seguinte ao
comando; eles vao precisar avancar o relogio. NAO relaxe o teste — avance o tempo nele.
Critério de aceitação: **o `GetID` aparece depois do `Getstat`** no `cdstate.rs`. Se nao aparecer, o
atraso continua certo pela spec, mas a causa e outra: o proximo passo vira diferenciar o TTY do
kernel contra a referencia.
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
  controle e para no driver de GPU do kernel; a referencia mostra que falta o `GetID`; suspeito medido e a resposta em zero ciclo (4.4p). Imagens de disco ficam fora do repositório, em
  `.../Programacao com agentes/roms/extraido/`. **Nunca commitar imagem de disco.**
