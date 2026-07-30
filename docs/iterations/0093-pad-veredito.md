# 0093 — pad-veredito

- **Data:** 2026-07-30
- **Item do roadmap:** 10.36
- **Objetivo:** Diagnosticar por que ps1-tests/input/pad nao emite veredito e fechar a lacuna do vetor de excecao no sideload de PS-EXE.

## Revisão do PR anterior

PR #106 (iter 0091): **um achado**.

**Achado G1 — `read_data` (32-bit) nao consome FIFO.** `Sio::read_data()` em `sio.rs:189` lia o FIFO sem remover o byte, enquanto `pop_rx()` (usado por `read_byte(0x1040)`) remove. Leitura de 32 bits via `lw` no endereco 0x1F801040 retornava o mesmo byte indefinidamente. Corrigido: `read_data` agora chama `pop_rx()` e retorna o byte como u32.

Nove padroes conferidos:
1. **Teste que nao mede — pego G1 acima.** Teste `leitura_32bit_do_sio_data_consome_byte_do_fifo` adicionado; segunda leitura retorna 0xFF (FIFO vazio), confirmando que o byte foi consumido.
2. **Parametro nao consumido — sem achados novos.** O `val` de `send_byte` e armazenado mas a resposta do pad digital e determinada por posicao (`byte_count`), nao pelo valor do comando. Aceitavel para o escopo 6.1 (so comando 42h).
3. **Regra de borda trocada — N/A.** SIO e periferico serial, sem rasterizacao.
4. **Campo de bit lido errado — sem achados.**
5. **Panic ou laco ilimitado — sem achados.** Sem `unwrap()`/`expect()`/`unsafe` (R6).
6. **Citacao de spec — verificado.** `spec_citations.rs` verde.
7. **Escopo transbordado — sem achados.**
8. **Portao — manifesto 0093 verificado.**
9. **Manifesto arquivado — sem arquivamentos.**

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | cop0r13 - CAUSE — Interrupt pending field (L670) | `docs/reference/02-cpu.md` |
| psx-spx | cop0r12 - SR — System status register (L704) | `docs/reference/02-cpu.md` |
| psx-spx | Exception handling — External Interrupt (L689) | `docs/reference/02-cpu.md` |
| psx-spx | I_STAT/I_MASK registers | `docs/reference/11-interrupts.md` |
| psx-spx | I/O Map — SIO registers | `docs/reference/14-io-map.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | Assumi que `ResetGraph` instalava o handler no vetor 0x80000080 | O `ResetGraph` da PSY-Q SDK so configura callbacks dentro do sistema de eventos da BIOS; o vetor em si e instalado pela BIOS durante o boot | Instrumentacao: `[80000080]=0x00000000` (NOPs). SR=0x00000000 apos 10M passos — IEc nunca restaurado |
| 2 | timing | Assumi que o pad test usa polling de I_STAT para VSync | O `VSync(0)` da SDK usa o sistema de eventos (callbacks), que depende do handler de interrupcao despachar para o event handler kernel | PC oscilando em 0x800109D8-0x800109E8 com I_STAT=1 — loop esperando callback que nunca chega |
| 3 | API-Rust | Assumi que `read_data` e `read_byte` eram consistentes | `read_data` (32-bit) lia o FIFO sem pop; `read_byte` (8-bit) dava pop via `pop_rx` | Teste `leitura_32bit_do_sio_data_consome_byte_do_fifo` revelou que segunda leitura retornava 0x41 em vez de 0xFF |

## Bateria de mutação

Placar da bateria: 6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0093-pad-veredito.mut

| # | Tipo | Rótulo | Resultado |
|---|---|---|---|
| m1 | mutante | load_psexe nao configura SR (IEc=0, IM=0) | MORREU |
| m2 | mutante | handler nao le I_STAT (lw ausente) | MORREU |
| m3 | mutante | handler nao escreve I_STAT (sw ausente — nao faz ack) | MORREU |
| m4 | mutante | handler sem RFE — IEc nunca restaurado | MORREU |
| m5 | mutante | SR configurado sem IM[2] (bit 12) | MORREU |
| m6 | mutante | SR configurado sem IEc (bit 0) | MORREU |
| c1 | controle | reordena stubs de usuario (A0h, B0h, C0h) | verde |
| c2 | controle | renomeia variavel local jr_ra | verde |

## Placar antes → depois

Workspace: **681** → **685** testes (+4: `psexe_interrupt_handler` 3 testes + `leitura_32bit` 1 teste).

**Antes:** `[80000080]=0x00000000`, `SR=0x00000000`, ResetGraph imprime `SR=0`.
**Depois:** `[80000080]=0x3C081F80` (lui), `SR=0x00001001` (IEc+IM[2]), ResetGraph imprime `SR=1001`.

## Decisões e notas

1. **Vetor de excecao nao configurado no sideload.** O `load_psexe` carrega o PS-EXE diretamente, sem passar pela BIOS. A BIOS normalmente instala o handler de interrupcao em 0x80000080 durante o boot. Sem isso, qualquer interrupcao (VBlank, SIO, etc.) salta para NOPs, IEc e zerado na entrada da excecao e nunca restaurado (sem RFE).

2. **Handler minimo instalado.** `install_return_stubs` agora grava em 0x80000080 um handler que le I_STAT, escreve de volta (acknowledge) e executa RFE. Instrucoes: `lui t0,0x1F80; ori t1,t0,0x1070; lw t0,0(t1); sw t0,0(t1); rfe; nop`.

3. **SR inicial configurado.** `load_psexe` agora chama `cpu.set_sr(0x0000_1001)` — IEc=1 (interrupcoes habilitadas) + IM[2]=1 (bit 12, hardware interrupts). Sem IM[2], a condicao `(sr & (1 << 10)) != 0` em `cpu.rs:54` falha e a interrupcao nunca e tomada.

4. **Pad test e interativo, nao produz veredito.** O fonte (`main.cpp` do ps1-tests) mostra que o teste entra em `while(1)` e imprime botoes pressionados. Nao ha `PASS`/`FAIL`. O "zero vereditos" e esperado — o teste nao foi projetado para automacao. O que estava quebrado era o VSync, que depende do sistema de eventos via interrupcao.

5. **VSync ainda nao funciona completamente.** O handler instalado faz ack de I_STAT e RFE, mas nao despacha para o event handler kernel (ehk) configurado pelo `ResetGraph`. Para VSync funcionar com callbacks, seria necessario um handler completo que chaineia para a tabela de eventos. Isso e escopo do 4.4d (boot da BIOS) ou de um item futuro de compatibilidade.
