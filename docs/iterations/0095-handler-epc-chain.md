# 0095 — handler-epc-chain

- **Data:** 2026-07-30
- **Item do roadmap:** 4.4e
- **Objetivo:** Handler de exceção em 0x80000080 retorna ao EPC corretamente (mfc0+jr+rfe) e despacha para tabela de eventos do kernel.

## Revisão do PR anterior

Revisão do PR #109 (iter 0094): sem achados novos.

Nove padrões conferidos:
1. **Teste que não mede — verificado.** Apliquei mutação de `MERGED` para `NAO_EXISTE` no script e o teste `gh_pr_merge_verifica_estado_merged_apos_o_merge` falhou. O teste mede.
2. **Parâmetro não consumido — N/A.** Sem comandos GPU neste PR.
3. **Regra de borda trocada — N/A.** Sem rasterização.
4. **Campo de bit lido errado — N/A.** Sem campos de bit de hardware.
5. **Panic ou laço ilimitado — sem achados.** Sem unwrap/unsafe fora de teste.
6. **Citação de spec — verificado.** O doc da 0094 cita "Nenhuma seção de spec de hardware", sem citações falsas.
7. **Escopo transbordado — sem achados.** Item 10.37 implementa exatamente o que foi pedido: inversão da ordem em Wait-Checks + verificação MERGED após merge.
8. **Portão — manifesto 0094 confirmado.** Bateria 5/5, controles 2/2.
9. **Manifesto arquivado — sem arquivamentos.**

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | cop0cmd=10h - RFE opcode - Prepare Return from Exception (L712) | `docs/reference/02-cpu.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | delay-slot | Assumi que RFE automaticamente saltava para EPC (como x86 iret). Implementei `branch_target = Some(EPC)` dentro do RFE | `docs/reference/02-cpu.md` L712: "RFE does NOT automatically jump to EPC. Instead, the exception handler must copy EPC into a register, and then jump to that address." | Li a spec (R1) e vi que o handler anterior estava errado desde a 0093 |
| 2 | API-Rust | Instalei handler + I_MASK=1 no `Bus::new()`. Com BIOS vazia, VBlank interrompe os NOPs infinitamente, PC não avança | O handler deve ser instalado apenas quando há BIOS real; I_MASK=1 sem handler correto = loop de interrupções | Teste `desktop_boot::bios_vazia_mostra_display_ligado_padrao_gpu` falhou: PC=116 em vez de PC=4M após 1M passos |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0095-handler-epc-chain.mut

| Registro | Rótulo | Testes que pegaram, conforme o `.resultado` |
|---|---|---|
| m1 | handler sem mfc0 — k0 com lixo | `handler_retorna_ao_epc_apos_interrupcao` |
| m2 | handler sem jr — não salta para EPC | `handler_retorna_ao_epc_apos_interrupcao` |
| m3 | rfe no lugar do jr (inverte) | `handler_retorna_ao_epc_apos_interrupcao` |
| m4 | handler sem RFE | `handler_no_vector_0x80_acknowledge_istat_e_rfe_restaura_iec` |
| m5 | jr r0 em vez de jr k0 | `handler_retorna_ao_epc_apos_interrupcao` |
| c1 | comentário inofensivo antes do handler | sobreviveu |
| c2 | nop extra ao final do handler | sobreviveu |

## Placar antes → depois

Workspace: **688** → **689** testes (+1: `handler_retorna_ao_epc_apos_interrupcao`).

## Decisões e notas

1. **Handler anterior não retornava ao EPC.** O handler instalado pela 0093 terminava em `rfe; nop` sem `mfc0 k0,epc; jr k0`. A spec `02-cpu.md` L712 é explícita: RFE não salta automaticamente para EPC. O handler agora faz `mfc0 k0, EPC; ...; jr k0; rfe` (rfe no delay slot do jr). O `nop` de guarda foi mantido após `rfe`.

2. **I_MASK=0 bloqueia interrupções na BIOS.** O `irq.pending()` retorna false quando I_MASK=0x0000, então CAUSE.IP nunca acende. A BIOS nunca escreve I_MASK (4.4d). Sem interrupções, o handler de exceção em 0x80000080 nunca executa, e a tabela de eventos do kernel nunca é despachada. Este item conserta o HANDLER para quando interrupções chegarem; o 4.4d (I_MASK) é o próximo passo para fazê-las chegar.

3. **Manifesto 0093 reparado.** As âncoras m2, m3, m4 do manifesto 0093 não casavam mais depois da mudança de endereços no handler (0x80000088→0x8000008C, 0x8000008C→0x80000090, 0x80000090→0x80000098). Reparadas: m2 agora aponta para 0x8000_008C, m3 para 0x8000_0090, m4 para 0x8000_0098.
