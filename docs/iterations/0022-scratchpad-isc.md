<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0022 — scratchpad-isc (2ª rodada)

- **Data:** 2026-07-28
- **Item do roadmap:** 1.9 (revisão adversarial do PR #36)
- **Objetivo:** corrigir quatro achados da revisão (F1-F3, F5) sem alterar o escopo original.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | Exception Priority, cop0r12 - SR, bit 16 Isc | docs/reference/02-cpu.md |
| psx-spx | Memory Control ports, RAM_SIZE, BCC | docs/reference/12-memory-control.md |

## Os seis testes D1-D6 passavam — e ainda assim havia três lacunas

As três lacunas (F1, F2, F3) estavam todas **fora do que o handoff da iteração pedia**. O handoff
especificou: scratchpad 1KB (faixa e KSEG1), memory control stubs (0x1F801000..0x1F801023 +
0x1F801060), BCC (0xFFFE0130), e Isc suprimindo stores. D1-D6 testam exatamente isso — e
passam. As lacunas estão no que **não foi pedido**: sub-word reads dos mesmos registradores
(F1), o resto da janela de I/O de 4 KB (F2), e a ordenação entre Isc e AdES (F3). O mesmo
eixo do erro 1.8b (handoff subdimensionado), agora registrado como erro do orquestrador.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | Teste frágil | D3 (limite superior) sem testemunha RAM bastava | Um alias que passa o readback (escrita/leitura vão ao mesmo alias) não prova que o destino é scratchpad | Mutação de range (0x3FF→0x3FB) sobreviveu; D3 foi reforçado com testemunha RAM |
| 2 | Cobertura/escopo | region_read_byte só precisa tratar scratchpad; sub-word reads de memctrl/BCC caem em RAM | read8/read16 em 0x1F80_1060 ou 0xFFFE_0130 devem devolver o byte do registrador, não lixo de RAM | Revisão adversarial do PR #36: sonda mostrou 0xEE em vez de 0x88 |
| 3 | Cobertura/escopo | A janela de I/O 0x1F801000..0x1F801FFF está protegida pelos stubs de memctrl | Só 0x28 bytes estão cobertos (0x1000..0x1023 + 0x1060); I_STAT, DMA, timers, GPU, CDROM, SPU aliasam RAM | Revisão adversarial: write32(0x1F80_1074, 0xFFF) apareceu em read32(0x0000_1074) |
| 4 | Ordem-de-exceção | Isc=1 early-return antes do cálculo do endereço nos stores | Exception Priority põe AdES acima de qualquer decisão de isolamento; Isc decide para onde, não se o endereço é válido | Revisão adversarial: sw desalinhado com Isc=1 não dispara AdES (CAUSE=0 em vez de 0x14) |
| 5 | API-Rust | `.expect()` em `src/` é aceitável (só vale para `unwrap()`) | R6 proíbe `unwrap()` e o espírito alcança `expect()`; além disso o slicing `offset..offset+4` estoura na borda | Revisão adversarial do PR #36: achado menor |

## Bateria de mutação

5/5 mutantes pegos da rodada anterior + 6/6 novos = 11/11 pegos, 2/2 controles verdes.

### Rodada original

| Mutação | Teste que pegou |
|---|---|
| Remove region_read32 do read32 → tudo cai na RAM | D1, D2, D3, D5, D6 |
| Remove exclusão KSEG1 do scratchpad | D2 (KSEG1 devolve dados) |
| Remove Isc check de sw | D4 (store passa com Isc=1) |
| Endereço do BCC trocado (FFFE0130→FFFE0134) | D6 (escrita vai pra RAM) |
| Range do scratchpad reduzido (0x3FF→0x3FB) | D3 (testemunha RAM revela alias) |
| **Controle:** renomear is_isc → cache_isolated | Todos verdes |

### Rodada de correção

| Mutação | Teste que pegou |
|---|---|
| Remove memctrl/BCC de region_read_byte | `memctrl_bcc_read8_read16_nao_alias_ram` (F1: 0xEE vs 0x88) |
| Remove catch-all de region_read32 | `io_catch_all_nao_corrompe_ram` (F2: 0xEEEEEEEE vs 0) |
| Remove catch-all de region_write32 | `io_catch_all_nao_corrompe_ram` (F2: 0xFFF vaza pra RAM) |
| Remove catch-all de region_read_byte | `io_catch_all_nao_corrompe_ram` (F2: read8 devolve 0xEE) |
| Remove catch-all de region_write_byte | `io_catch_all_nao_corrompe_ram` (F2: write16/write8 vazam pra RAM) |
| Move isc antes do addr calc em sw (reverte F3) | `isc_nao_engole_address_error_sw` (F3: CAUSE=0 vs 0x14) |
| **Controle:** restaurar .expect() em Scratchpad::read32 | Todos verdes |
| **Controle:** renomear is_isc → cache_isolated | Todos verdes |

## Placar antes → depois

209 → 212 testes (+3 bus_scratchpad_isc).

## Revisão cruzada (orquestrador)

Três lacunas de comportamento (F1, F2, F3) mais um achado menor (F5) identificados na
revisão adversarial do PR #36. Todos corrigidos e cobertos por teste nesta rodada.

## Decisões e notas

1. **Isc no CPU, não no Bus.** O Bus não conhece COP0 (regra R3 + armadilha 6 do STATUS). A checagem `(sr >> 16) & 1` está em `sw`/`sb`/`sh`/`swl`/`swr` no CPU, retornando cedo sem chamar o bus. Reads não são afetados (não há dado cacheado para servir).
2. **Scratchpad ignorado em KSEG1.** `region_read32`/`region_write32` checam `kseg == 0b101` e devolvem 0/ignoram. Comportamento ASSUMIDO até o 1.11 (Bus Error).
3. **MemCtrl e BCC são stubs.** Apenas guardam e retornam o valor escrito via `write32`/`read32`. Sub-word writes são ignorados. RAM_SIZE não afeta espelhos.
4. **KSEG2 não passa por `to_physical`.** Endereços em KSEG2 (0xC0000000+) mantêm o valor original via braço `_`, o que faz `0xFFFE_0130` chegar intacto ao region decode.
5. **Isc não engole AdES.** A checagem de `is_isc()` agora ocorre após o cálculo do endereço e a validação de alinhamento nos cinco stores (sw, sb, sh, swl, swr). AdES dispara mesmo com Isc=1.
6. **Catch-all 0x1F801000..0x1F801FFF.** Leitura devolve 0, escrita engolida, para toda a faixa de I/O que não tem registrador implementado. **Dívida registrada:** IRQ e DMA serão implementados no M3, GPU no M2, timers no M3. O catch-all será substituído por decodificação real em cada iteração.

## Erro de processo: escopo múltiplo reprovado pelo commit-lint

Quatro commits desta iteração usaram escopo com vírgula (`feat(bus,cpu)`, `fix(test,bus)`,
`test(bus,cpu)`, `fix(bus,cpu)`) porque a mudança tocava dois módulos. O job `commit-lint`
reprovou o PR #36: o regex é `^(test|feat|fix|refactor|docs|chore)\([a-z0-9-]+\): .+`, que
não aceita vírgula. Toda a história da `main` até aqui usa escopo único, então quem estava
fora do padrão eram os commits, não o lint — a regra foi mantida e as mensagens reescritas
(escopo `bus`, com o segundo módulo citado no resumo). As árvores são idênticas às originais
(`git diff` vazio contra o head antigo), só as mensagens mudaram.

Consequência para o registro: os SHAs `head_depois` das duas linhas da 0022 em
`docs/metricas.csv` (`3e30915` e `a2d8afe`) apontam para commits que a reescrita órfãos.
O mapeamento é `3e30915 → 3faa73f` e `a2d8afe → bba175d` (merge `b947ff6`). As linhas ficam
com os SHAs originais de propósito: são o que o trabalhador de fato produziu.

Lição para as próximas iterações: o handoff deve dizer explicitamente ao trabalhador que o
escopo do commit é **um único identificador** `[a-z0-9-]`, mesmo quando a mudança toca dois
módulos.
