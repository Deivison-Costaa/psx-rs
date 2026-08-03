# ROADMAP

Cada item = **1 iteração = 1 PR** (commits test→feat→docs). Uma linha por item, sem prosa —
narrativa mora em `docs/iterations/NNNN-*.md`. Trabalho fora da escada ganha sufixo (`0012b`).
Teto de tamanho imposto por `roadmap_size.rs`.

Itens fechados saem daqui para `docs/ROADMAP-fechado.md`; a escada mantém só o que FALTA.
**Defeito achado por medição não entra aqui** — vai para `docs/achados.md`, numerado pela
iteração que o achou (`NNNN.k`). A escada é o que construir; os achados são o que consertar.
Regra imposta por `roadmap_arquivo.rs`.

## M4 — CDROM
- [ ] 4.4b Crash Bandicoot: chegar ao primeiro nivel (o Rayman chegou na 0184)
- [ ] 4.5 1o frame: rollback do init do LIBSN; poll orfao do TMR2 (diag 0137-0147)

## M5 — GTE
- [ ] 5.4b NCS/NCT/NCCS/NCCT
- [ ] 5.4c NCDS/NCDT/CC/CDP
- [ ] 5.4d DCPL/DPCS/DPCT/INTPL
- [ ] 5.5 Flags de saturação/overflow completos
- [ ] 5.6 Amidog psxtest_gte no scoreboard → jogo 3D jogável

## M6 — SIO: controle e memory card
- [ ] 6.3 Memory card (.mcd)

## M7 — SPU
- [ ] 7.1 Regs de voz + ADPCM
- [ ] 7.2 Pitch/ADSR/volume + mixer
- [ ] 7.3 Saída cpal + ring buffer
- [ ] 7.4 Reverb + noise + CD-DA/XA

## M9 — App desktop
- [ ] 9.1 Biblioteca: scan BIN/CUE, título/serial/região, lista
- [ ] 9.2 Snapshot do core (serde) → save states F5/F8 + slots
- [ ] 9.3 Memory cards automáticos por serial + tela de saves
- [ ] 9.4 Controles PS/Xbox (gilrs) + tela de mapeamento + perfis
- [ ] 9.5 Tela de configurações (BIOS, vídeo, áudio, pasta) em TOML
- [ ] 9.6 Fast-forward + recentes + tempo de jogo

## M11 — Apresentação (incremental desde o M1)
- [ ] 11.2 Gráficos de metricas.csv + scoreboard-data
- [ ] 11.3 Roteiro de demo
