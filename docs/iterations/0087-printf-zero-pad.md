# 0087 — printf-zero-pad

- **Data:** 2026-07-30
- **Item do roadmap:** 1.11c
- **Objetivo:** Implementar flags de largura e zero-pad no printf da BIOS (%08x, %2d).

## Revisão do PR anterior

Revisão do PR #101 (iter 0086): **achado 1 defeito**.

### Defeito: máscara FLAG bit 11

`read_control(31)` e `write_control(31)` usavam a máscara `0x7FFF_F800`, que preserva o bit 11. A spec (`docs/reference/07-gte.md` L371-373) diz que bits 0-11 são read-only e bits 12-30 são writable. A máscara correta é `0x7FFF_F000`. O erro permitia que software escrevesse e lesse o bit 11 do FLAG, que deveria ser sempre 0.

Correção: máscara alterada para `0x7FFF_F000` nas duas funções. Âncora do manifesto 0084 (m2) reparada para refletir a nova máscara. Teste `flag_bit11_read_only_escrita_com_bit11_retorna_0` adicionado em `gte_registers.rs`.

Nove padrões conferidos:
1. Teste que não mede — todos os testes têm asserções com valores exatos; o `assert_ne!` em `rtps_satura_ir1` e `rtps_divide_overflow` é apropriado para bit flags
2. Parâmetro não consumido — sem novos comandos GPU; GTE não tem parâmetros de comando que afetem FIFO além do cmd/sf/lm
3. Regra de borda trocada — N/A (GTE, não GPU)
4. Campo de bit lido errado — flag_error_bit ORs bits 30-23 e 18-13 corretamente (exclui IR3 bit 22); saturate_ir force_lm0 para IR3 conforme spec
5. Panic ou laço ilimitado — sem unwrap/expect/unsafe; UNR_TABLE idx validado pelo clamp 0x8000..0xFFFF
6. Citação de spec — `confere-citacoes.ps1` verde
7. Escopo transbordado — FLAG bit 31 adicionado junto com RTPS/RTPT; escopo razoável para o item 5.2
8. Portão — manifesto reparado (âncora 0084 atualizada), `.resultado` rastreado
9. Manifesto arquivado — sem arquivamentos

### Prioridade GP1(09h)

O `if bit == 0` na linha 1747 está no braço GP1(03h) (Display Enable), não GP1(09h). O handler GP1(09h) (linha 1781) corretamente só seta `allow_upper_y`. O defeito original já estava consertado. Bloco PRIORIDADE não se aplica.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | A(3Fh) printf hook (já implementado no 1.11b) | `crates/psx-core/src/cpu.rs` |

A spec do printf da BIOS não está documentada em psx-spx; o comportamento esperado (largura mínima e zero-pad) é o padrão POSIX/ISO C para `printf`.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | nenhum | A implementação foi direta; o único ajuste foi o `_` do match de desconhecido que emitia `0` de `width=0` | O especificador desconhecido deve emitir `%` + spec literal, sem largura | teste `printf_especificador_desconhecido_sai_literal` quebrou |
| 2 | nenhum | O padding de números negativos com zero-pad segue a ordem normal (pad + sign + digits) | printf padrão: sign vem antes do zero-pad (`-0001`, não `000-1`) | inspeção do código ao escrever `emit_padded_signed` |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0087-printf-zero-pad.mut

| # | Tipo | Rótulo | Resultado |
|---|---|---|---|
| m1 | mutante | zero_pad nunca true (flag '0' ignorada) | MORREU |
| m2 | mutante | width sempre 0 (largura ignorada) | MORREU |
| m3 | mutante | pad_char sempre espaco (zero-pad ignorado) | MORREU |
| m4 | mutante | padding no else branch usa zero pad (ignora espaco) | MORREU |
| m5 | mutante | parse_printf_spec nao parseia digitos como largura | MORREU |
| c1 | controle | adiciona let _ = body_len | verde |
| c2 | controle | adiciona let _ = 0 no inicio de emit_signed | verde |

## Placar antes → depois

Workspace: **643** → **651** testes (+8: cpu_printf_hook +1: gte_registers flag bit 11).

## Decisões e notas

1. **Especificador desconhecido com largura perde a largura.** Antes, `%2o` emitia `%2o` literalmente (o `2` era tratado como spec desconhecido e depois `o` como literal). Agora `%2o` emite `%o` (a largura é consumida pelo parser mas o spec desconhecido só emite `%` + spec). A BIOS nunca usa especificadores desconhecidos com largura, então sem impacto real.

2. **Assinatura de emit_signed/emit_unsigned/emit_hex alterada.** As funções agora recebem `zero_pad: bool` e `width: usize`. Chamadores antigos (inexistentes fora do `do_printf`) precisariam ser atualizados.
