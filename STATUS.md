# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0124** — `read_sector_from_disc` lia todo setor de `010h` (offset do Mode1); num disco
Mode2/Form1 isso devolve o sub-header como dado e desloca o setor 8 bytes. Agora o offset sai do
byte `00Fh` de cada setor. **Defeito real, mas NAO era a causa:** os 17 comandos e o ponto de
parada ficaram identicos (invariante 26, agora pelo lado negativo).

## Próxima tarefa

**ROADMAP 4.4t — o shell para depois da tela de licenca, e o bloqueio nao e mais do CD-ROM.**
Tres medicoes da 0124 dizem isso: (a) a troca do terceiro `GetID` e completa — a BIOS le o INT3,
acka, le os OITO bytes do INT2 licenciado, acka, e nao pede mais nada em 310 M passos; (b) a tela
**SONY COMPUTER ENTERTAINMENT** renderiza inteira e correta (VRAM em
`psx-estado/referencias/0124-vram-apos-licenca.png`, display 640x478, 318 278 pixels nao-zero);
(c) o TTY do kernel para em `SetGraphDebug:level:1,type:0`, sem nenhuma linha sobre `SYSTEM.CNF` —
a referencia do DuckStation ja carregou `SCUS_949.00` neste ponto.
O PC final fica num laco de contagem do shell:
`0x800422D8: addiu $t1,$t1,1` / `0x800422DC: bne $t1,$s1,0x8004205C`.
**Iteracao de diagnostico.** Instrumentar esse laco: quem sao `$t1` e `$s1`, que tabela ele varre,
e o que ele espera mudar. Ver tambem `0x800422C8`/`0x800422D0` (`beq $v0,$t4` / `beq $v0,$t5`).
Armadilha conhecida: NAO instrumente o CD-ROM de novo — as tres medicoes acima ja o eliminaram, e
foi exatamente esse erro que custou a 0119 (quatro hipoteses refutadas por olhar o subsistema
errado). O discriminador barato aqui e o TTY contra a referencia, invariante 27.
Critério de aceitação: nomear o que o laco espera, com medicao — ou mostrar `SYSTEM.CNF` sendo lido.
Invariantes relevantes: 26, 27, 28.

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

Workspace: **821** testes.

## Bloqueios

- **4.4 Boot de jogo**: sem bloqueio conhecido; o boot passa do handshake do controle, do logo
  SONY e agora pede o `GetID`, mas recebe "sem disco" e repete para sempre (4.4q). Imagens de disco
  ficam fora do repositório, em `.../Programacao com agentes/roms/extraido/`.
  **Nunca commitar imagem de disco.**
