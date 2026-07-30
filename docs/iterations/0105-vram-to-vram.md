# 0105 — vram-to-vram

- **Data:** 2026-07-30
- **Item do roadmap:** 2.2b
- **Objetivo:** implementar `GP0(80h)`, a cópia VRAM→VRAM, que até aqui era consumida e ignorada.

## Revisão do PR anterior

PR #120 (iter 0104), do próprio orquestrador: quatro checks verdes, `headRefOid` conferido, bateria
6/6.

## Spec consultada

`docs/reference/03-gpu.md`:

- **L609-615** — formato do comando: 4 palavras (comando, origem `YyyyXxxxh`, destino, `YsizXsizh`,
  com Xpos e Xsiz contados em halfwords). *"Copies data within framebuffer. The transfer is
  affected by Mask setting."*
- **L667-675** — máscaras dos parâmetros de COPY: `Xpos AND 3FFh`, `Ypos AND 1FFh`,
  `Xsiz=((Xsiz-1) AND 3FFh)+1`, `Ysiz=((Ysiz-1) AND 1FFh)+1`. *"the only special case is that
  Size=0 is handled as Size=max"*, e a fórmula é **não-monotona** para Ysiz com o bit 9 ligado
  (0x201 dá 1 linha).
- **L692-693** — *"The coordinates for the above VRAM transfer commands are absolute framebuffer
  addresses (not relative to Draw Offset, and not clipped to Draw Area)."*
- **`docs/reference/03-gpu.md` L697-700** — wrapping na borda oposta, *"without any carry-out from X to Y, nor from Y to X"*.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que o comportamento real é | Como foi pego |
|---|---|---|---|---|
| 1 | **hipótese** | Que os três defeitos visíveis do logo da BIOS (losango pela metade, "PlayStation" virando barra vermelha, sprite da Sony não composto) fossem o blit faltando. Escrevi isso no handoff da 0104 como se fosse diagnóstico | **Falso.** Implementei o blit inteiro e a VRAM saiu **byte a byte idêntica**. Histograma exato no ponto de decodificação do GP0 mostra que o boot emite **zero** comandos `80h`. O que desenha as barras são **360 quads texturizados `2Ch`** | O despejo da VRAM depois da implementação, comparado com o de antes. Foi a medida que eu deveria ter feito **antes** de escrever o handoff |
| 2 | ferramenta | Que meu histograma caseiro de comandos GP0 (lido do `sw` para 0x1F801810, pulando parâmetros por classe) fosse confiável | Ele reportou `86h x2`, isto é, dois blits. Eram palavras de **dado** lidas como comando: minha contagem de parâmetros erra em polígonos, que têm tamanho variável. A medida exata (contador no ponto de decodificação da GPU) deu **0** | Instrumentei o `execute_vram_to_vram` e contei zero execuções, contra as "2" do histograma |
| 3 | teste | Que mascarar `Xpos` e `Ypos` na entrada fossem simétricos | Mascarar `Xpos` é **redundante**: o laço já faz `(x + col) & 0x3FF` e `0x400` divide `2^16`. Para `Y` não vale — `0x200` não divide `2^16`. O mutante do X é equivalente **por construção**; o do Y é matável | Bateria em 5/7. m1 (Ysiz em 10 bits) era lacuna real do teste e virou caso novo; m3 virou `equivalente:` com justificativa |
| 4 | ferramenta | Que `justificativa:` aceitasse continuação de linha, como mostra o exemplo do formato | **Nenhum dos dois parsers aceita**: nem o `mutantes.ps1` nem o `mutation_format.rs`. E eu afirmei no meio do caminho que o de Rust aceitava — errado, ele reprova igual | O script morreu com `linha sem chave:valor nem sentinela`; rodei o meta-teste em Rust esperando confirmar a divergência e ele reprovou também. Item 10.46 |

## Bateria de mutação

Placar da bateria: 6/6 mutantes mortos, 2/2 controles verdes, 1 equivalente — docs/mutantes/0105-vram-to-vram.mut

| Registro | Rótulo | Testes que pegaram, conforme o `.resultado` |
|---|---|---|
| m1 | Ysiz mascarado em 10 bits | `ysiz_com_bit9_ligado_conta_so_os_9_bits_baixos` |
| m2 | tamanho zero vira zero em vez do máximo | `tamanho_zero_vale_o_maximo` |
| m3 | *equivalente* — Xpos sem máscara na entrada | sobreviveu, como deve |
| m4 | destino sem wrap, com carry de X para Y | `copia_envolve_na_borda_sem_carry_de_x_para_y` |
| m5 | cópia in-place, sem ler a origem antes | `origem_e_lida_antes_de_qualquer_escrita_em_regiao_sobreposta` |
| m6 | check-mask ignorado | `mask_check_protege_o_pixel_de_destino_com_bit15` |
| m7 | set-mask não força o bit15 | `set_mask_marca_bit15_no_destino` |
| c1 | `Vec::with_capacity` → `Vec::new` | sobreviveu |
| c2 | leitura dos bits de máscara reordenada | sobreviveu |

O manifesto **0100** teve a âncora `m2` envelhecida por esta iteração (ela apontava para a linha do
item 2.2b, que fechou). Reancorada no item aberto do M2 e a bateria dela **rodou de novo**: 5/5.

## Placar antes → depois

Workspace: **720** → **731** testes (+11 em `gpu_vram_to_vram`).

Efeito no boot da BIOS: **nenhum**, e isso é o resultado mais importante desta iteração. A VRAM sai
byte a byte igual porque a BIOS não emite `GP0(80h)` no boot. O comando existe, está correto e
testado; simplesmente não é o que estava quebrando a tela.

## Revisão cruzada (orquestrador)

Iteração inteira do orquestrador.

## Decisões e notas

1. **A máscara entrou junto (E6h), e não é desvio de escopo.** A frase que define o comando na spec
   diz *"The transfer is affected by Mask setting"*. Implementar o blit sem ela seria implementar
   metade. Fecha também a metade VRAM→VRAM do item 10.7.
2. **Ordem de varredura em região sobreposta é ASSUMIDA** (invariante 18): lemos toda a origem
   antes de escrever. A spec não decide, e essa é a única escolha cujo resultado não depende da
   ordem. Custa memória proporcional ao retângulo; para 1024×512 são 1 MB no pior caso, aceitável.
3. **`SkipParams` foi removido.** Era o estado que engolia os 3 parâmetros do comando classe 4 e
   ficou morto. O clippy pegou; a alternativa (deixar o variante com `#[allow(dead_code)]`) seria
   guardar o caminho antigo ao lado do novo.
4. **O item 2.2c foi criado a partir da medição**, não da intuição: 360 comandos `2Ch` no boot,
   zero `80h`. O handoff diz o primeiro passo barato (contar cores distintas na região das barras)
   e nomeia o erro que custou esta iteração, para não se repetir.
