<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0117 — dma-gpu-vram-para-ram

- **Data:** 2026-07-30
- **Item do roadmap:** 4.4l
- **Objetivo:** fazer o canal 2 do DMA transferir no sentido dispositivo→RAM, drenando a janela
  pedida por `GP0(C0h)` para que o `GPUSTAT.26` volte a subir.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § DMA Channel Control | docs/reference/04-dma.md |
| psx-spx | § GPU Status Register | docs/reference/03-gpu.md |

Uma linha resolve o item, e é a primeira do registrador: *"0 Transfer direction (0=device to RAM,
1=RAM to device)"*. O `execute_block` lia esse bit para **nada** — não existia um caminho
device→RAM no emulador inteiro.

## Como o item foi encontrado (medição antes de código)

O handoff da 0116 dizia: `GPUSTAT.26` preso em zero, hipótese *não confirmada* de que um comando
GP0 tivesse ficado faminto por parâmetros no linked-list, e a ordem explícita de medir antes de
consertar. A medição derrubou a hipótese e achou outra coisa. Harness `gp0stuck`, em três passadas:

1. **Não é o linked-list.** Em 511 transferências disparadas, **zero** quedas do bit 26 dentro de
   uma transferência de DMA. A hipótese do handoff estava errada.
2. **A queda vem de escrita direta da CPU no GP0.** Guardando as últimas 24 escritas em
   `0x1F801810` a cada transição 1→0 e ficando com a ÚLTIMA (a que não volta): passo 157 609 882,
   `pc=0x8005097C`, `GP0=0xC0000000` — um **VRAM→CPU**, precedido de `01h` (clear cache). Depois
   dele o bit 26 nunca mais sobe em 22 M passos.
3. **Por que ninguém drenou.** Classificando todo `D2_CHCR` disparado por valor:

```
    CHCR=0x01000401  direcao=da RAM   sync=2 (linked-list)  x503
    CHCR=0x01000201  direcao=da RAM   sync=1 (slice)        x6
    CHCR=0x01000200  direcao=para RAM sync=1 (slice)        x2   <-- o StoreImage
```

O kernel dispara o dreno por DMA no sentido device→RAM, e `try_execute_dma2` nunca olhou o bit 0:
mandava `execute_block`, que lê RAM e empurra no `GP0`. Ou seja, o `StoreImage` **rodava ao
contrário** — despejava lixo da RAM no fluxo de comandos da GPU e deixava a janela do `C0h`
intocada. Com a GPU parada em `VramToCpu`, `GPUSTAT.26` fica zero para sempre e o driver desiste
com `GPU timeout`.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | **diagnóstico** | Que o `GPUSTAT.26` preso vinha de um comando entregue pela metade pelo linked-list — foi o que escrevi no handoff da 0116. | § DMA Channel Control: o bit 0 do `CHCR` é o sentido, e o kernel usa `0x01000200` (device→RAM) para o `StoreImage`. | O próprio handoff mandava medir antes de consertar (invariante 26, escrita na iteração anterior). A primeira passada do `gp0stuck` deu **zero** quedas dentro de transferência e matou a hipótese em 4 minutos, antes de qualquer linha de código. |
| 2 | endereçamento | Que um DMA no sentido errado simplesmente não faria nada. | — | Cinco dos sete testes novos passaram **antes** do fix: `MADR` avançava, o bit 24 caía, o gate do `DPCR` valia. O transfer errado rodava inteiro e com aparência de sucesso — só que lendo da ponta errada. Um canal sem noção de sentido não falha, ele corrompe. |

## Bateria de mutação

Placar da bateria: 6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0117-dma-gpu-vram-para-ram.mut

| # | Mutação | Teste que pegou |
|---|---|---|
| m1 | sentido ignorado, tudo vira RAM→dispositivo (o defeito original) | `dma2_para_ram_drena_a_janela_pedida_pelo_c0h` |
| m2 | sentido invertido | `dma2_da_ram_continua_empurrando_para_o_gp0` |
| m3 | dreno grava zero em vez do que a GPU entrega | `dma2_para_ram_drena_a_janela_pedida_pelo_c0h` |
| m4 | palavra drenada gravada em big-endian | `dma2_para_ram_drena_a_janela_pedida_pelo_c0h` |
| m5 | dreno lê o `GPUSTAT` (offset 4) em vez do `GPUREAD` (offset 0) | `dma2_para_ram_devolve_o_gpustat_26_ao_terminar` |
| m6 | gate do `DPCR` do canal 2 deixa de valer | `dma2_para_ram_nao_roda_com_o_canal_desabilitado_no_dpcr` |
| c1 | bit de sentido testado por igualdade (cosmético) | verde |
| c2 | tipo da palavra drenada anotado (cosmético) | verde |

