# psx-rs

Emulador de PlayStation 1 em Rust, construído por agentes de IA: um orquestrador (Claude)
gerencia loops de um trabalhador (opencode/DeepSeek) que escreve o código sob TDD, bateria de
mutação e revisão adversarial. Projeto final da cadeira de Programação com Agentes.

- **Progresso:** [ROADMAP.md](ROADMAP.md) · estado atual em [STATUS.md](STATUS.md)
- **Processo e regras:** [CLAUDE.md](CLAUDE.md) · diário em [docs/orquestracao.md](docs/orquestracao.md)
- **Métricas por iteração (custo/tokens/tempo):** [docs/metricas.csv](docs/metricas.csv)
- **Relatório (rascunho vivo):** [docs/relatorio.md](docs/relatorio.md)

## Estrutura

| Crate | Papel |
|---|---|
| `psx-core` | emulação pura, sem I/O |
| `psx-cli` | runner headless (testes de EXE, scoreboard) |
| `psx-desktop` | app egui: biblioteca de jogos, saves, controles |

## Rodando

Requer uma BIOS de PS1 obtida do seu próprio console (ex.: SCPH1001.BIN) — não incluída.

**Guia de uso completo (jogar, teclas, memory card, áudio): [docs/como-rodar.md](docs/como-rodar.md).**

```
cargo nextest run --workspace   # testes + meta-testes de processo
cargo build --release           # sempre release: debug e ~20x mais lento
./target/release/psx-desktop bios/SCPH1001.BIN cartao.mcd
```
