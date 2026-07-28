<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0034 — handoff-2-1-real

- **Data:** 2026-07-28
- **Item do roadmap:** nenhum (passo zero do 2.1)
- **Objetivo:** escrever o handoff do 2.1, que ainda não existia apesar de duas tentativas.

## Spec consultada

Todas as linhas foram conferidas com `grep -n` nesta iteração, e são **absolutas do arquivo**
(não relativas à marca `CORPO:`).

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | Portas `1F801810h`/`1F801814h`, leitura ≠ escrita | `03-gpu.md` L144-147 |
| psx-spx | Tabela de bits do GPUSTAT | `03-gpu.md` L1002-1032 |
| psx-spx | GP1(00h) Reset — GPUSTAT = `14802000h` | `03-gpu.md` L747-763 |
| psx-spx | GP1(02h/03h/04h/08h) | `03-gpu.md` L773, L779, L789, L885-893 |
| psx-spx | GP0(E1h) Draw Mode, GP0(E6h) Mask Bit | `03-gpu.md` L492, L578 |
| psx-spx | GP0(00h) NOP e mirrors | `03-gpu.md` L721, L734 |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que era verdade | Como foi pego |
|---|---|---|---|---|
| 1 | processo | Que o handoff do 2.1 existia, porque a "Próxima tarefa" apontava para ele | A 0033 substituiu o handoff por **um ponteiro**: *"Handoff em `docs/iterations/0032-handoff-2-1.md` (revisado e aprovado)"*. Só que a 0032 é o registro de **correções** do handoff velho e traz o handoff do **1.14**, não o do 2.1 | Abri o STATUS antes de despachar e segui o ponteiro |

Vale registrar o mecanismo, porque é novo: as iterações anteriores erravam o *conteúdo* do
handoff (citação inventada, número não medido). Esta errou a *existência* dele — trocou o
handoff por uma referência que parecia responsável ("revisado e aprovado") e que não levava a
lugar nenhum. Um despacho em cima disso mandaria o trabalhador ler um doc de correções
esperando encontrar escopo e testes de aceitação.

## Bateria de mutação

Não se aplica: sem mudança em `crates/`.

## Placar antes → depois

258 → 258 testes (inalterado).

## Revisão cruzada (orquestrador)

Iteração do próprio orquestrador.

## Decisões e notas

1. **O handoff usa o valor de reset pronto da spec (`14802000h`) como golden value** em vez de
   mandar montar o GPUSTAT bit a bit. Montagem bit a bit é onde a versão de 0031 inventou
   "bits de versão da GPU": quando a spec já dá o número, conferir contra o número é mais
   barato e não admite invenção.
2. **A6 é o teste que fecha o item, e ele é de fora para dentro:** `cop.exe` tem que imprimir
   `pass - testCop0Disabled` / `pass - testCop0Enabled` **sem** o stub temporário de GPUSTAT.
   Hoje isso só sai com o andaime aplicado à mão (medido na revisão da 0033). Amarrar o
   critério de sucesso a um EXE de hardware real, e não a um teste sintético que nós mesmos
   escrevemos, é a diferença entre medir o emulador e medir a nossa expectativa dele.
3. **Armadilha 5 (comandos GP0 multi-palavra) é a que eu esperaria falhar primeiro.** Se o
   consumo de parâmetros não for respeitado, palavras de dados viram comandos e o GPUSTAT muda
   sozinho — um defeito que passa despercebido em teste sintético e aparece como saída
   corrompida no EXE real. O handoff dá a alternativa explícita de restringir o escopo e
   documentar o que ficou de fora, em vez de engolir comando em silêncio.
