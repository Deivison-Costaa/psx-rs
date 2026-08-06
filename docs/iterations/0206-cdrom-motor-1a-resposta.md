<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0206 — cdrom-motor-1a-resposta

- **Data:** 2026-08-06
- **Item do roadmap:** 10.55 (achado legado, iteração de origem 0121); 10.56 verificado e
  fechado como desatualizado na mesma revisão
- **Objetivo:** a primeira resposta de um comando de CD-ROM tem que usar o atraso menor
  ("when stopped", 0005cf4h) quando o motor está parado, não sempre o atraso de motor ligado
  (000c4e1h).

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § First Response — tabela "Nop (normal)" vs "Nop (when stopped)" (L2047-2054) | docs/reference/06-cdrom.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a medição mostrou | Como foi pego |
|---|---|---|---|---|
| 1 | escopo | Que só precisava adicionar o parâmetro `motor_on` em `first_response_cycles` e pronto | 2 testes pré-existentes (`cdrom_primeira_resposta.rs`) quebraram: assumiam motor ligado (usavam a constante `000c4e1h`) sem nunca chamar `insert_disc()` — passavam por coincidência porque o motor começava desligado e a aritmética de cada teste não dependia do valor exato, até agora | `cargo test --workspace` — corrigidos chamando `insert_disc()` explicitamente, preservando a intenção original de cada teste (medir o atraso "normal") |
| 2 | achado (10.56) | Que "Result FIFO anterior legível na janela da primeira resposta" (10.56) era um bug ainda presente | Escrevi um teste-sonda (Test 20h, 4 bytes de resposta, lê 2 e deixa 2 pendentes, manda um comando novo sem avançar, lê os 2 restantes) — os bytes antigos continuam corretos e `RSLRRDY` continua alto durante a janela pendente do novo comando. `send_command`/`result_clear` só rodam quando `deliver_first()` de fato entrega a resposta nova, não na escrita do comando (`latch_command`) — o achado já estava corrigido, provavelmente como efeito colateral de alguma correção posterior à iteração 0121 que não atualizou `achados.md` | Teste-sonda descartado após confirmar; achado fechado como desatualizado, sem mudança de código |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente —
`docs/mutantes/0206-cdrom-motor-1a-resposta.mut`.

- m1 (guarda do motor invertida): morto.
- m2 (sempre atraso de motor ligado): morto.
- m3 (sempre atraso de motor parado): morto.
- m4 (constante do motor parado com off-by-one): morto.
- m5 (`bus.rs` passa motor sempre ligado): morto.
- c1 (parênteses redundantes no getter): verde.
- c2 (constante reescrita com separador de milhar): verde.

## Placar antes → depois

Workspace: **1266** → **1268** testes (2 novos em `cdrom_primeira_resposta_motor.rs`; 10.56
não ganhou teste novo — achado fechado sem mudança de código).

## Revisão cruzada (orquestrador)

Sem achados — esta iteração foi conduzida pelo próprio orquestrador (exceção vigente em
`docs/orquestracao.md`; ver STATUS.md).

## Decisões e notas

**1. `insert_disc()` é o jeito mais simples de ligar o motor num teste.** Já existia como
método público do `Cdrom` (usado por outros testes pra simular "tem disco"), e também liga o
motor como efeito colateral — não precisei inventar nenhuma API nova, só um getter
`motor_on()` pra expor o estado pro `bus.rs`.

**2. 10.56 fechado sem PR de código, só bookkeeping** — mesmo padrão já usado nesta sessão
pros achados 10.6/10.7 (iteração 0203): confirmar contra o código atual antes de "consertar"
algo que já está certo evita trabalho e erro de primeira tentativa desnecessários.
