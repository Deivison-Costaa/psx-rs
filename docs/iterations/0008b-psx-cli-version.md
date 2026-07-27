# 0008b — psx-cli --version

- **Data:** 2026-07-27
- **Item do roadmap:** 0.8b (trabalho fora da escada)
- **Objetivo:** suportar flag `--version` no psx-cli.

## Spec consultada

N/A — não há spec de hardware para flag `--version`.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | nenhum | — | — | sem erros |

## Bateria de mutação

3/3 mutantes pegos, 1/1 controle verde:

| # | Mutação | Teste que pegou | Resultado |
|---|---|---|---|
| 1 | `--version` → `-version` (flag errada) | `version_flag_prints_name_and_version` | stdout vazio, `.starts_with("psx-cli ")` falha |
| 2 | `println!` removido | `version_flag_prints_name_and_version` | stdout vazio, `.starts_with("psx-cli ")` falha |
| 3 | `"psx-cli "` → `"psx "` (prefixo errado) | `version_flag_prints_name_and_version` | `got: "psx 0.1.0\n"`, `.starts_with("psx-cli ")` falha |
| C | `let args: Vec<String> = ...` → `let args = ...::collect::<Vec<_>>()` (sintaxe equivalente) | `version_flag_prints_name_and_version` | passa — controle verde |

## Placar antes → depois

Workspace: 8 testes → 9 testes (1 novo: `psx-cli::version_flag_prints_name_and_version`).
Scoreboard: N/A.

## Decisões e notas

- Nenhuma dependência nova (usa `std::env::args()` diretamente).
- `env!("CARGO_PKG_VERSION")` é resolvido em compile-time, sem custo em runtime.
- Código mínimo proposital (R4): apenas `--version`, sem parsing geral de args.
