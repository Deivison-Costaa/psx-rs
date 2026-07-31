# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0117** — o canal 2 do DMA nao olhava o bit 0 do `CHCR` (sentido): o `StoreImage` do kernel
(`CHCR=0x01000200`, device->RAM) rodava ao contrario, empurrando RAM no `GP0` e deixando a janela
do `GP0(C0h)` sem drenar. Com a GPU parada em `VramToCpu`, o `GPUSTAT.26` ficava zero e o driver
imprimia `GPU timeout`. **O `GPU timeout` sumiu e o boot passou do logo** (ROADMAP 4.4l).

## Próxima tarefa

**ROADMAP 4.4m — o shell nao arranca o jogo.**
Medido na 0117 (400 M passos, BIOS + disco do Crash): TTY limpo, sem `GPU timeout`, terminando em
`PS-X Control PAD Driver Ver 3.0`; `GPUSTAT=0x544E220A` (bit 26 alto); PC circulando em
`0x80059ED8..0x80059F0C` e `0x8003D404`; VRAM com 322 325 px nao-zero, fundo azul-escuro e a
esfera da abertura desenhada. Ou seja: o shell roda, desenha, e nao chega ao jogo.
**Meça primeiro** (invariante 26): instrumente os comandos do CD-ROM enviados depois do passo
~160 M — o shell tem de fazer `Setloc`+`ReadN` da trilha 1 para ler `SYSTEM.CNF` e depois o
executavel. O harness `psx-estado/instrumentacao/shellwait.rs` ja decodifica os portos
`0x1F801800..3`; reaproveite em vez de escrever outro. Confira TAMBEM se o `INT1` de dados
prontos chega, agora que `I_STAT.2` funciona (0114).
Spec: `docs/reference/06-cdrom.md` (§ Getloc/Setloc/ReadN) e `docs/reference/15-cdrom-format.md`
(§ ISO 9660, § SYSTEM.CNF). Arquivos-alvo: `crates/psx-core/src/cdrom.rs`.
Armadilha conhecida: o shell so olha o disco depois de montar a tela; nao confunda "nao le o
disco" com "le e nao entende o sistema de arquivos" — o harness tem de mostrar QUAL comando saiu.
Critério de aceitação: o TTY mostra o shell lendo o disco, ou a medicao prova que ele nem tenta
(e ai o item vira outro).
Invariantes relevantes: 24, 26.

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

Workspace: **782** testes.

## Bloqueios

- **4.4 Boot de jogo**: sem bloqueio conhecido; desde a 0115 o boot passa do handshake do
  controle e para no driver de GPU do kernel; a 0117 destravou o `GPU timeout` e o boot passa do logo (4.4m). Imagens de disco ficam fora do repositório, em
  `.../Programacao com agentes/roms/extraido/`. **Nunca commitar imagem de disco.**
