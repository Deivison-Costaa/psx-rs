# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0146** — a sondagem da tabela EvCB em `testevent_descritor` rodava a cada passo da CPU (~66
leituras de barramento contra 1-2 do `cpu.step`), e o evento so aparece no passo 86.988.128:
eram 87 M de varreduras sem nada para achar. Passa a amostrar a cada 10 k passos, teto
justificado por medicao (a condicao persiste >= 900 k passos). Teste: 100 s -> 12,7 s.

## Próxima tarefa

**ROADMAP 4.5 — rastrear quem escreve `BFC06FDC` em `mem[$v1+0x18]` entre o primeiro e o
segundo boot.**

Ja provado: (i) o trampolim `0x2C94..0x2DB8` carrega `$t0` de `mem[$v1+0x18]` e faz
`jalr $ra, $t0`; (ii) no primeiro boot o slot aponta para funcoes normais do kernel; (iii) aos
354 M contem `BFC06FDC`, que leva ao `SysInitMemory`, que reinicializa `A000E000h`+`2000h` —
onde mora o array de ExCB.

Medir: sonda de escrita (`sw`) com endereco-alvo `$v1+0x18` pos-primeiro-boot, registrando PC e
valor escrito.

Armadilhas: (a) `$v1` e carregado antes do trampolim, entao o endereco-alvo depende do valor em
runtime — ler `$v1` no STEP em `0x2DAC` e usar o valor dinamico; (b) reconstruir release antes
de medir; (c) sondas sao descartaveis, reverter antes de commitar.

Invariantes relevantes: 25, 27, 30, 31.

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

Workspace: **879** testes.

## Bloqueios

- **4.4 Boot de jogo**: o motor 4.4ad agora avança setores sequencialmente; a fronteira
  seguinte medida no Crash é VSync/IRQ0 pós-kernel. Imagens de disco ficam fora do
  repositório, em `.../Programacao com agentes/roms/extraido/`.
  **Nunca commitar imagem de disco.**
