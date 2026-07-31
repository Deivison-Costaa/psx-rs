# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0123** — `GetTOC` (1Eh) caia no braco default e nunca armava a segunda resposta. Agora faz
INT3(stat) + INT2(stat), reusando o caso 1 do `deliver_second`. **O boot passou a ler o disco:**
4 comandos viraram 17, com a cadeia `Setloc/SeekL/Setmode/ReadN/Pause` da referencia e INT1 em
27 924 passos.

## Próxima tarefa

**ROADMAP 4.4s — setor Mode2/Form1 lido a partir do offset do Mode1.**
Medido na 0123, despejando o `.bin`: o byte `00Fh` do setor 4 e `02h` — **Mode2/Form1**. A spec
(§ Mode2/Form1 (CD-XA), `docs/reference/15-cdrom-format.md` L621) da para esse formato
`010h 4 Sub-Header`, `014h 4 Copy of Sub-Header` e os dados so em **`018h`**; o Mode1 (L613) e que
comeca em `010h`. O `read_sector_from_disc` usa `abs_sector*2352 + 0x10` fixo, entao **todo setor
sai 8 bytes deslocado**, com o sub-header na frente. Prova no setor 16 (PVD do ISO9660): de `010h`
sai `00 00 09 00 00 00 09 00 01 'CD001'` em vez de `01 'CD001'`.
Sintoma: a BIOS le os setores de licenca (LBA 4 e 5, `Setloc 00:02:04`/`00:02:05`), reemite `GetID`
e para. E a verificacao de licenca falhando com dados deslocados.
Alvo: `crates/psx-core/src/cdrom.rs`, `read_sector_from_disc` — escolher o offset pelo byte de modo
do proprio setor (`00Fh`), nao por constante.
Armadilha conhecida: o disco de stub dos testes (`insert_disc` sem `.bin`) preenche `i+1` e nao tem
header nenhum; os 11 testes de `cdrom_read.rs`/`cdrom_dma.rs` que dependem dele NAO podem quebrar.
E ha `read_n_retorna_dados_do_bin_no_setor_correto` em `cdrom_read.rs`, que monta um `.bin`
sintetico — confira em que modo ele monta antes de mudar o offset.
Critério de aceitação: o `cdstate.rs` mostra comando novo depois do terceiro `GetID`.
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

Workspace: **814** testes.

## Bloqueios

- **4.4 Boot de jogo**: sem bloqueio conhecido; o boot passa do handshake do controle, do logo
  SONY e agora pede o `GetID`, mas recebe "sem disco" e repete para sempre (4.4q). Imagens de disco
  ficam fora do repositório, em `.../Programacao com agentes/roms/extraido/`.
  **Nunca commitar imagem de disco.**
