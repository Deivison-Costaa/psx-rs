<!-- Custo, tokens e duracao NAO entram aqui: sao medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0201 - sio-write32

- **Data:** 2026-08-05
- **Item do roadmap:** Achado 10.48
- **Objetivo:** encaminhar escritas `sw` para MODE, CTRL e BAUD do SIO0.

## Spec consultada

| Fonte | Secao | Arquivo local |
|---|---|---|
| psx-spx | § SIO_MODE (L79) | docs/reference/17-sio.md |
| psx-spx | § SIO_CTRL (L93) | docs/reference/17-sio.md |
| psx-spx | § SIO_BAUD (L128) | docs/reference/17-sio.md |

MODE e CTRL sao registradores R/W de 16 bits e BAUD e o reload de 16 bits. STAT ocupa
`1044h` e e somente leitura. O caminho correto para um `sw` que comece na janela e decompor
os quatro bytes em ordem little-endian e deixar `Sio::write_byte` ignorar os offsets sem
registrador, como ja acontece no caminho de byte.

## O que entrou

- `region_write32` ganhou um braco explicito para `0x1F80_1044..=0x1F80_104F`, antes do
  sumidouro generico.
- O valor e convertido com `to_le_bytes` e encaminhado por quatro chamadas a
  `self.sio.write_byte(phys + i, byte_i)`. Os quatro offsets sao importantes para escritas
  iniciadas em qualquer registrador da janela; `sio.rs` continua sendo a autoridade para
  MODE, CTRL, BAUD e os offsets sem uso.
- `bus_sio_write32.rs` cobre MODE (`1234h`), CTRL (`2A03h`) e BAUD (`9ABCh`) via `write32`
  e `read16`.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | processo | O `.resultado` gerado pela bateria ja seria aceito pela verificacao local sem estar rastreado. | A prova da bateria precisa existir no repositorio para que o clone limpo da CI possa conferi-la. | A primeira execucao de `cargo test --all` falhou em `mutation_battery`; o erro pediu `git add docs/mutantes/0201-sio-write32.resultado`. |

O teste de integracao foi executado antes do fix e falhou por assercao: MODE permaneceu em
zero depois do `write32`. Depois do braco explicito, passou sem alterar `region_write_byte` ou
`sio.rs`.

## Bateria de mutacao

Placar da bateria: **6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente** -
`docs/mutantes/0201-sio-write32.mut`.

| # | Mutacao | Teste que pegou |
|---|---|---|
| m1 | conversao big-endian troca a ordem dos bytes | `write32_em_joy_mode_ctrl_e_baud_entrega_os_dois_bytes` |
| m2 | byte baixo usa o segundo byte | `write32_em_joy_mode_ctrl_e_baud_entrega_os_dois_bytes` |
| m3 | byte alto usa o byte baixo | `write32_em_joy_mode_ctrl_e_baud_entrega_os_dois_bytes` |
| m4 | terceiro byte volta ao offset anterior | `write32_em_joy_mode_ctrl_e_baud_entrega_os_dois_bytes` |
| m5 | quarto byte volta ao offset anterior | `write32_em_joy_mode_ctrl_e_baud_entrega_os_dois_bytes` |
| m6 | faixa explicita deixa de cobrir a janela SIO | `write32_em_joy_mode_ctrl_e_baud_entrega_os_dois_bytes` |
| c1 | decomposicao explicita equivalente a `to_le_bytes` | verde |
| c2 | quebra de linha na chamada nao altera o acesso | verde |

## Placar antes -> depois

- Workspace: 1242 -> 1243 testes (1 teste novo em `bus_sio_write32.rs`).
- `cargo fmt --all -- --check`: verde.
- `cargo clippy --all-targets -- -D warnings`: verde.
- `cargo test --all`: todos os testes funcionais e meta-testes passaram; o único teste em
  falha foi `status_handoff::placar_do_status_bate_com_a_contagem_de_testes`, que detecta o
  1242 antigo do `STATUS.md` contra 1243 testes. O arquivo foi deliberadamente preservado
  sem alteração conforme o handoff do orquestrador.

## Revisao cruzada (orquestrador)

Pendente: revisao adversarial do orquestrador.

## Decisoes e notas

- O teste le MODE, CTRL e BAUD com `read16`, porque o defeito desta iteracao e o caminho de
  escrita; a leitura de `read32` em MODE continua sendo uma questao separada do decodificador
  de leitura e nao foi ampliada por R4.
- O valor de CTRL evita os bits de acao `Acknowledge` e `Reset`, que `Sio::update_ctrl`
  consome por semantica de hardware em vez de manter no registrador.
- `region_write_byte` e `sio.rs` nao mudaram. `STATUS.md` tambem nao mudou, conforme o
  handoff do lote do orquestrador.
