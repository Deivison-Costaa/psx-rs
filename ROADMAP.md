# ROADMAP

Cada item = **1 iteração = 1 PR** (commits test→feat→docs). Uma linha por item, sem prosa —
narrativa mora em `docs/iterations/NNNN-*.md`. Trabalho fora da escada ganha sufixo (`0012b`).
Teto de tamanho imposto por `roadmap_size.rs`.

Itens fechados saem daqui para `docs/ROADMAP-fechado.md`; a escada mantém só o que FALTA.
**Defeito achado por medição não entra aqui** — vai para `docs/achados.md`, numerado pela
iteração que o achou (`NNNN.k`). A escada é o que construir; os achados são o que consertar.
Regra imposta por `roadmap_arquivo.rs`.

## M9 — App desktop
- [x] 9.1 Biblioteca: scan BIN/CUE, título/serial/região, lista (iter 0193)
- [ ] 9.2 Snapshot do core (serde) → save states F5/F8 + slots
- [ ] 9.3 Memory cards automáticos por serial + tela de saves
- [ ] 9.4 Controles PS/Xbox (gilrs) + tela de mapeamento + perfis
- [ ] 9.5 Tela de configurações (BIOS, vídeo, áudio, pasta) em TOML
- [ ] 9.6 Fast-forward + recentes + tempo de jogo

## M11 — Apresentação (incremental desde o M1)
- [ ] 11.2 Gráficos de metricas.csv + scoreboard-data
- [ ] 11.3 Roteiro de demo
