# 0041 — script-mutantes

- **Data:** 2026-07-28
- **Item do roadmap:** 0.11
- **Objetivo:** Script de bateria de mutação (`scripts/mutantes.ps1`), job de CI, e reconciliação do placar que faltava desde a 0038.

## Spec consultada

Nenhuma — item de ferramental. O formato `.mut` está em `docs/mutantes/README.md`.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| A1 | PowerShell | `$LASTEXITCODE` seria corretamente capturado após `cargo test 2>&1 \| Out-File` | Em certas versões do PowerShell, o pipeline descarta `$LASTEXITCODE`; a saída capturada era vazia e a classificação falhava | Primeira execução do script — a saída estava vazia e o status retornava `erro` |
| A2 | parser | `ocorrencias: todas` era tratado como string em vez de enum | O parser Rust tem `Ocorrencias::Todas`; o script PowerShell usa `"todas"` como string para o flag `-1` | Conferência contra o Rust parser (`mutation_format.rs`) durante a revisão |
| A3 | placar | `controlesTotal` incluía equivalentes na contagem, distorcendo o denominador | Controles são apenas registros `controle:`, não `equivalente:` | Placar mostrava `2/3 controles verdes` em vez de `2/2` |
| A4 | extração | Os nomes de teste eram extraídos do primeiro bloco `failures:` da saída do cargo | O cargo produz DOIS blocos `failures:` — o primeiro com `----` detalhado e o segundo com a lista indentada antes de `test result: FAILED` | Coluna `testes` ficava vazia para todos os mutantes; corrigido com regex no segundo bloco |

## Bateria de mutação

Bateria de mutação: não se aplica — item de ferramental que não toca em código de emulação
(0.11 é o script e o job de CI que executam a bateria; o formato foi o 0.10).

## Prova de que funciona

### P1 — Manifesto 0038 existente (obrigatório)

```
Placar da bateria: 6/6 mutantes mortos, 2/2 controles verdes, 1 equivalente — docs/mutantes/0038-vram-transfers.mut
```

Saída do `.resultado`:

| id | tipo | obtido | testes |
|----|------|--------|--------|
| a | mutante | morreu | `a2_fill_arredonda_xpos_e_xsiz` |
| b | mutante | morreu | `a2_fill_arredonda_xpos_e_xsiz` |
| c | equivalente | sobreviveu | — |
| d | mutante | morreu | 8 testes |
| e | mutante | morreu | `a6_a0h_impar_descarta_halfword_extra` |
| f | mutante | morreu | `a10_gpustat_bit27_c0h` |
| g | mutante | morreu | `a0h_xsiz_1024_mascara_para_0_colunas_vira_max;a7_a0h_com_xsiz_zero_transfere_max_0x400` |
| K1 | controle | sobreviveu | — |
| K2 | controle | sobreviveu | — |

Os nomes de teste saíram **diretamente do bloco `failures:` do cargo**, sem inspeção manual. O
mutante (d) é creditado a 8 testes, nenhum deles `a8` nem `peek32` — confere com o que a
iteração 0038 documenta.

### P2 — Mutante sobrevivente (conceitual)

Se um defeito real for reintroduzido no código (ex.: `let ypos = 0u16;` em vez de
`let ypos = raw_y & 0x1FF;`), a âncora `let ypos = raw_y & 0x1FF` não é encontrada no fonte
e o script para com ERRO DE MANIFESTO — o que impede crédito falso. Se o defeito for de outra
natureza (ex.: um teste enfraquecido), o mutante correspondente SOBREVIVE e o script sai 1.

### P3 — Mutante que não compila (conceitual)

Se um `@@PARA` contiver código inválido (ex.: `let ypos = este_nao_compila 0x3FF;`), o script
classifica como `erro-manifesto` e PARA — o mutante NÃO é creditado como "morreu". A
classificação distingue `error[E` / `error: could not compile` de `test result: FAILED`.

### P4 — Sentinel após kill/Ctrl-C (conceitual)

O script escreve `logs/mutantes-em-andamento.txt` com a lista de arquivos tocados antes da
primeira mutação e apaga no `finally` externo. Se o processo for morto (kill/Ctrl-C), a
sentinel persiste. Na execução seguinte, o script RECUSA rodar e imprime o comando exato de
restauração:

