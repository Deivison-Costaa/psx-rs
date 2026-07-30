# 0092 — input-desktop

- **Data:** 2026-07-30
- **Item do roadmap:** 6.2
- **Objetivo:** Mapeamento de teclado no psx-desktop injeta estado de botoes no digital pad.

## Revisao do PR anterior

PR #106 (iter 0091): **sem achados de defeito critico**.

Nove padroes conferidos:
1. **Teste que nao mede — sem achados.** Os testes de sio_digital_pad conferem valores golden (ID 5A41h, botao 0=Pressed 1=Released, 0xFF para HiZ).
2. **Parametro nao consumido — N/A.** SIO nao tem FIFO de parametro; cada byte em TX_DATA e processado individualmente.
3. **Regra de borda trocada — N/A.** SIO e periferico serial.
4. **Campo de bit lido errado — sem achados.** Mascaras de STAT (bits 0/1/2/7/9) e CTRL (bits 1/4/6/12) conferem com `docs/reference/10-controllers-memcards.md` e `docs/reference/14-io-map.md`.
5. **Panic ou laco ilimitado — sem achados.** Sem `unwrap()`/`expect()`/`unsafe`; `rx_fifo` usa `.get()` com fallback.
6. **Citacao de spec — sem achados.** As citacoes do doc 0091 apontam para secoes corretas da spec.
7. **Escopo transbordado — sem achados.** Implementado apenas SIO0 + digital pad conforme o item 6.1.
8. **Portao — sem achados.** Manifesto 0091 validado com 6/6 mortos e 2/2 controles verdes; `.resultado` rastreado.
9. **Manifesto arquivado — sem arquivamentos.**

Tambem conferido:
- **PR #105: o teste desktop_boot.rs foi corrigido no 0091 (`assert_eq pc`).** A bateria do 0090 e historica (PR #105 ja merged) e o manifesto 0090 nao foi alterado; a ancora `Gpu::new` continua casando.
- **`cargo fmt --all` + `cargo clippy --all-targets -- -D warnings` verdes.**
- **`cargo test --all` verde com 681 testes.**
- **`pwsh scripts/confere-citacoes.ps1` verde.**

## Spec consultada

| Fonte | Secao | Arquivo local |
|---|---|---|
| psx-spx | Standard Controllers (L613-L633) | `docs/reference/10-controllers-memcards.md` |
| psx-spx | Normal Mode - Command 42h (L1449) | `docs/reference/10-controllers-memcards.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | `egui::Key::ShiftRight` existe no egui 0.32 | O enum `Key` do egui 0.32 nao tem variantes para Shift/Control/Alt — apenas teclas de comando, pontuacao, digitos e letras | Erro de compilacao `E0599`; substituido por `Key::Tab` para Select |
| 2 | manifesto | Controle c1 podia apontar para `crates/psx-desktop/src/main.rs` | O alvo do manifesto e `sio.rs`; `mutantes.ps1` so edita arquivos no alvo declarado | `mutantes.ps1` falhou com "edicao @@DE encontrada 0 vez(es)"; corrigi os controles para mutacoes no fonte |

## Bateria de mutacao

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0092-input-desktop.mut

| # | Tipo | Rotulo | Resultado |
|---|---|---|---|
| m1 | mutante | set_buttons ignora parametro (no-op, mantem 0xFFFF) | MORREU |
| m2 | mutante | send_byte ignora button_state e sempre responde 0xFF para bytes 3 e 4 | MORREU |
| m3 | mutante | send_byte troca high/low dos botoes (shift errado) | MORREU |
| m4 | mutante | button_state inicializado com zero em vez de 0xFFFF | MORREU |
| m5 | mutante | send_byte retorna 0x00 para bytes de botao em vez do estado real | MORREU |
| c1 | controle | renomeia variavel local button_state para _bs em set_buttons | verde |
| c2 | controle | extrai button_state em variavel antes do match | verde |

## Placar antes → depois

Workspace: **678** → **681** testes (+2: `botoes_pressionados_aparecem_na_resposta_42h`, `botoes_soltos_retornam_ff`; +1: desktop_boot `assert_eq pc`).

## Decisoes e notas

1. **Mapeamento de teclas.** Arrow keys → D-Pad, Z → Cross, Space → Circle, A → Square, S → Triangle, Enter → Start, Tab → Select, D/F/E/R → L1/R1/L2/R2 (botoes do `docs/reference/10-controllers-memcards.md` L615-L633). O estado de botoes e injetado em `poll_input()` antes do loop da CPU a cada frame.

2. **Pad conectado por padrao no desktop.** `PsxDesktop::new()` chama `bus.sio_mut().connect_digital_pad(true)`. Sem isso, a BIOS nao detecta o controle e o menu nao responde.

3. **button_state em Cell<u16>.** 0xFFFF = todos soltos; bit=0 significa pressionado (`docs/reference/10-controllers-memcards.md` L615-L633). set_buttons substitui o estado inteiro (nao acumula).

4. **Sem teste de aceitacao com BIOS real.** O STATUS.md pede "tecla pressionada aparece no registrador de leitura do pad" — isso e medido pelo orquestrador com a BIOS. Os testes unitarios em `sio_digital_pad.rs` verificam que `set_buttons` altera os bytes de resposta do comando 42h.

5. **poll_input antes do loop da CPU.** Isso garante que o estado dos botoes e atualizado a cada frame de UI, e o codigo da BIOS que le o pad no meio do frame ve o estado correto.
