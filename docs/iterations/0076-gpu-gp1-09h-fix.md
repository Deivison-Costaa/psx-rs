# 0076 — gpu-gp1-09h-fix

- **Data:** 2026-07-30
- **Item do roadmap:** 10.32
- **Objetivo:** remover as 4 linhas `if bit == 0` que limpavam GPUSTAT.15 ativamente no handler de GP1(09h). Fechar o gate não limpa o latch — foi medido pelo orquestrador em worktree isolado (`gpu/gp0-e1` fecha em 10p/0f sem o `if`, era 9p/1f com ele).

## Revisão do PR anterior

Revisão do PR anterior (#92, iter 0078): achado — testes vazios em `crates/psx-core/tests/disc_flag.rs`.
Os 3 testes (`disc_flag_cue_minimo_aceito_com_bios`, `disc_flag_sem_bios_erro`, `disc_flag_arquivo_cue_inexistente_erro`) chamam `cli_available()` que retorna `false`, entram no `if !cli_available()` e só imprimem um `eprintln` — zero asserções. Sempre passam, não medem nada. A correção foi impedida pelo meta-teste `bateria_nomes_de_teste_existem` (o `.resultado` da 0078 referencia esses nomes de teste).
Dívida registrada no 10.33 do ROADMAP (antes deste doc).
Padrões conferidos:
1. Teste que não mede — **achado** 3 testes vazios em `psx-core/tests/disc_flag.rs`
2. Parâmetro não consumido — sem novos comandos GP0
3. Regra de borda trocada — sem rasterização
4. Campo de bit lido errado — `--disc` parseia caminhos corretamente
5. Panic ou laço ilimitado — sem `unwrap()`/`expect()` fora de teste
6. Citação de spec — `confere-citacoes.ps1` verde
7. Escopo transbordado — item 4.3c bem delimitado; dívida 10.33 registrada
8. Portão que não mede — bateria manual, sem portão automático
9. Manifesto arquivado — nenhum arquivado na 0078

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | GP1(09h) - Set VRAM size (v2) (L943) | docs/reference/03-gpu.md |

A spec de GP1(09h) (L943-958) descreve o bit 0 como portão do decodificador de endereço de Y:
bit 0 = 0 mascara Y para 9 bits; bit 0 = 1 habilita a faixa 512-1023. **Não menciona limpeza
de GPUSTAT.15.** O GPUSTAT.15 é um latch escrito por GP0(E1h) e pelo texpage de polígono
texturizado, gateado na escrita. Fechar o portão não apaga o que já estava latcheado.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | teste-que-certifica-o-defeito | Que GP1(09h).0=0 limpa GPUSTAT.15 ativamente (herdado da iter 0074) | GP1(09h) só controla o gate do decodificador; não mexe em GPUSTAT.15 | Medido pelo orquestrador em worktree isolado: `gpu/gp0-e1` 9p/1f com o `if`, 10p/0f sem ele. Confirmado localmente via scoreboard |
| 2 | manifesto | Que as âncoras do manifesto 0076 não sobreporiam as do manifesto 0076-gpu-texture-disable-gate (que também tem iteracao 0076) | Duas branches diferentes geraram manifestos para a mesma iteração com edições idênticas (m1 e m4/m2) | `mutation_manifest` reprovou "edicao duplicada entre registros". Corrigido estendendo as âncoras de m1 e m4 com linhas de contexto adjacentes |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente - docs/mutantes/0076-gpu-gp1-09h-fix.mut

| Registro | Rótulo | Testes que pegaram |
|---|---|---|
| m1 | re-adiciona `if bit==0` que limpa GPUSTAT.15 ativamente (regressão) | `gp1_09h_fechar_gate_nao_limpa_gpustat_15` |
| m2 | GP1(09h) não atualiza allow_upper_y (corpo vazio) | `gp1_09h_abre_o_gate_do_bit_15` + mais 3 |
| m3 | GP1(09h) lê bit 3 em vez de bit 0 | `gp1_09h_abre_o_gate_do_bit_15` + mais 3 |
| m4 | GP1(09h) sempre abre o gate (allow_upper_y = true) | `gp1_09h_gate_fechado_mascara_reescrita_e1h` |
| m5 | GP1(09h) sempre fecha o gate (bit & 0) | `gp1_09h_abre_o_gate_do_bit_15` + mais 3 |
| c1 | renomeia bit para gate_bit no braço 0x09 | sobreviveu |
| c2 | embrulha allow_upper_y.set em if true | sobreviveu |

## Placar antes → depois

Workspace: **567** testes (após merge com main que trouxe +3 da 0077 e +3 da 0078).

**Hardware — `gpu/gp0-e1`:** **10p/0f** (medido via scoreboard). Antes da correção: 9p/1f
(PR #88). O subteste `testUnsetAllowTextureDisablePreservesBit` esperava GPUSTAT.15=1 após
fechar o gate, mas o `if bit == 0` limpava ativamente.

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo orquestrador. -->

## Decisões e notas

1. A remoção das 4 linhas `if bit == 0 { self.stat.set(s & !(1 << 15)); }` no handler
   GP1(09h) é a correção correta. O gate fecha o decodificador de Y mas o latch do GPUSTAT.15
   mantém o último valor escrito por E1h ou texpage.
2. O teste `gp1_09h_bit0_zero_proibe_gpustat_15` da iter 0074 foi corrigido: renomeado para
   `gp1_09h_fechar_gate_nao_limpa_gpustat_15` com asserção de 1 (não 0) — o latch é mantido.
3. Teste novo `gp1_09h_gate_fechado_mascara_reescrita_e1h`: gate fechado + reescrever E1h
   produz GPUSTAT.15=0 (a máscara de E1h limpa o bit e o gate fechado não o reinsere).
4. Manifestos 0050 e 0074 tiveram âncoras reparadas (o bloco `0x09 =>` encurtou com a remoção
   do `if bit == 0`).
5. Conflitos de merge com main resolvidos: ROADMAP (tomou versão de main), testes (manteve
   HEAD com nomes de teste corrigidos), manifestos (HEAD para o da branch, main para o antigo).
6. O manifesto `0076-gpu-texture-disable-gate.mut` (iteração 0076 anterior, de branch diferente)
   coexiste com este — as âncoras de m1 e m4 foram estendidas para evitar sobreposição de
   edições. O meta-teste `manifesto_de_mutacao_forma_e_integridade` confirma.
