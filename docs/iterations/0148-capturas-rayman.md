# 0148 — capturas-rayman

- **Data:** 2026-08-01
- **Item do roadmap:** 10.72
- **Objetivo:** guardar prova datada do que o emulador desenha e registrar o que um segundo
  disco revelou. Até aqui o projeto media boot só com o Crash Bandicoot, e nenhuma imagem era
  versionada.

## Spec consultada

Nenhuma seção de hardware. O formato do dump está em `crates/psx-cli/src/main.rs`
(`write_vram_dump`): VRAM inteira, 1024×512, 16 bits por pixel, RGB555 little-endian.

Bateria de mutação: não se aplica — nenhuma linha de código de produção mudou, só documentação
e capturas de execução.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição mostrou | Como foi pego |
|---|---|---|---|---|
| 1 | protocolo | Que capturas eram documentação avulsa e podiam entrar por um PR fora do ciclo de iteração, intitulado `docs: ...` | **Não existe PR avulso neste projeto.** A CI valida TODO título contra `iter NNNN: resumo (ROADMAP X.Y)`; o PR #165 reprovou no `commit-lint` antes de qualquer revisão humana | `commit-lint` do PR #165 |
| 2 | handoff | Que registrar 10.72 e 10.73 numa branch bastava para o trabalhador poder trabalhar no 10.73 | As linhas ficaram **só na branch não mergeada**: a `main` não tinha 10.73, e a rodada foi disparada apontando para um item que não existia. É a falha de "item fantasma" que o `status_handoff.rs` documenta ter custado os PRs #114 e #115 | Conferência de `git show main:ROADMAP.md` depois de a CI reprovar |
| 3 | numeração | Que a numeração da iteração podia ser decidida depois | O trabalhador escolhe o número sozinho a partir do STATUS e já havia criado `iter/0148-vsync-timeout`. Duas iterações reivindicando 0148 ao mesmo tempo | `git branch` durante o conserto |

O erro 2 é o grave. Os outros dois são de forma; esse invalidava a rodada.

## Medição

Todas as capturas no commit `760eb75` (merge da 0146), com:

```sh
psx-cli --bios bios/SCPH1001.BIN --disc <CUE> --max-steps <N> --dump-vram <saida>.raw
```

### Os dois discos divergem, e é isso que interessa

| | Crash Bandicoot | Rayman |
|---|---|---|
| framebuffer aos 400 M passos | **vazio** | **tela da Ubi Soft desenhada** |
| VRAM aos 400 M | texturas carregadas no canto inferior direito | logo com degradê de arco-íris |
| TTY | 1513 bytes | 8900 bytes |
| sintoma | congela antes de desenhar | desenha e trava em `VSync: timeout` |

A tela da Ubi Soft **não é da BIOS**: é código do próprio jogo. Ela exercita CPU, CD-ROM, DMA e
rasterizador juntos, num caminho que o Crash nunca alcança.

No Crash, o padrão colorido da VRAM que parece ruído **é** dado real: textura indexada por CLUT
(4 ou 8 bits) exibida como cor direta de 16 bits aparece assim num dump cru.

### O Rayman está travado, não lento

VRAM aos 1.500 M passos **byte a byte idêntica** à de 400 M (mesmo SHA-256), e o TTY parou de
crescer em 8900 bytes. Quase 4× mais passos, zero progresso.

## Bateria de mutação

Bateria de mutação: não se aplica — nenhuma linha de código de produção mudou, só documentação
e capturas de execução.

## Placar antes → depois

Workspace: **882** testes (inalterado — nenhum teste criado ou removido).

## Revisão cruzada (orquestrador)

Iteração do orquestrador (`fonte=orquestrador`), nascida de um erro de processo do próprio
orquestrador — ver erro 2.

O que merecia ceticismo: **capturas são prova ou enfeite?** Só são prova com procedência. Por
isso o `docs/capturas/README.md` registra comando, commit e formato do dump; sem isso, uma
imagem bonita não sustenta afirmação nenhuma.

Segundo ponto: **nenhuma BIOS e nenhuma imagem de disco é versionada.** As capturas são saída do
nosso emulador, em resolução de VRAM, guardadas para documentar progresso de emulação.

## Decisões e notas

**1. O Rayman só bootou com um `.cue` reduzido.** O original tem 51 tracks (uma de dados, 50 de
áudio CD-DA) e o nosso `parse_cue` guarda um único `bin_path`: cada linha `FILE` sobrescreve a
anterior, então sobraria a track 51, que é áudio. Montei um `.cue` só com a track 01. Registrado
como **10.72** — é a dívida que esta iteração fecha por registro, não por conserto.

**2. `VSync` não é função do kernel.** Não está em nenhuma tabela A/B/C nem em
`docs/reference/13-kernel-bios.md`. É da LIBGPU, que o jogo linka estaticamente — logo a mensagem
vem do código do próprio Rayman, e o timeout é um contador dele. Registrado como **10.73**, e é
o que a próxima iteração investiga.

**3. Três linhas do ROADMAP foram comprimidas** (10.13, 10.63, 10.66) para caber sob o teto de
7 KB. Encurtadas, nunca apagadas — o contexto mora nos docs de iteração.
