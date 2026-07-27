# 0009 — carregamento de BIOS

- **Data:** 2026-07-27
- **Item do roadmap:** 0.9
- **Objetivo:** tipo `Bios` com validação de tamanho (exatos 512 KiB) e acesso `read32` little-endian; flag `--bios <path>` no psx-cli que lê o arquivo e exibe tamanho + SHA-256.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | Memory Map — BIOS ROM (KSEG1 0xBFC00000, 512K) | docs/reference/01-memory-map.md:34 |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | Esqueci `#[derive(Debug)]` no `Bios`, erro de compilação no `unwrap_err()` dos testes. | `unwrap_err` requer `Debug` no tipo de erro e no valor Ok. | Compilador apontou. |

## Bateria de mutação

Placar: **3/3 mutantes pegos, 1/1 controles verdes**.

| Mutação | Efeito | Teste que pegou |
|---|---|---|
| `!=` trocado por `==` na validação de tamanho | BIOS de tamanho errado passa; BIOS correto falha | `bios_from_bytes_muito_curto`, `bios_from_bytes_muito_longo`, `bios_from_bytes_vazio`, `bios_from_bytes_ok` |
| `from_le_bytes` trocado por `from_be_bytes` | read32 devolve big-endian | `bios_read32_little_endian`, `bios_read32_primeiro_word`, `bios_read32_ultimo_word` |
| offset deslocado em +1 (começar em `[offset+1]`) | read32 lê bytes errados + panic no último word | `bios_read32_little_endian`, `bios_read32_primeiro_word`, `bios_read32_offset_dentro_limite`, `bios_read32_ultimo_word` |
| _Controle_: renomear `data` → `bytes` em `from_bytes` | sem efeito semântico | todos verdes |

## Placar antes → depois

Workspace: 9 → **16** testes (+7: 8 bus_bios - 1 version que já existia). 0 falhas, 0 warnings.

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->

## Decisões e notas

- `Bios` recebe `Vec<u8>` por valor, sem I/O (R3). O CLI faz a leitura do arquivo.
- SHA-256 não entra no core (armadilha do STATUS). Fica exclusivamente no CLI.
- `BiosError::WrongSize` expõe `got` e `expected` para mensagens de erro informativas.
- BIOS exception/panic em `read32` com offset inválido: aceitável por ora (item 1.1 trará acesso roteado pelo bus com bounds check).

### Achados

SEVERIDADE: média / PROCESSO / O QUE ESTAVA: sem commit test(...) separado (testes entraram
junto com os feat), um commit em inglês, e um `chore:` sem escopo que o commit-lint reprovou
— primeira captura real do guard. CORRIGIDO: histórico da branch reestruturado pelo
orquestrador em test → feat(bus) → feat(cli) → docs, mensagens em português.

SEVERIDADE: baixa / crates/psx-core/src/bus.rs / O QUE ESTÁ ESCRITO: read32 indexa direto e
panica com offset > 0x7FFFC. O QUE A SPEC DIZ: acessos à região da BIOS são mascarados pelo
barramento (01-memory-map.md § Memory Mirrors). COMO PROVAR: read32(0x80000) → panic.
DECISÃO: aceito por ora — a máscara de endereço é exatamente o item 1.1 (Bus), registrada
como armadilha no handoff.

Bateria conferida: mutações e controle batem com os testes do diff — genuína.