Erro de manifesto na primeira corrida: o m3 trocava `gpu.read32(0)` por `0`, e o `to_le_bytes()`
seguinte não compilava (`ambiguous numeric type`). Corrigido para `0u32`. A corrida abortada
deixou a sentinela `logs/mutantes-em-andamento.txt`; a árvore estava limpa (o m3 nem chegou a ser
aplicado), então bastou remover a sentinela.

## Placar antes → depois

Workspace: 775 → **782** testes (7 novos em `dma_gpu_vram_para_ram.rs`), 0 falhas.

Efeito medido no boot real (SCPH1001 + disco do Crash, 400 M passos):

| | antes | depois |
|---|---|---|
| TTY | `GPU timeout:QUE=(n,n)` repetido, 891 bytes | **sem nenhum `GPU timeout`**, 539 bytes |
| `GPUSTAT` final | `0x184E260A` (bit 26 = 0) | `0x544E220A` (bit 26 = 1) |
| PC ao fim | preso no driver de GPU (`0x80051200..`) | laço do shell (`0x80059ED8..0x80059F0C`) e `0x8003D404` |
| VRAM não-zero | 315 767 px | 322 325 px |
| tela | logo da SONY | **passou do logo**: fundo azul-escuro e a esfera da abertura na VRAM |

**Critério de aceitação do item: cumprido.** O `GPU timeout` sumiu e o PC saiu do laço do driver.

## Revisão cruzada (orquestrador)

Sem achados que barrem o merge.

- **A correção é uma bifurcação de 4 linhas dentro do laço que já existia.** O sentido é lido uma
  vez, fora do laço; o passo do `MADR`, o gate do `DPCR`, o `BCR` e a baixa do bit 24 são os
  mesmos nos dois sentidos — nada foi duplicado.
- **O dreno usa a porta pública da GPU.** `gpu.read32(0)` é o mesmo `GPUREAD` que a CPU lê, com o
  mesmo efeito colateral de consumir a fila: quando `remaining` chega a zero, o `gpu.rs` já
  limpava o bit 27, subia o bit 26 e voltava para `Idle`. Não foi preciso mexer em `gpu.rs`.
- **Só o modo bloco (sync=1) ganhou sentido.** É o único que o kernel usa para `StoreImage`
  (medido: `CHCR=0x01000200`). Burst (sync=0) continua não implementado e linked-list é
  RAM→dispositivo por definição. Não generalizei (R4).
- **Buraco conhecido, deixado aberto (R4).** `GP0(C0h)` sem transferência pendente devolve zero em
  vez de espelhar o comportamento real do `GPUREAD`; e um dreno maior que a janela lê zeros em vez
  de repetir o último dado. Nenhum dos dois aparece no caminho medido. Anotado como 10.50.
- **Gates do projeto:** `purity`, `file_size`, `comment_density`, `roadmap_size`, `status_size`,
  `spec_citations`, `mutation_manifest`, `mutation_anchors` e `mutation_battery` verdes.

## Decisões e notas

- **A invariante 26 pagou na iteração seguinte à que a criou.** Ela nasceu na 0116 justamente
  porque eu tinha tratado um defeito real como causa. Aqui o handoff carregava outra hipótese
  minha, com a ordem de medir antes; a medição a refutou na primeira passada. O custo de medir
  foram três execuções do harness; o custo de não medir teria sido reescrever o parser de GP0.
- **Três iterações, três buracos de fiação, um sintoma cada vez menor.** 4.4j (byte alto do SIO0),
  4.4k (IRQ3 do DMA) e 4.4l (sentido do canal 2) são todos "o dado existe, o caminho não". O
  padrão da invariante 24 continua rendendo, com a ressalva da 26.
- **Próximo degrau.** O boot passou do logo e está no laço do shell com a esfera da abertura já
  desenhada na VRAM. O jogo ainda não arranca: falta medir se o shell chega a ler o `SYSTEM.CNF`
  do disco. É o item 4.4m.
