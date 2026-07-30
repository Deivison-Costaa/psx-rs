# 0088 — gte-nclip-avsz3-avsz4-sqr-op

- **Data:** 2026-07-30
- **Item do roadmap:** 5.3
- **Objetivo:** Implementar comandos GTE NCLIP, AVSZ3, AVSZ4, SQR e OP.

## Revisão do PR anterior

Revisão do PR #102 (iter 0087): **sem achados**.

Nove padrões conferidos:
1. Teste que não mede — todos os 11 testes têm asserções com valores golden da spec; nenhum round-trip ou assert_ne como única asserção
2. Parâmetro não consumido — sem novos comandos GPU; GTE não tem parâmetros de comando que afetem FIFO
3. Regra de borda trocada — N/A (GTE, não GPU)
4. Campo de bit lido errado — cmd extraído de func & 0x3F; sf de bit 19; lm de bit 10; nenhum shift ou máscara nova
5. Panic ou laço ilimitado — sem unwrap/expect/unsafe; loops ausentes nos novos comandos
6. Citação de spec — `confere-citacoes.ps1` verde
7. Escopo transbordado — 5 comandos GTE conforme item 5.3 do ROADMAP; sem funcionalidade extra
8. Portão — manifesto novo, `.resultado` rastreado, âncoras reparadas durante a bateria
9. Manifesto arquivado — sem arquivamentos

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | GTE Command Encoding (L117) | `docs/reference/07-gte.md` |
| psx-spx | NCLIP (L513) | `docs/reference/07-gte.md` |
| psx-spx | AVSZ3 (L523) | `docs/reference/07-gte.md` |
| psx-spx | AVSZ4 (L524) | `docs/reference/07-gte.md` |
| psx-spx | SQR (L566) | `docs/reference/07-gte.md` |
| psx-spx | OP (L574) | `docs/reference/07-gte.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | Cálculo manual | AVSZ3 OTZ = 0x68*600/4096 = 9 | 0x68 = 104 decimal; 104*600/4096 = 15 | teste `avsz3_media_ponderada_de_tres_z` falhou |
| 2 | Cálculo manual | AVSZ4 OTZ = 0x40*1000/4096 = 9 | 0x40 = 64; 64*1000/4096 = 15 | teste `avsz4_media_ponderada_de_quatro_z` falhou |
| 3 | Ordem dos termos | OP MAC1 = IR2*D3 - IR3*D2 = 22-21 = 1 | A fórmula correta é IR3*D2 - IR2*D3 = 21-22 = -1 | teste `op_produto_vetorial_sf0_lm0` falhou |
| 4 | Layout de registrador | D1 = regs[32] as i16 com valor 0x0010_0000 dá 0, não 0x0010 | RT11 está nos 16 bits inferiores; escrever 0x0010_0000 põe 0x0010 nos 16 superiores | teste `op_sf1_desloca_12_bits` falhou |
| 5 | Âncoras multi-linha | DE multi-linha funcionaria no mutantes.ps1 | Script usa Get-Content -Raw que preserva LF; manifest escrito com LF mas arquivos-fonte com LF também; o erro real foi ocorrências não documentadas | bateria falhou em m2, m4, m7 |

## Bateria de mutação

Placar da bateria: 7/7 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0088-gte-nclip-avsz3-avsz4-sqr-op.mut

| # | Tipo | Rótulo | Resultado |
|---|---|---|---|
| m1 | mutante | NCLIP inverte sinal de MAC0 | MORREU |
| m2 | mutante | SZ1 usa regs[16] em vez de regs[17] (AVSZ3 e AVSZ4) | MORREU |
| m3 | mutante | AVSZ4 usa ZSF3 em vez de ZSF4 | MORREU |
| m4 | mutante | SQR/OP/RTPS usam sf=0 sempre (ignora bit sf) | MORREU |
| m5 | mutante | OP troca ordem do produto vetorial | MORREU |
| m6 | mutante | NCLIP troca sinal de termo (sx1*sy2 negativo) | MORREU |
| m7 | mutante | OTZ nao satura em 0xFFFF (AVSZ3 e AVSZ4) | MORREU |
| c1 | controle | variavel local renomeada em nclip | verde |
| c2 | controle | adiciona let _ = 0 no inicio de sqr | verde |

## Placar antes → depois

Workspace: **651** → **662** testes (+11: gte_nclip_avsz3_avsz4_sqr_op).

## Decisões e notas

1. **FLAG zerado no início de cada comando.** `execute_command` agora aplica `self.regs[63] &= 0x7FFF_F000` antes do match, conforme spec L373-375. Testes existentes (RTPS/RTPT) continuam passando porque já zeravam o flag internamente.

2. **NCLIP overflow positivo nunca ocorre com entradas de 16 bits.** O produto máximo de dois S16 é ~10^9, e a fórmula de NCLIP cancela termos de forma que o resultado sempre cabe em 32 bits. O bit 16 do FLAG existe na spec mas é inalcançável via software normal.

3. **lm do SQR é irrelevante.** A spec diz que "lm flag for negative saturation has no effect" porque o quadrado é sempre positivo. lm=0 e lm=1 produzem resultados idênticos.
