# 0065 — cdrom-read

- **Data:** 2026-07-29
- **Item do roadmap:** 4.2c
- **Objetivo:** Comandos ReadN (06h) e ReadS (1Bh) com INT1, RDDATA, DRQSTS e encadeamento.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | ReadN (L856), ReadS (L919), ReadN/ReadS (L924), HSTS DRQSTS (L236), RDDATA (L284), HCHPCTL BFRD (L277-278) | docs/reference/06-cdrom.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | Pause durante ReadN resultaria em INT0 imediatamente apos ack | Pause tem INT3→INT2 como segunda resposta, e so depois de ack fica INT0 | Teste `pause_para_read_n` falhou; corrigido para esperar INT2 e so depois verificar INT0 |
| 2 | timing | ReadN/ReadS disparam INT5 com mesmo padrao de erro do SeekL (0x80) | Comportamento documentado para comandos sem disco — assumimos consistente com Setloc | Teste `read_n_sem_disco_retorna_int5` escrito com 0x80 |
| 3 | registro | O `data_pos=2048` setado como default no construtor acionaria DRQSTS falso-positivo | data_pos=0 por padrao, DRQSTS so acende quando read_mode != 0 | Conferido no construtor — data_pos=0 e read_mode=0 |

## Revisão do PR anterior (iter 0064)

1. **(TESTE QUE NÃO MEDE)** `read_data_sector_no_segundo_setor` usava buffer uniforme (todos bytes 0xAA), tornando impossível distinguir setor 0 do setor 1. Corrigido: buffer com padrão distinto (0xCC de fundo, 0xBB no setor 1) e asserção dupla (`sector[0]==0xBB, sector[1]==0xCC`).
2. **(PARÂMETRO NÃO CONSUMIDO)** `read_data_sector` é função pura que recebe `&[u8]` e índice; não há FIFO de comandos envolvido — OK.
3. **(REGRA DE BORDA)** CDROM file format não tem regra de borda gráfica — OK.
4. **(CAMPO DE BIT LIDO ERRADO)** `bin_offset` usa `total_frames * 2352`; testado com golden value explícito — OK.
5. **(PANIC/LAÇO ILIMITADO)** Sem `unsafe`/`unwrap()` no código de cdrom_bin_cue — OK.
6. **(CITAÇÃO DE SPEC)** `confere-citacoes.ps1` verde — OK.
7. **(ESCOPO TRANSBORDADO)** Diff do PR #79 só contém o item 4.2b — OK.

## Bateria de mutação

Placar da bateria: 7/7 mutantes mortos, 2/2 controles verdes, 0 equivalente - ./docs/mutantes/0065-cdrom-read.mut

| Mutante | Rótulo | Resultado |
|---|---|---|
| m1 | ReadN sem disco retorna INT3 em vez de INT5 | MORREU |
| m2 | ReadS sem disco retorna INT3 em vez de INT5 | MORREU |
| m3 | deliver_second poe INT2 em vez de INT1 | MORREU |
| m4 | ReadN nao encadeia — pending_second nao e re-setado | MORREU |
| m5 | buffer de dados preenchido com zeros | MORREU |
| m6 | data_pos nao e resetado — DRQSTS fica desligado | MORREU |
| m7 | ReadN nao seta reading flag no stat | MORREU |
| K1 | extrair stat_byte em variavel local (equivale) | verde |
| K2 | if em vez de else if no ReadN sem disco (equivale) | verde |

## Placar antes → depois

Workspace: 538 testes (10 novos: cdrom_read).

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo orquestrador -->

## Decisões e notas

- Dados do setor são stub: 2048 bytes preenchidos com `(i+1) & 0xFF`. O buffer real será fornecido pelo parser BIN/CUE no item 4.3 (DMA canal 3 + entrega de setores).
- `data_buffer` usa `Cell<[u8; 2048]>` — cópia integral a cada byte lido via RDDATA. Pode ser trocado por `RefCell<Vec<u8>>` ou DMA com ponteiro quando performance importar no 4.3.
- Encadeamento do ReadN: `deliver_second` case 5 sempre zera `pending_second`; a reativação (set(5)) ocorre em `write8` HCLRCTL quando `read_mode==1`. Isso evita alterar a estrutura dos casos 1-4 existentes.
- BFRD (bit 7 do HCHPCTL) é armazenado mas não usado como gate para leitura RDDATA neste estágio — dados ficam disponíveis imediatamente após INT1. O gate DMAn será implementado no item 4.3.
- Manifesto 0063: âncoras `m2` e `K1` quebraram porque o padrão `if !disc_inserted.get()` foi replicado em ReadN/ReadS. Corrigido adicionando `} else { self.seeking.set(true);` ao @@DE para torná-los específicos do SeekL.
