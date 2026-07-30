# ROADMAP — marcos fechados

Historico. Marco que fecha 100% sai de `ROADMAP.md` e vem para ca, para o teto de 10 KB
da escada valer so para o que FALTA. Narrativa de cada item continua em
`docs/iterations/NNNN-*.md`. Regra imposta por `roadmap_arquivo.rs`.

## M0 — Infra e processo
- [x] 0.1 Repo, merge-commit-only, PR template (iter 0001)
- [x] 0.2 Workspace 3 crates + esqueleto (iter 0002)
- [x] 0.3 Meta-testes de processo (7) (iter 0005)
- [x] 0.4 CI check + commit-lint + proteção de branch (iter 0004)
- [x] 0.5 Docs de gestão (iter 0003)
- [x] 0.6 psx-spx fatiado em docs/reference com índice de seções (iter 0006)
- [x] 0.7 fetch de EXEs de teste + scoreboard esqueleto (iter 0007)
- [x] 0.8 Orquestração opencode/DeepSeek + smoke test (iter 0008/0008b)
- [x] 0.9 Carregamento de BIOS com validação de hash (1ª iteração do trabalhador) (iter 0009)
- [x] 0.10 Formato de manifesto de mutação + meta-teste (iter 0040)
- [x] 0.11 scripts/mutantes.ps1 + job de CI + reconciliação do placar (iter 0041)
- [x] 0.12 Verificador de citações de spec (iter 0043)

## M1 — CPU R3000A até o BIOS falar
- [x] 1.1 Scheduler + bus (KUSEG/KSEG0/KSEG1), RAM 2MB, BIOS ROM (iter 0010)
- [x] 1.2 Fetch/decode + LUI/ORI/SW (iter 0011)
- [x] 1.3 ALU: ADDU/SUBU/AND/OR/XOR/NOR/SLT/SLTU + imediatos (iter 0012)
- [x] 1.3b Shifts SLL/SRL/SRA/SLLV/SRLV/SRAV (fatiado de 1.3 na revisão da 0012) (iter 0013)
- [x] 1.4 Loads/stores + load delay slot (iter 0014)
- [x] 1.5 Branches/jumps + branch delay slot (iter 0015)
- [x] 1.6 MULT/MULTU/DIV/DIVU + HI/LO com stalls (iter 0016)
- [x] 1.7 LWL/LWR/SWL/SWR (iter 0018)
- [x] 1.8a COP0: SR/CAUSE/EPC/BadVaddr/PRID + MTC0/MFC0 + RFE (iter 0020)
- [x] 1.8b Mecanismo de exceção: overflow, syscall, break, AdEL/AdES, bit BD (iter 0021)
- [x] 1.9 Cache isolation + scratchpad + memory control stubs (iter 0022)
- [x] 1.10 Hook de TTY (A0h/B0h) → BIOS imprimindo no console (iter 0025)
- [x] 1.11 Sideload de PS-EXE no psx-cli + Amidog psxtest_cpu no scoreboard (iter 0027)
- [x] 1.11b Hook de printf A(3Fh) com expansão de % → Amidog imprimindo no TTY (iter 0029)
- [x] 1.11c printf: flags de largura e zero-pad (iter 0087)
- [x] 1.12 CI: job scoreboard ligado (iter 0031)
- [x] 1.13 Veredito no scoreboard: parse de saida das suites (iter 0036)
- [x] 1.14 Opcode nao implementado gera excecao (RI 0Ah / CpU 0Bh) em vez de panic (iter 0033)

## M3 — DMA, IRQ, timers
- [x] 3.1 Interrupt controller (I_STAT/I_MASK) + COP0 (iter 0055)
- [x] 3.2 DMA regs + canal 6 (OTC) (iter 0056)
- [x] 3.3 DMA canal 2 GPU (block + linked-list) (iter 0057)
- [x] 3.4 Timers 0/1/2 — registradores e contagem básica (iter 0058)
- [x] 3.4b Timers — modos de sync Hblank/Vblank (iter 0059)
- [x] 3.4c Timers — fontes de clock Dotclock/Hblank (iter 0060)
- [x] 3.4d Timers — conexão de IRQ4/IRQ5/IRQ6 ao controlador (deferido da 3.4) (iter 0061)
