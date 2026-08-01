<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0131 — espera-tela-sce

- **Data:** 2026-08-01
- **Item do roadmap:** 4.4z
- **Objetivo:** Descobrir o que o shell espera para sair da tela SCE (candidatos do handoff:
  SPU, timer, joypad).

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Setloc - Command 02h (L787): amm/ass/asect são MSF ABSOLUTOS do disco | docs/reference/06-cdrom.md |
| psx-spx | 1ª trilha da 1ª sessão começa em 00:02:00 (docs/reference/06-cdrom.md L850) | docs/reference/06-cdrom.md |

## Instrumentação

Flag `--sample-pcs inicio:fim:passo` no `psx-cli`: amostra `cpu.pc` na janela de steps
pedida. Zero mudança no psx-core.

## Cadeia de medições (cada passo motivado pelo anterior)

1. **Amostra macro 95M–195M (passo 200k):** 82% das amostras num único PC `0x800422DC` —
   suspeita de aliasing (passo composto × período do laço).
2. **Amostra com passo PRIMO 30011 (120M–135M):** distribuição limpa — o shell gasta ~99%
   do tempo num laço de 10 instruções: `0x8004205C..64` + `0x800422C8..E0`.
3. **Dump + disasm do laço:** `lw $v0, 0($a0); srl $v0,24` → compara com `$t4=0x20` e
   `$t5=0x30` (triângulo flat/gouraud GP0); sem match, `$a0+=0x10`, `$t1++`,
   `bne $t1,$s1`. É o parser de primitivas do modelo 3D do logo.
4. **Trace dos caminhos de match (101M steps): ZERO matches; tabela de saída `0x80138EE8`
   toda zerada.** O laço nunca acha um triângulo.
5. **Trace do fecho do laço (primeiras iterações, step ~112,95M):** `a0=0xA0010028`
   (RAM física 0x10028 — a área onde a BIOS carrega a licença/logo do disco);
   `s1=0x2E0E1E1E` — o "número de primitivas" lido do cabeçalho é LIXO (~773 milhões de
   iterações; por isso o spin é infinito dentro de UMA chamada — o prólogo da função nunca
   re-executa em 101M steps).
6. **RAM × disco:** os bytes em `0x80010000` existem no `.bin` no **setor 155**, offset
   intra-setor 0x18 (correto para Form1). O TMD do logo PlayStation de verdade está no
   **setor 5** (`41 00 00 00...` = ID de TMD; setor 6 contém os primitivos `0x20`).
   **155 − 5 = 150 = o pregap de 2 segundos (MSF 00:02:00).**

## Veredito

**Nenhum dos candidatos do handoff (SPU/timer/joypad).** O shell não espera hardware: espera
um DADO válido. Ele varre o TMD do logo PlayStation que a BIOS carregou do disco para
`0x80010000`, e o conteúdo é lixo porque **todos os reads de CD entregam o setor N+150**:
`read_sector_from_disc` (`crates/psx-core/src/cdrom.rs:518-520`) converte MSF absoluto em
setor (`bcd_to_int(mm)*60*75 + ss*75 + ff`) e indexa o `.bin` com `abs_sector * 2352` — mas
o `.bin` começa em MSF 00:02:00 (docs/reference/06-cdrom.md L850), então o offset correto é
`(abs_sector - 150) * 2352`. Correção é o item 4.4aa (uma linha + teste com golden do setor
5 = TMD `41h`).

Consistência com o histórico: os ~86 setores lidos na 0129 eram a licença/logo (com
conteúdo do lugar errado); a montagem do filesystem ISO ainda NEM aconteceu — o logo vem
antes. O off-by-150 explica de uma vez o spin, o congelamento da tela SCE (0130) e
possivelmente o "sem disco" antigo do GetID (4.4q).

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | medição (aliasing) | Que amostragem com passo 200k dá um histograma fiel | n/a — período do laço (10 instr) divide o passo composto; 82% das amostras caíram no MESMO PC | Reamostragem com passo primo 30011 mudou o histograma por completo |
| 2 | API-Rust (ferramenta) | Que `Get-Content \| Set-Content` preserva o line-ending do arquivo | n/a — Set-Content grava CRLF; o portão de âncoras compara linha crua e TODA âncora passou a falhar | Cascata de falsos "âncora envelhecida"; quase arquivei o manifesto 0128 por engano (desfeito); guarda: renormalizar com `git checkout` e nunca reescrever fonte via Set-Content |
| 3 | endereçamento | (herdado desde a 4.3c) Que o `.bin` é indexado pelo MSF absoluto | § Setloc (L787): MSF é absoluto do DISCO; a trilha 1 começa em 00:02:00 (docs/reference/06-cdrom.md L850) — o arquivo começa 150 setores DEPOIS do zero absoluto | Bytes da RAM localizados no `.bin` em setor N+150 do esperado |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0131-espera-tela-sce.mut

Bateria MANUAL (invariante 29) — alvo `crates/psx-cli/src/main.rs`, assassino
`cargo test -p psx-cli --test espera_tela_sce` (aplicado → rodado → revertido, um a um):

| id | mutação | resultado |
|---|---|---|
| m1 | janela exclui o início (`steps > start`) | MORREU (0.05s) |
| m2 | fase do passo deslocada (`% stride == 1`) | MORREU (0.05s) |
| m3 | rótulo `sample` → `sampled` | MORREU (0.05s) |
| m4 | amostra `regs[2]` em vez do PC | MORREU (0.06s) |
| m5 | parser troca início/fim (janela vazia) | MORREU (0.05s) |
| c1 | comentário na declaração | SOBREVIVEU (esperado) |
| c2 | comentário antes do bloco | SOBREVIVEU (esperado) |

Efeito colateral do parâmetro novo em `run()`: as âncoras de manifestos antigos de CLI
envelheceram. **0078/0079/0125/0128 arquivados** (registro afetado nomeado em cada um);
**0129 teve a âncora c1 atualizada e a bateria RE-EXECUTADA inteira: 5/5 + 2/2** (resultado
regravado). Stub do portão em `crates/psx-core/tests/espera_tela_sce.rs`.

## Placar antes → depois

842 → 844 testes no workspace (o teste da iteração no psx-cli + o stub do portão no psx-core).

## Revisão cruzada (orquestrador)

- **A1 (menor):** `espera_tela_sce.rs` não remove o `.psexe` sintético ao fim (os testes
  irmãos removem). Artefato de 4 KB órfão em `tests/bins/` — sem efeito em medição; limpar
  na próxima passada pelo arquivo.
- Verificações: (a) a identificação "setor 5 = TMD" bate com o formato (ID 0x00000041 no
  primeiro word do user data); (b) o passo primo foi conferido contra o mesmo fenômeno em
  duas janelas distintas; (c) a re-execução da bateria 0129 seguiu a regra do portão
  ("atualizou âncora → rode a bateria de novo"), não foi placar herdado.

## Decisões e notas

- A janela fina 100M–100,5M havia pego OUTRA fase (rotina de retângulos `0x8004Axxx`) — o
  spin do TMD só começa em ~112,94M. Amostrar mais de uma janela antes de concluir.
- O disasm foi feito com um decodificador MIPS descartável no scratchpad (não commitado).
- Invariante 33 criada: a saída da tela SCE é dirigida por DADO (TMD do logo), não por
  evento de hardware; e todo read de CD entregava N+150.
