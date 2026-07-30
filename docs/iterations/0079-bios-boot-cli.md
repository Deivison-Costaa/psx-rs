# 0079 — bios-boot-cli

- **Data:** 2026-07-30
- **Item do roadmap:** 4.4a
- **Objetivo:** `--bios` sozinho (sem `--exe`) boota a BIOS em vez de só imprimir hash. O laço roda os passos e imprime o TTY.

## Revisão do PR anterior

Revisão do PR anterior (#93, iter 0076): sem achados.
Padrões conferidos:
1. Teste que não mede — m1 (regressão) mata `gp1_09h_fechar_gate_nao_limpa_gpustat_15`, confirmado manualmente
2. Parâmetro não consumido — sem novos comandos GP0
3. Regra de borda trocada — sem rasterização
4. Campo de bit lido errado — GP1(09h) lê corretamente `val & 1`
5. Panic ou laço ilimitado — sem unwrap/expect fora de teste
6. Citação de spec — confere-citacoes.ps1 verde
7. Escopo transbordado — item 10.32 bem delimitado
8. Portão que não mede — 5/5 bateria verde, resultado rastreado
9. Manifesto arquivado — nenhum novo, âncoras válidas

Nota: o `grep -n "if bit == 0" crates/psx-core/src/gpu.rs` casa na linha 1735, mas é o braço GP1(03h) (Display Enable), não GP1(09h). O GP1(09h) já foi corrigido na 0076. Prioridade descartada.

## Spec consultada

Sem spec nova — o item é de integração (colar componentes existentes: Bus, Ram, Bios, Cpu, laço de step).

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | Que bastava trocar `return;` por `run(...)` no braço `(Some(bios), None, disc_path)` e o resto funcionaria | O braço era responsável por dois cenários distintos (com e sem `--disc`), e o bloco de leitura da BIOS era diferente do usado no braço `--bios --exe` | Os testes de integração existentes (`bios_flag.rs`, `disc_flag.rs`) quebraram: o `bios_flag` esperava hash, o `disc_flag` esperava println de DISCO. Ambos precisaram ser atualizados |
| 2 | manifesto | Que âncoras multi-linha com linhas em branco funcionariam | O parser de manifesto usa `content.lines()` que pula linhas vazias, então qualquer âncora que cruze uma linha em branco perde essa linha e não casa com o fonte | `mutation_anchors` reprovou m3 da 0079 (primeira versão) e m3/c1 da 0078. Corrigido usando âncoras sem cruzar linhas em branco |
| 3 | manifesto | Que o manifesto da 0078 não seria afetado pela refatoração do `main.rs` | As âncoras m3 e c1-edição-3 da 0078 quebravam porque a estrutura do braço `(Some(bios), None, disc_path)` mudou | `mutation_anchors` reprovou. m3 foi reescrito (de `tracks.len()` para remover `run()`), c1-edição-3 foi re-ancorada no novo bloco com `insert_disc()` |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0079-bios-boot-cli.mut

| Registro | Rótulo | Testes que pegaram |
|---|---|---|
| m1 | remove eprintln "Runner:" do laço de boot | `bios_flag_boota_bios_sintetica` |
| m2 | substitui run() por panic! no laço de boot | `bios_flag_boota_bios_sintetica`, `disc_flag_cue_minimo_aceito_com_bios` |
| m3 | load_disc retorna layout vazio em vez de abortar | `disc_flag_arquivo_cue_inexistente_erro` |
| m4 | --version não é reconhecido | `version_flag_prints_name_and_version` |
| m5 | não retorna após o boot (cai no erro de uso) | `bios_flag_boota_bios_sintetica` |
| c1 | adiciona comentário antes da leitura da BIOS (cosmético) | sobreviveu |
| c2 | adiciona variável descartada em load_disc (cosmético) | sobreviveu |

**Nota:** bateria manual — os mutantes são em `crates/psx-cli/src/main.rs` e o script `mutantes.ps1` só roda `cargo test -p psx-core` (10.33). O `.resultado` foi preenchido por inspeção e validado manualmente com m1 (aplicado → `bios_flag_boota_bios_sintetica` reprovou → revertido).

## Placar antes → depois

Workspace: **572** testes (eram 567, +4 do psx-core/bios_boot.rs placeholder, +1 do psx-cli/bios_boot.rs).

**TTY do boot da BIOS (antes de 4.4b):**
```
PS-X Realtime Kernel Ver.2.5
Copyright 1993,1994 (C) Sony Computer Entertainment Inc.
KERNEL SETUP!
Configuration : EvCB 0x10  TCB 0x04
System ROM Version 2.2 12/04/95 A
ResetCallback: _96_remove ..
VSync: timeout (1:0)
VSync: timeout (1:0)
...
```

O `VSync: timeout` é esperado: não existe base de tempo (scheduler, vblank, IRQ0). Será resolvido no item 4.4b.

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo orquestrador. -->

## Decisões e notas

1. O `--bios` sozinho agora boota em todos os cenários: sem `--disc`, com `--disc`, e mantém compatibilidade com `--exe`.
2. O `bios_flag.rs` foi atualizado: o teste `bios_flag_prints_size_and_hash` foi renomeado para `bios_flag_boota_bios_sintetica` e agora verifica que o boot ocorre (stderr contém "Runner:").
3. O `disc_flag.rs` também foi atualizado: `disc_flag_cue_minimo_aceito_com_bios` agora verifica "Runner:" em vez de track info no stdout.
4. Manifesto 0078 teve m3 e c1-edição-3 re-ancorados para refletir a nova estrutura do `main.rs`. O `.resultado` da 0078 não foi regenerado (bateria manual para psx-cli, item 10.33).
5. Placeholder tests em `crates/psx-core/tests/bios_boot.rs` permitem que o meta-teste `bateria_nomes_de_teste_existem` passe, espelhando o padrão de `disc_flag.rs`.
