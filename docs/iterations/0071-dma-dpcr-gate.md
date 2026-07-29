# 0071 — dma-dpcr-gate

- **Data:** 2026-07-29
- **Item do roadmap:** 10.19
- **Objetivo:** adicionar consulta ao DPCR como gate de habilitação nos três `try_execute_*` do DMA.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | 1F8010F0h - DPCR - DMA Control Register (R/W) (L121) | docs/reference/04-dma.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | Que o teste de DMA3 com CDROM desabilitado bastava com `insert_disc()` | `drqsts_active()` exige `read_mode != 0`, `data_pos < 2048` e `hchpctl & 0x80`, o que requer a sequência completa Setloc→ReadN→HCLRCTL→HCHPCTL | O teste `dma3_transfere_com_canal_habilitado_no_dpcr` falhou na primeira rodada porque o CDROM não estava pronto; o teste desabilitado passava mas pelo motivo errado (CDROM não pronto, não pelo gate) |
| 2 | processo | Que `@@PARA` vazio era aceito no manifesto para deletar linhas | O meta-teste `mutation_anchors` exige `@@PARA` não-vazio | `manifesto_de_mutacao_ancoras_sao_reais` reprovou m1, m2 e m3 com "edicao 1: @@PARA vazio". Substituí por `let _ = self.dpcr & (1 << N);` que é um no-op equivalente |
| 3 | processo | Que a chave `mutante: c1` bastava para marcar um controle | A chave do manifesto é `controle:`, não o prefixo do ID | O placar da primeira bateria deu `0/0 controles verdes` com c1 e c2 sobrevivendo como "mutantes". Corrigi as chaves para `controle:` |

## Bateria de mutação

Placar da bateria: 7/7 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0071-dma-dpcr-gate.mut

| Registro | Tipo | Rótulo | Teste que pegou |
|---|---|---|---|
| m1 | mutante | OTC sem gate DPCR | `otc_nao_transfere_com_canal_desabilitado_no_dpcr` |
| m2 | mutante | DMA3 sem gate DPCR | `dma3_nao_transfere_com_canal_desabilitado_no_dpcr` |
| m3 | mutante | DMA2 sem gate DPCR | `dma2_nao_transfere_com_canal_desabilitado_no_dpcr` |
| m4 | mutante | OTC usa bit 26 em vez de 27 | `otc_nao_transfere_com_canal_desabilitado_no_dpcr` |
| m5 | mutante | DMA2 usa bit 10 em vez de 11 | `dma2_transfere_com_canal_habilitado_no_dpcr` |
| m6 | mutante | DMA3 condição invertida | `dma3_nao_transfere_com_canal_desabilitado_no_dpcr` e `dma3_transfere_com_canal_habilitado_no_dpcr` |
| m7 | mutante | DMA3 usa bit 14 em vez de 15 | `dma3_nao_transfere_com_canal_desabilitado_no_dpcr` |
| c1 | controle | `let _` no-op antes do for em try_execute_otc | nenhum (sobreviveu) |
| c2 | controle | formatação numérica 0x0765_4321 → 0x07654321 | nenhum (sobreviveu) |

## Placar antes → depois

Workspace: **549** → **556** testes (+7: `dma_dpcr_gate`).
Testes existentes que precisaram de correção: todos os testes de `dma_otc.rs` (7), `dma_gpu.rs` (11) e `cdrom_dma.rs` (6) que disparam transferência DMA passaram a habilitar o canal no DPCR antes de escrever CHCR.

## Revisão cruzada (orquestrador)

Revisada e mergeada pelo PR #85. A implementação está certa e resolveu o que prometia; os achados
abaixo são de registro e de teste, não de emulação.

**Medição de hardware, que é o que justifica o item.** `ps1-tests/dma/otc-test` no `8d4267e`:
`6p/34f` → **`7p/30f`**, e `testOtcStandardWithMasterDisabled` passou. Os quatro subtestes que a
iteração 0068 previu, exatamente, sem regressão em nenhum outro. É a primeira correção do projeto
guiada por suíte de hardware em vez de teste próprio.

**A1 — a tabela de mutação deste doc não batia com o `.resultado` em 3 das 9 linhas.** O doc
creditava m4 a `otc_transfere_com_canal_habilitado_no_dpcr` e m7 a
`dma3_transfere_com_canal_habilitado_no_dpcr`; a máquina registrou os dois como mortos por
`..._nao_transfere_com_canal_desabilitado_no_dpcr`, e m6 por dois testes, não um. Corrigido acima
para o que a máquina diz. A linha `Placar da bateria:` estava certa — ela é conferida por
meta-teste; a tabela por registro não é, e é justamente onde a iteração 0038 inflou dois créditos
por inspeção. Virou o item 10.28.

**A2 — `assert_ne!` numa correção de defeito.** `dma_dpcr_gate.rs:141` afirma
`assert_ne!(val, 0xDEAD_BEEF)`, isto é, "o valor mudou", em vez do valor que a transferência
deveria produzir. É o padrão 1 da lista de revisão, na rodada cujo passo A era conferir aquela
lista. A bateria não pegou porque m6 morre também pelo teste do lado desabilitado. Não bloqueia o
merge — o gate está provado pelos outros seis testes —, mas o teste precisa afirmar o valor.
Virou o item 10.29.

**A3 — habilitar o canal no DPCR não dispara transferência pendente.** O modelo só dispara na
escrita de CHCR. A sequência real "escreve CHCR com o canal desabilitado, depois habilita no
DPCR" deveria iniciar a transferência no hardware e não inicia aqui. Nenhum subteste do `otc-test`
cobre isso hoje, e por isso não custou nada agora. Virou o item 10.30.

**Conferidos e sem achado:** parâmetro não consumido (nenhum comando novo), regra de borda
(nenhuma rasterização), campo de bit lido errado (bits 27/15/11 conferidos contra `04-dma.md`
L136/L130/L128), panic ou laço ilimitado (só três `return` novos), citação de spec (a única do doc
é a L121 do DPCR, correta), escopo transbordado (nada além do gate), portão que não mede (a
bateria mata 7 de 7 com controles verdes e foi rodada pela máquina), manifesto arquivado
(nenhum).

## Decisões e notas

1. **O gate é a PRIMEIRA verificação em cada `try_execute_*`.** A ordem importa: se o DPCR estiver desabilitado, o código retorna sem tocar em CHCR, RAM, CDROM ou GPU. Isso evita que um canal desabilitado consuma o estado de outro componente.
2. **O reset `07654321h` desabilita todos os canais.** O bit 3 de cada nibble (Master Enable) é zero para DMA0–DMA6 no reset. Isso significa que nenhum canal executa sem que o software escreva explicitamente no DPCR — o comportamento que o `otc-test` do ps1-tests espera.
3. **Os testes existentes já provavam que os canais funcionavam quando habilitados.** A mudança nos testes existentes foi puramente mecânica: adicionar `bus.write32::<BusRead>(DPCR, 0x0765_4321 | (1 << N))` antes de cada CHCR write. Nenhum teste mudou de semântica.
4. **A preparação do CDROM para DMA3 foi replicada no teste novo.** Para evitar dependência entre arquivos de teste, as funções `cd_write`, `cd_read`, `set_bank` e `preparar_cdrom_para_dma3` foram copiadas inline em `dma_dpcr_gate.rs`. O protocolo do projeto prefere duplicação controlada em testes a acoplamento entre arquivos de teste.
