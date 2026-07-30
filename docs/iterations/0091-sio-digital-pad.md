# 0091 — sio-digital-pad

- **Data:** 2026-07-30
- **Item do roadmap:** 6.1
- **Objetivo:** SIO0 com registradores mapeados em memória e digital pad respondendo ao comando 42h.

## Revisão do PR anterior

PR #105 (iter 0090): **um achado**.

**Achado G1 — `desktop_boot.rs`: loop da CPU não participa de nenhuma asserção.** O teste `bios_vazia_mostra_display_ligado_padrao_gpu` executa 1M passos de CPU que não afetam as asserções — `Gpu::new()` já inicia com display ligado (bit 23 = 1 em `stat = 0x1480_2000`), e o `framebuffer_for_display()` retorna `Some` independentemente de a CPU ter executado ou não. Mutação confirmada: removi o loop e o teste continuou passando. Corrigido com `assert_eq!(cpu.pc, 0xBFC0_0000 + 1_000_000 * 4)` antes e depois do loop, provando que o PC avança.

Nove padrões conferidos:
1. **Teste que não mede — pego G1 acima.** Corrigido na mesma rodada.
2. **Parâmetro não consumido — N/A.** SIO não tem FIFO de parâmetro como GPU; cada byte escrito em TX_DATA é um comando independente.
3. **Regra de borda trocada — N/A.** SIO é periférico serial, sem rasterização.
4. **Campo de bit lido errado — sem achados.** As máscaras de STAT e CTRL seguem a spec de `docs/reference/10-controllers-memcards.md` e do documento de Serial Interfaces.
5. **Panic ou laço ilimitado — sem achados.** Sem `unwrap()`/`expect()`/`unsafe` (R6); `rx_fifo` usa `.get()` com fallback.
6. **Citação de spec — pendente de verificação.** A spec de SIO foi fetcheada do psx-spx via webfetch; o doc cita `docs/reference/10-controllers-memcards.md` L568 (Controller ID) e L1449 (Command 42h).
7. **Escopo transbordado — sem achados.** Implementado apenas SIO0 + digital pad; SIO1 (serial port) e memory card ficam para itens futuros.
8. **Portão — manifesto 0090 afetado pela correção do teste.** O manifesto 0090 usava o teste antigo; a correção do desktop_boot.rs nesta rodada altera o teste, mas o manifesto 0090 é histórico (PR #105 já merged).
9. **Manifesto arquivado — sem arquivamentos.**

Também conferido:
- **PR #105 merge commit preserva os commits test→feat→docs.**
- **`cargo fmt --all` + `cargo clippy --all-targets -- -D warnings` verdes.**
- **`cargo test --all` verde com 677 testes.**
- **`pwsh scripts/confere-citacoes.ps1` verde.**

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | Serial Interfaces (SIO) registers | Fetch online (`serialinterfacessio.md`) |
| psx-spx | Controller ID (Halfword Number 0) (L568) | `docs/reference/10-controllers-memcards.md` |
| psx-spx | Normal Mode - Command 42h "B" - Read Buttons (L1449) | `docs/reference/10-controllers-memcards.md` |
| psx-spx | Standard Controllers (L613) | `docs/reference/10-controllers-memcards.md` |
| psx-spx | I/O Map — SIO registers | `docs/reference/14-io-map.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | `region_read_byte` poderia chamar métodos `&mut self` do Sio | O Sio precisa de interior mutability (Cell/RefCell) porque leituras de RX_DATA têm efeito colateral (pop da FIFO) | Erro de compilação: `read_byte` precisava de `&mut self` mas `region_read_byte` recebe `&self`; resolvido com `Cell`/`RefCell`, seguindo o padrão do CDROM |
| 2 | nenhum | Assumi que a spec SIO estava em `docs/reference/13-sio.md` (não existe) | A spec de registradores SIO está no documento `serialinterfacessio.md` do psx-spx, não fetcheado pelo script | `File not found` ao abrir o caminho indicado pelo STATUS.md; fetchei via webfetch do GitHub |
| 3 | timing | A transferência poderia ser modelada como instantânea sem efeitos colaterais | O protocolo SPI-like exige que o byte de endereço (01h) seja enviado primeiro e sua resposta (HiZ) seja ignorada; o contador de bytes por transação (/CS) é essencial | Teste `cs_desassertado_nao_transfere` falhou na primeira versão porque `write_tx` não verificava /CS; corrigido |

## Bateria de mutação

Placar da bateria: 6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0091-sio-digital-pad.mut

| # | Tipo | Rótulo | Resultado |
|---|---|---|---|
| m1 | mutante | resposta de pad digital sempre 0xFF (ignora pad_connected) | MORREU |
| m2 | mutante | STAT.1 nunca reporta RX data (rx_fifo sempre vazio) | MORREU |
| m3 | mutante | send_byte nao empurra resposta para rx_fifo | MORREU |
| m4 | mutante | cs_asserted sempre retorna false | MORREU |
| m5 | mutante | ctrl bit4 ack nao limpa STAT.9 | MORREU |
| m6 | mutante | DTR transicao nao reseta byte_count | MORREU |
| c1 | controle | variavel `_unused` no inicio de read_stat | verde |
| c2 | controle | reordena match de read_byte (1044..104F antes de 1040) | verde |

## Placar antes → depois

Workspace: **671** → **678** testes (+7: `sio_digital_pad`).

## Decisões e notas

1. **Interior mutability com Cell/RefCell.** Leituras de RX_DATA destroem o dado (pop da FIFO), então `read_byte` precisa de efeito colateral mesmo recebendo `&self`. O CDROM já usa `Cell`; seguimos o mesmo padrão.

2. **SIO1 não implementado.** Os registradores de SIO1 (0x1F801050-0x1F80105E) não são necessários para o digital pad. O catchall do bus retorna 0 para leituras e ignora escritas.

3. **Protocolo de comunicação.** O digital pad responde ao comando `01h 42h 00h 00h 00h` com `(HiZ) 41h 5Ah FFh FFh` (ID `5A41h` + botões `0xFFFF` todos soltos). O byte de endereço (`01h`) tem resposta HiZ que deve ser ignorada.

4. **Timing do /ACK.** Modelado como síncrono: STAT.7 = 1 após cada byte enviado, STAT.7 = 0 após leitura do RX_DATA. O IRQ7 é gerado quando CTRL.12 (DSR interrupt enable) está setado.

5. **ps1-tests/input/pad não executado.** O STATUS.md menciona `ps1-tests/input/pad` para medição, mas o placar do scoreboard local requer BIOS. A medição antes/depois fica para o orquestrador ou para quando o boot da BIOS destravar.

6. **Spec de registradores SIO fetcheada online.** O `fetch-reference-docs.ps1` não inclui `serialinterfacessio.md` porque o psx-spx o tem como documento separado. O conteúdo foi lido via webfetch e está referenciado neste doc.
