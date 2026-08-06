<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0203d — vblank-snapshot

- **Data:** 2026-08-06
- **Item do roadmap:** 10.42 (achado legado, "linhas trêmulas: captura sem sync com vblank")
- **Objetivo:** o framebuffer que o app desktop lê só pode mudar uma vez por vblank do PS1,
  não a cada escrita de VRAM que aconteça a qualquer instante entre dois vblanks.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Vertical Refresh Rates (L1426) | docs/reference/03-gpu.md |
| psx-spx | § GP1(05h) - Start of Display area (L809) | docs/reference/03-gpu.md |

A spec não tem uma frase literal do tipo "o framebuffer só atualiza no vblank" — isso é
inferido da física do vídeo: a taxa de atualização real do quadro é a taxa de refresh
vertical da tabela acima, e jogos que trocam de buffer via GP1(05h) fazem isso alinhados ao
vblank por convenção. Isso está documentado explicitamente nas Decisões abaixo, sem esconder
que é inferência e não citação direta.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição mostrou | Como foi pego |
|---|---|---|---|---|
| 1 | escopo | Que só o novo teste (`gpu_vblank_snapshot`) precisava passar | 6 testes pré-existentes (`gpu_framebuffer.rs` t3/t4/t5/t6, `gpu_desktop_egui.rs` d3, `gpu_display_altura_480i.rs`) escreviam em VRAM e liam `framebuffer()` sem nenhum vblank no meio — o fix os quebrou, porque eles dependiam do comportamento antigo (ler VRAM ao vivo) sem saber disso | `cargo test --workspace` |
| 2 | mutação | Que a bateria e o `mutation_anchors` de manifestos ANTIGOS (0051, 0052, e o `c2` de 0203-gpuread-latch) não seriam afetados por eu mudar o corpo de `enter_vblank`/`framebuffer` e o fim do struct `Gpu` | 3 âncoras de manifestos de outras iterações envelheceram (duas ancoravam o corpo inteiro das funções mudadas, uma ancorava o campo final do struct antes de eu inserir `display_snapshot` depois dele) — arquivadas com `arquivada:` no cabeçalho, mesmo padrão já usado nas iterações 0164/0172/0038 |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente —
`docs/mutantes/0203-vblank-snapshot.mut`.

- m1 (`enter_vblank` para de latchar o snapshot): morto.
- m2 (`framebuffer()` volta a ler `vram` ao vivo): morto.
- m3 (direção da cópia invertida): morto.
- m4 (latch movido pro `exit_vblank`): morto.
- m5 (`display_snapshot` alocado com tamanho errado, `1024*511`): morto (panic de
  `copy_from_slice` por tamanhos incompatíveis).
- c1 (ordem do campo `display_snapshot` no struct/construtor): verde.
- c2 (`copy_from_slice` trocado por `clone_from` equivalente): verde.

## Placar antes → depois

Workspace: **1249** → **1250** testes (1 novo em `gpu_vblank_snapshot.rs`).

## Revisão cruzada (orquestrador)

Sem achados — esta iteração foi conduzida pelo próprio orquestrador (exceção vigente em
`docs/orquestracao.md`; ver STATUS.md).

## Decisões e notas

**1. Por que latchar em `enter_vblank()` e não criar um evento novo no scheduler.**
`enter_vblank()` já é chamado exatamente uma vez por vblank real, a partir do evento
`VBLANK_ENTER` agendado em `bus.rs`. Colocar o latch ali reaproveita um hook que R2 já exige
que exista — não precisei tocar no scheduler nem no bus.

**2. Por que isso não é uma prova rigorosa de spec, e por que tá tudo bem.** A spec descreve
timings de vídeo (refresh rate) e o protocolo de troca de buffer via GP1(05h), mas não
descreve como um EMULADOR deveria amostrar VRAM pra apresentar a imagem — essa é uma decisão
de arquitetura de emulador, não de hardware real (hardware real escaneia VRAM continuamente
durante o display ativo; um jogo sem double-buffer pode genuinamente mostrar rasgo na tela
real). A aproximação "latch uma vez por vblank" é a técnica padrão usada por praticamente todo
emulador de PS1 com rasterizador por software, e resolve o sintoma medido (triângulos
aparecendo/sumindo de forma inconsistente entre frames de host) sem fingir que existe uma
citação de spec que não existe.

**3. `display_start_x`/`display_start_y` continuam lidos ao vivo, não fazem parte do
snapshot.** GP1(05h) já é aplicado imediatamente hoje; sincronizar ISSO com vblank também
seria uma correção legítima (jogos que trocam de buffer relying on isso), mas é um achado
separado — não ampliei o escopo (R4).

**4. Custo de memória.** `display_snapshot` dobra o estado salvo pela GPU (mais 1 MB por save
state) e copia 1024×512 halfwords a cada vblank (~60×/s de tempo do PS1). `TAMANHO_DO_ESTADO`
em `snapshot_estado.rs` atualizado de 4.858.072 bytes (era 3.809.488 antes desta iteração).
