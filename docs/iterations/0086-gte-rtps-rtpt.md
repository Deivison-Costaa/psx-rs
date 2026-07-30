# 0086 — gte-rtps-rtpt

- **Data:** 2026-07-30
- **Item do roadmap:** 5.2
- **Objetivo:** Implementar RTPS e RTPT com divisão UNR, deslocamento de FIFOs e saturação de registradores.

## Revisão do PR anterior

Revisão do PR #100 (iter 0085): sem achados relevantes de defeito.

Nove padrões conferidos:
1. Teste que não mede — todos os testes de `irq_halfword.rs` têm asserções com valores exatos; `assert_ne!` corrigido para `assert_eq!` em `gte_registers.rs`
2. Parâmetro não consumido — sem novos comandos GPU na 0085
3. Regra de borda trocada — N/A (IRQ, não GPU)
4. Campo de bit lido errado — Byte/halfword write/read com offsets e máscaras 0x7FF corretos
5. Panic ou laço ilimitado — sem unwrap/expect/unsafe
6. Citação de spec — `confere-citacoes.ps1` verde
7. Escopo transbordado — Read byte/halfword adicionado junto com write; escopo razoável
8. Portão — `.resultado` rastreado, `mutation_anchors` verde (âncora M2 reparada na 0086)
9. Manifesto arquivado — sem arquivamentos

### Prioridade GP1(09h)

O `if bit == 0` na linha 1747 de `gpu.rs` está no braço **GP1(03h)** (Display Enable), não em GP1(09h). O handler de GP1(09h) (linha 1781) corretamente só seta `allow_upper_y`. O defeito original já estava consertado. Bloco PRIORIDADE não se aplica.

### Blocos autolimitados conferidos

- 10.16: `[x]` — especificação de citações já corrigida
- I_MASK: registrado no ROADMAP
- "sem asserção": registrado como 10.34 (com ç, grep não casa)
- "nome qualificado": registrado como 10.35
- 2.2b VRAM->VRAM: já existe no ROADMAP
- 4.4a/4.4b: já implementados e `[x]`

### Status do boot da BIOS

Verificado: após o 4.4c (I_MASK via SH), o boot da BIOS ainda trava em `VSync: timeout`. TTY: 557 bytes. O bloqueio real ainda não foi identificado — I_MASK permanece 0x0000.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | RTPS (L481), RTPT (L482), GTE Division Inaccuracy (L684), GTE Command Encoding (L117), Data Register Summary (L137), Control Register Summary (L156), GTE Saturation (L341), cop2r63 FLAG (L349) | docs/reference/07-gte.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | registrador-mapeamento | ctc2(rd=5) escreve RT33 (cnt5) | cnt5 = cop2r37 = TRX; RT33 é cnt4 | teste dava IR1=0 em vez de 100; inspeção da tabela de registradores |
| 2 | registrador-mapeamento | VZ0 é cop2r2 | cop2r0=VXY0, cop2r1=VZ0, cop2r2=VXY1 | IR1 errado; trace do layout de registradores |
| 3 | pre-set-de-reg | regs[8] mantém o valor da hora da escrita da instrução | regs[8] tem o último valor setado, todas as instruções leem o mesmo | testes retornavam valores errados (16416 em vez de 20480) |
| 4 | gte-divide-u32 | leading_zeros em u32 (23 zeros para 0x100) | SZ3 é u16, leading_zeros deve ser 7 | SX2/SY2 errados (6619336 em vez de 3276900) |
| 5 | ofx-ofy-offset | OFX=0x0001_0000 é offset desprezível | OFX em 16.16 fixed-point: 1.0 = 1 pixel de offset | SXY2 ficava (101, 52) em vez de (100, 50) |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0086-gte-rtps-rtpt.mut

| # | Tipo | Rótulo | Resultado |
|---|---|---|---|
| m1 | mutante | sf sempre 0 no execute_command | MORREU |
| m2 | mutante | saturate_ir nunca satura | MORREU |
| m3 | mutante | SXY FIFO não desloca | MORREU |
| m4 | mutante | UNR overflow não seta flag 17 | MORREU |
| m5 | mutante | SZ3 não satura | MORREU |
| c1 | controle | comentário antes de execute_command | verde |
| c2 | controle | comentário antes de rtps | verde |

## Placar antes → depois

Workspace: **637** → **643** testes (+6: gte_rtps_rtpt).

## Decisões e notas

1. **Execução sem contagem de ciclos.** A spec indica 15 ciclos para RTPS e 23 para RTPT, mas a contagem de ciclos não foi implementada — o comando executa em zero ciclos de CPU. Simplificação registrada; será tratada quando o scheduler for integrado ao GTE.

2. **FLAG bit 31 computado dinamicamente.** `read_control(31)` computa bit 31 como OR dos bits 30-23 e 18-13, conforme `docs/reference/07-gte.md` (L351). Bits 12-30 são armazenados; bits 0-11 são sempre zero. A função `write_control(31)` mascara para preservar apenas os bits writable (12-30).

3. **Âncora do manifesto 0084 reparada.** A adição do FLAG bit 31 em `read_control` quebrou a âncora do mutante m2 da iteração 0084. A âncora foi atualizada para casar com o novo código.

4. **Divisão UNR opera em u16.** A função `gte_divide` converte H e SZ3 para u16 antes do algoritmo, pois `leading_zeros` em u32 produziria contagens erradas para valores de 16 bits.

5. **FIFO de SZ tem 4 estágios.** O RTPT processa 3 vértices; após o terceiro, SZ0 contém o valor anterior (não o do primeiro vértice). O teste `rtpt_processa_tres_vertices_e_desloca_fifos` valida este comportamento.