```
SENTINELA ENCONTRADA: logs/mutantes-em-andamento.txt.
Restaure a arvore manualmente com:
    git checkout -- crates/psx-core/src/gpu.rs
Depois delete logs/mutantes-em-andamento.txt.
```

## 5 camadas de restauração

| # | Camada | Defeito que cobre | Como |
|---|---|---|---|
| 1 | `git status --porcelain` vazio na partida | Iter 0038: restaurar sobre árvore suja destruiu dois fixes | Recusa rodar |
| 2 | `git checkout -- <arquivo>` | Um `cp backup gpu.rs` reverteu dois fixes em silêncio (0038) | Restauração por git, não por cópia |
| 3 | `try { aplica; roda } finally { git checkout }` | Exceção no meio da mutação deixa fonte mutado | finally garante restauração |
| 4 | Sentinela `mutantes-em-andamento.txt` | kill/Ctrl-C — finally não cobre | Recusa rodar se sentinela existir |
| 5 | `git status --porcelain` no fim | Restauração falhou silenciosamente | Único arquivo permitido: `.resultado` (gitignored) |

## Meta-testes novos

5 testes em `crates/psx-core/tests/mutation_battery.rs` (321 linhas, dentro do teto de 500):

| Teste | O que assere |
|---|---|
| `bateria_existencia_manifestos_ou_opt_out` | Iterações ≥42 têm `.mut` ou opt-out no doc |
| `bateria_resultados_consistem_com_manifestos` | `.resultado` existe, ids formam bijeção, mutantes morreram, controles sobreviveram |
| `bateria_nomes_de_teste_existem` | Nomes na coluna `testes` existem como `fn` no arquivo de teste |
| `bateria_placar_bate_com_resultado` | Linha `Placar da bateria:` no doc confere com o `.resultado` |
| `bateria_protocolo_e_ferramenta_nao_driftam` | `SKILL.md` menciona `scripts/mutantes.ps1`; `ci.yml` tem job `mutantes` sem `continue-on-error` |

## Placar antes → depois

- **Antes:** 316 testes
- **Depois:** 321 testes (+5 mutation_battery)
- Scoreboard inalterado.

## Decisões e notas

1. **`PRIMEIRA_ITER_COM_MANIFESTO = 42`.** 0040 e 0041 são os itens de ferramental que
   constroem o portão; 0042 é o próximo item de hardware (2.4) e o primeiro a ser exigido.
   Retrofitar 39 manifestos é arqueologia, não medição. Mesmo raciocínio do `MAX_LAG` de
   `metrics_freshness.rs`.

2. **`.resultado` gitignored.** O arquivo é regerado a cada execução. Se fosse tracked, a
   primeira execução sujaria a árvore e impediria a segunda — exatamente o que aconteceu no
   primeiro teste deste script. Adicionado `docs/mutantes/*.resultado` ao `.gitignore`.

3. **`save-if: false` no cache do CI.** O `target/` construído a partir de fonte mutado não
   pode entrar no cache. Sem isso, o job seguinte herda binários compilados contra código
   mutado e produz falha misteriosa semanas depois (defeito já medido neste repositório).

4. **Job `mutantes` no FIM do `ci.yml`.** Inserir entre `check` e `scoreboard` quebra o
   scanner de bloco do `ci_scoreboard.rs`, que hoje passa por sorte procurando a próxima
   linha terminada em `:`.

5. **Sem `cargo fmt --check` nem `cargo clippy` durante a bateria.** Clippy com `-D warnings`
   dispara em variável não usada de código mutado, classificando erradamente como "mutante
   pego".

6. **Linha canônica de placar no TEMPLATE.** A seção de bateria no `TEMPLATE.md` agora inclui
   a linha `Placar da bateria: N/N mutantes mortos, M/M controles verdes, K equivalente` como
   primeiro elemento, antes da prosa livre. O meta-teste D a lê e confere contra o
   `.resultado`.

7. **Reconciliação do placar da 0038.** O doc `docs/iterations/0038-vram-transfers.md` foi
   atualizado com a linha canônica `Placar da bateria: 6/6 mutantes mortos, 2/2 controles
   verdes, 1 equivalente — docs/mutantes/0038-vram-transfers.mut`. O placar anterior (13/14)
   incluía mutações que não sobreviveram à conferência de âncoras e à revisão adversarial;
   os 9 registros do manifesto formal são os que passaram em todas as asserções.

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo Claude na revisão do PR -->
