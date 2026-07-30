# 0099 — bios-mirror

- **Data:** 2026-07-30
- **Item do roadmap:** 4.4f
- **Objetivo:** investigar e corrigir o dispatch de eventos do kernel — o handler em 0x80000080 despacha para 0x00000C80 mas o callback de VSync nao e invocado.

## Revisão do PR anterior

Revisao do PR anterior (#113, iter 0098): sem achados.

9 padroes conferidos:
1. TESTE QUE NAO MEDE — bateria 6/6 garante medicao, e os 4 testes sao de comportamento verificado manualmente contra processos falsos
2. PARAMETRO NAO CONSUMIDO — nao aplicavel (script PowerShell)
3. REGRA DE BORDA TROCADA — nao aplicavel
4. CAMPO DE BIT LIDO ERRADO — nao aplicavel
5. PANIC ou LACO ILIMITADO — sem unwrap()/expect() fora de teste (R6)
6. CITACAO DE SPEC — doc da 0098 declara "Nenhuma secao de spec de hardware", sem citacoes
7. ESCOPO TRANSBORDADO — item 10.38 focado no detector de travamento, sem extras
8. PORTAO QUE NAO MEDE — verificacao de comportamento documentada no item 5 do doc
9. NAO ARQUIVE MANIFESTO — .resultado versionado e rastreado

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Interrupts, § Interrupt Request/Execution | `docs/reference/11-interrupts.md` |
| hardware | BIOS ROM mirror (0x1FC00000→0x00000000 via KUSEG/KSEG0) | comportamento inferido de documentacao de hardware e emuladores de referencia |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | enderecamento | Que a BIOS escreve o codigo do handler de eventos na RAM durante a inicializacao do kernel | O handler em 0x00000C80 vem do espelhamento da BIOS ROM. A copia do kernel escreve ZEROS em 0xC80 (ROM fonte em 0x10780 e zeros) | Instrumentacao de leitura/escrita em 0xC80 — RAM tem zeros, ROM tem codigo valido |
| 2 | enderecamento | Que o espelhamento era desabilitado por qualquer write na faixa de memory control (0x1F801000-0x1F801023) | Apenas write no EXP1_BASE (0x1F801000) desabilita o mirror. Write em EXP2 (0x1F801010) nao desabilita | Teste `mirror_nao_desativado_por_outros_registradores` — bateria m3 |
| 3 | timing | Que o mirror seria suficiente para o dispatch de VSync funcionar | O stub em 0x80000080 (3 instrucoes) nao configura $sp antes de pular para 0xC80. O codigo em ROM 0xC80 comeca com `lw $ra, 0x2C($sp)` — precisa de stack frame | Execucao com mirror ativo — TTY identico, VSync: timeout persiste |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0099-bios-mirror.mut

| Registro | Rotulo | Testes que pegaram |
|---|---|---|
| m1 | mirror nunca ativo | mirror_ativo_leitura_kuseg_retorna_bios, mirror_ativo_leitura_kseg0_retorna_bios |
| m2 | KSEG1 tambem passa pelo mirror | mirror_ativo_leitura_kseg1_retorna_ram |
| m3 | mirror desativado por qualquer write na faixa | mirror_nao_desativado_por_outros_registradores |
| m4 | teto do mirror 8MB | mirror_nao_afeta_enderecos_acima_de_512kb |
| m5 | mirror desativado no write32 removido | mirror_desativado_por_write_na_exp1_base |
| c1 | variavel cosmetica no read32 | sobreviveu |
| c2 | constante 0x8_0000 → 524288 | sobreviveu |

## Placar antes → depois

Workspace: **696** testes (+6: `bus_bios_mirror` 5 testes + `bios_dispatch_vsync` 1 teste de aceitacao).

O teste `bios_dispatch_vsync` FALHA (captura o estado quebrado). A correcao completa do dispatch de VSync requer a proxima iteracao (substituicao do stub).

## Revisão cruzada (orquestrador)

(Preenchido pelo orquestrador na revisao do PR.)

## Decisões e notas

1. **O espelhamento da BIOS ROM e necessario mas nao suficiente.** O mirror e uma feature do hardware PS1 que estava ausente. Sem ele, o codigo do handler de eventos em 0x00000C80 e lido como zeros da RAM. Com ele, o codigo correto (ROM 0xC80) fica visivel. Porem, o stub de 3 instrucoes em 0x80000080 nao configura $sp, e o codigo em ROM 0xC80 espera uma stack frame montada.

2. **O stub de excecao nao e substituido pelo kernel.** O stub em 0x80000080 (`lui $k0, 0; addiu $k0, 0xC80; jr $k0`) deveria ser substituido durante a inicializacao do kernel. Isso nao acontece. Investigar por que na proxima iteracao.

3. **O kernel copia apenas 0x8BF0 bytes a partir de 0x500.** O loop de copia em ROM 0x420-0x458 copia de ROM 0xBFC10000 para RAM 0x00000500, cobrindo [0x500, 0x90F0). O codigo em 0xC80 esta DENTRO dessa faixa, mas a fonte ROM (offset 0x10780) e ZEROS. O codigo correto esta em ROM 0xC80 (fora da faixa copiada), acessivel apenas via mirror.

4. **O ponteiro da tabela EvCB em 0x108 e configurado corretamente** — valor 0xA000E1EC. A tabela em 0xE1EC contem dados. O handler em 0xC90+ e executado. Mas o codigo em ROM 0xC80 comeca com `lw $ra, 0x2C($sp)` com $sp nao configurado pelo stub.

5. **Proximo passo: investigar por que o kernel nao substitui o stub.** Instrumentar writes para 0x80 durante a execucao.
