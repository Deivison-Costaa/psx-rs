# 0194 — snapshot-save-state

- **Data:** 2026-08-03
- **Item do roadmap:** 9.2
- **Objetivo:** congelar a máquina inteira num vetor de bytes e trazê-la de volta idêntica —
  e ligar isso a F5/F8 com dez slots por jogo no app desktop.

## Spec consultada

Nenhuma. Save state não é hardware: não há seção de psx-spx que diga o que gravar. O que
define o conteúdo é o próprio `Bus` — e é justamente por isso que a bateria de mutação deste
item vale mais do que uma citação: cada mutante retira um subsistema do estado salvo.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | nenhum | Que um retrato de doze valores (regs, PC, RAM, VRAM, GTE, timers…) bastava para provar que a restauração funciona | A fila de eventos do `scheduler` não aparece em nenhum registrador observável | **Mutante m4 sobreviveu**: apagar `bus.scheduler = estado.scheduler` não quebrou teste nenhum. O teste comportamental passou a comparar o snapshot INTEIRO depois de re-executar, não o retrato |
| 2 | API-Rust | Que `bincode::config::standard()` era a escolha óbvia | O padrão do bincode é varint: a VRAM ocuparia de 1 a 3 MiB conforme o que o jogo tivesse desenhado, e o arquivo mudaria de tamanho a cada save | Medido: 3.281.958 bytes com a VRAM zerada. Trocado por `with_fixed_int_encoding()` — 3.809.483 bytes, sempre |
| 3 | nenhum | Que dava para derivar `Serialize` no `Bus` inteiro | O `Bus` guarda `disc_bin: Option<Vec<u8>>` — a imagem do disco, centenas de MB. Um `derive` no `Bus` botaria o disco dentro de cada slot | Escrito antes de rodar, ao listar os campos. Virou o teste `imagem_do_disco_nao_entra_no_snapshot` (8 MiB de disco não podem crescer o estado em mais de 1 KiB) |

## Bateria de mutação

Placar da bateria: 13/13 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0194-snapshot-save-state.mut

| Mutante | O que quebra | Quem pegou |
|---|---|---|
| m1 | RAM fora da restauração | `estado_volta_inteiro_depois_de_ser_rabiscado` |
| m2 | GPU/VRAM fora da restauração | idem |
| m3 | SPU fora da restauração | `snapshot_de_estado_restaurado_e_byte_a_byte_o_mesmo` |
| m4 | scheduler fora da restauração | `maquina_restaurada_continua_executando_igual` (depois do conserto) |
| m5 | relógio mestre não volta | `estado_volta_inteiro_depois_de_ser_rabiscado` |
| m6 | GTE fora da restauração | idem |
| m7 | timers fora da restauração | `snapshot_de_estado_restaurado_e_byte_a_byte_o_mesmo` |
| m8 | I_STAT/I_MASK fora | `estado_volta_inteiro_depois_de_ser_rabiscado` |
| m9 | SIO fora da restauração | `snapshot_de_estado_restaurado_e_byte_a_byte_o_mesmo` |
| m10 | save de outro jogo aceito | `snapshot_de_outro_jogo_e_recusado` |
| m11 | mágico não conferido | `magico_errado_e_recusado` |
| m12 | versão não conferida | `versao_diferente_e_recusada_dizendo_qual` |
| m13 | corpo começa um byte antes | `estado_volta_inteiro_depois_de_ser_rabiscado` |

## Placar antes → depois

Workspace: 1152 → 1164 testes.

**A bateria foi rodada de novo com a BIOS e os discos escondidos**, simulando o clone limpo
da CI: 13/13 do mesmo jeito. Era a armadilha da 0190 — lá o `gte_fuzz_hardware` se ignorava
sozinho sem o log e todo mutante sobrevivia na CI. Aqui quem mata os mutantes são os testes
de máquina sintética; o `save_state_de_maquina_real_leva_ao_mesmo_lugar` é prova extra, não
cobertura, e a coluna de testes do `.resultado` mostra isso registro por registro.

Tamanho do estado, medido: **3.809.483 bytes** (3,63 MiB) — RAM 2 MiB + VRAM 1 MiB +
RAM do SPU 512 KiB + o resto. Constante, por causa do inteiro de tamanho fixo. Dez slots
por jogo custam 36 MiB de disco.

## Revisão cruzada (orquestrador)

Feita na fusão do PR #208 (2026-08-04), com dois revisores headless de briefs disjuntos.
Consertado na própria branch:

- `snapshot::salva` engolia falha do bincode com `unwrap_or_default()` e produzia arquivo
  de 12 bytes reportado como "salvo (0 KiB)". Virou `Result` com `SnapshotError::Codificacao`.
- A bateria da 0194 só mutou a **restauração**; mutantes de "campo removido do `Estado`"
  (scratchpad, timers, sio, mdec, cdrom, mem_ctrl, bcc, tty_buffer — 8 de 18 campos)
  ficavam verdes porque o roundtrip compara a implementação com ela mesma. Armadura nova:
  `estado_codificado_tem_tamanho_fixo_conhecido` (3.809.483 bytes ancorados) mata a classe
  inteira, e `retrato()` passou a ler scratchpad e TIMER1.
- `serde`/`bincode` ganharam `default-features = false`: a promessa da allowlist ("não abre
  arquivo") passou a ser imposta pelo manifesto, não só pelo uso atual.

Registrado para depois (achados 0198.4, 0198.6): o teste com BIOS real não roda na CI e a
máquina sintética executa só NOPs.

## Decisões e notas

- **`serde` e `bincode` entraram na allowlist do `purity.rs`** — as duas primeiras
  dependências do `psx-core` desde o começo do projeto. A justificativa que o próprio teste
  pedia: as duas são puras (geram código de (de)serialização e trocam esse código por
  bytes), nenhuma abre arquivo, soquete ou relógio. Quem grava o `.state` é o frontend.
- **A imagem do disco e a BIOS ficam de fora**, de propósito, e são **preservadas** na
  restauração (`disco_e_bios_sobrevivem_a_restauracao`). O save state assume que o mesmo
  disco está inserido; é o que o serial no cabeçalho confere.
- **Nada é escrito na máquina antes de o estado inteiro ter sido decodificado.** Arquivo
  truncado devolve `Corrompido` e a máquina segue rodando — testado por
  `estado_recusado_nao_mexe_na_maquina`. Um save state meio aplicado seria pior do que
  nenhum.
- **`serde` não tem `impl` para `[T; N]` com N > 32.** VRAM, tabelas do MDEC, os 64
  registradores do GTE, o scratchpad e o buffer de setor do CD-ROM passam disso. Resolvido
  com `src/serde_grande.rs` (~70 linhas, `serialize_tuple` + visitor), não com uma
  dependência a mais.
- **Campos do `Bus` viraram `pub(crate)`.** Alternativa seria um acessor por subsistema
  (16 métodos) só para o `snapshot.rs` ler. `pub(crate)` mantém a fronteira pública igual.
- Cabeçalho: `PSXRS-ST` + versão `u32` little-endian. O serial mora no corpo, não no
  cabeçalho — quem quiser saber de que jogo é um `.state` chama `serial_de`.
- Teclas: **F5** salva, **F8** carrega, **F6/F7** trocam o slot (0..9), `saves/<serial>-<n>.state`.
- **Medido em máquina de verdade:** BIOS + Crash, 20 M passos → snapshot → +2 M → restaura →
  +2 M ⇒ estado byte a byte idêntico.
