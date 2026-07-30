# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. **Só handoff:** o que fazer agora e o que a máquina precisa saber para
> julgar uma rodada. Referência estável mora em `docs/invariantes.md` e é citada por número.
> Teto de 6 KB imposto por `status_size.rs`; forma imposta por `status_handoff.rs`.

## Última iteração concluída

**0105** — GP0(80h), copia VRAM->VRAM, com mascara, wrap e coordenadas absolutas (ROADMAP 2.2b).

## Próxima tarefa

**ROADMAP 2.2c — o quad texturizado GP0(2Ch) sai como barra chapada.**
Medido pelo orquestrador em 30/07 com histograma exato no ponto de decodificacao do GP0, BIOS real
+ disco, 400M passos. O boot emite **360 comandos 2Ch** (quad texturizado, opaco, modulado) e
**nenhum 80h**. Sao os 2Ch que desenham a palavra "PlayStation" no logo, e hoje ela sai como duas
barras vermelhas horizontais chapadas. O sprite `SONY COMPUTER ENTERTAINMENT` esta carregado na
VRAM (canto superior direito) pelos 63 comandos A0h, e tambem nao aparece composto.
Spec: `docs/reference/03-gpu.md`, secoes de Polygon e de Texpage/CLUT (offset +115 sobre o indice).
Arquivos-alvo: `crates/psx-core/src/gpu.rs`.
Critério de aceitação: as barras viram texto legivel no despejo da VRAM.
Invariantes relevantes: 13, 18, 19.

**Primeiro passo, barato:** conte as cores distintas na regiao das barras. Cor unica = a textura
nao esta sendo amostrada; varias = amostra mas a modulacao ou a UV estao erradas. Os itens 10.13
(modulacao vs raw texture) e 10.11 (textura de retangulo) sao vizinhos e ainda NAO foram medidos
contra este caso.

**Erro que ja custou uma iteracao — nao repetir:** o handoff da 0104 afirmava que os tres defeitos
visiveis do logo eram culpa do blit VRAM->VRAM faltando. Era falso. Implementei o blit inteiro e a
VRAM saiu **byte a byte identica**. Antes de atribuir defeito visual a um comando, MEÇA se o
comando e sequer emitido.

## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- **Escopo de commit é UM único identificador `[a-z0-9-]`.** `feat(bus,cpu)` reprova no
  `commit-lint`; quando a mudança toca dois módulos, escolha o principal e cite o outro no
  resumo. Custou uma reescrita de 4 mensagens no PR #36 (ver `0022-scratchpad-isc.md`).
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.

## Placar de testes

Workspace: **731** testes.

## Bloqueios

- **4.4 Boot de jogo**: DESBLOQUEADO em 30/07 — o usuário forneceu as imagens. Ficam fora do
  repositório, em `C:\psx-roms\` (extraídas dos zips em `.../roms`). **Nunca commitar imagem de
  disco.** Depende agora do 2.2b.
