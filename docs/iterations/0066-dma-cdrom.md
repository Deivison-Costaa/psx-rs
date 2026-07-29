# 0066 — dma-cdrom

- **Data:** 2026-07-29
- **Item do roadmap:** 4.3a
- **Objetivo:** DMA canal 3 — registradores, gate BFRD no DRQSTS e transferência RDDATA→RAM.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | DMA Register Summary, DMA3 channel 3, SyncMode=0, DPCR, DICR, CHCR (L32, L62, L129-130, L189) | docs/reference/04-dma.md |
| psx-spx | HCHPCTL BFRD, RDDATA, Copy Data to Main RAM (L940-954) | docs/reference/06-cdrom.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | registro | O handoff dizia "BFRD (bit 7 do HCHPCTL) é armazenado" mas o campo `bfrd: Cell<bool>` nunca era populado da escrita do registrador — HCHPCTL (bank 0, offset 3) não tinha handler de escrita | HCHPCTL é um registrador write-only no bank 0, offset 3; BFRD é o bit 7 | Revisão adversarial (padrão #4): encontrado antes de começar o item novo. Corrigido na revisão do PR #80 |
| 2 | timing | DRQSTS ficava sempre ligado quando havia dados no buffer, independente de BFRD | DRQSTS só deve acender após BFRD=1 (L285-286: "software must set the BFRD flag, then wait until DRQSTS is set") | Teste `drqsts_baixo_quando_bfrd_nao_setado_apos_int1` falhou; corrigido adicionando gate `(hchpctl & 0x80) != 0` em `hsts()` |
| 3 | timing | O teste `hsts_drqsts_setado_quando_dados_disponiveis` da iter 0065 quebrou com o gate BFRD porque não setava BFRD antes de verificar DRQSTS | O fluxo correto é INT1 → BFRD=1 → DRQSTS=1 | Teste corrigido adicionando `hchpctl_write(0x80)` antes da asserção |

## Revisão do PR anterior (iter 0065)

Revisão do PR anterior: sem achados novos após correção do HCHPCTL.

- Padrão #1 (teste que não mede): `rddata_retorna_sequencia_de_bytes` usa `assert_ne!` contra (0,0,0,0) — fraco, mas mitigado por m5 (buffer zerado). OK para stub.
- Padrão #2 (parâmetro não consumido): ReadN/ReadS são sem parâmetros, parâmetros consumidos pelo Setloc. OK.
- Padrão #3 (regra de borda): CDROM não tem regra de borda de pixel. OK.
- Padrão #4 (campo de bit lido errado): HCHPCTL sem handler de escrita — **CORRIGIDO** nesta rodada. Renomeado `bfrd` → `hchpctl: Cell<u8>`, adicionado handler `3 if self.bank.get() == 0`.
- Padrão #5 (panic/loop): `data_pos` tem guarda `pos < 2048`. OK.
- Padrão #6 (citação de spec): `confere-citacoes.ps1` verde. OK.
- Padrão #7 (escopo): PR #80 contém só o item 4.2c. OK.

## Bateria de mutação

Placar da bateria: 7/7 mutantes mortos, 2/2 controles verdes, 0 equivalente - ./docs/mutantes/0066-dma-cdrom.mut

| Mutante | Rótulo | Resultado |
|---|---|---|
| m1 | DMA3 não verifica DRQSTS — transfere sem BFRD | MORREU |
| m2 | DMA3 não verifica bit 28 — executa só com bit 24 | MORREU |
| m3 | DMA3 não limpa bit 24 após transferência | MORREU |
| m4 | DMA3 lê RDDATA do offset errado (0 em vez de 2) | MORREU |
| m5 | DMA3 escreve com endian errado (big-endian) | MORREU |
| m6 | BCR zero tratado como 0 palavras em vez de 10000h | MORREU |
| m7 | DMA3 incrementa MADR com step -4 em vez de +4 | MORREU |
| K1 | Extrair mask de bits 24+28 em variável local | verde |
| K2 | Extrair madr mascarado em variável local | verde |

## Placar antes → depois

Workspace: 548 testes (10 novos: cdrom_dma).

## Divisão de item (R4)

O item 4.3 do ROADMAP juntava três funcionalidades: DMA3, BFRD gate, e acoplamento do DiscLayout. Dividido em:
- **4.3a** (este item): DMA canal 3 + gate BFRD no DRQSTS
- **4.3b**: Acoplar DiscLayout + dados do .bin à entrega de setores do ReadN/ReadS

## Decisões e notas

- DMA3 segue o mesmo padrão de trigger dos canais existentes: executa quando CHCR bits 24+28 estão setados. Adicionalmente verifica `cdrom.drqsts_active()` como condição extra — sem DRQSTS, não transfere.
- Valor de CHCR (`11000000h`) tem bits 24 e 28 setados; ambos são limpos após transferência (igual OTC).
- Endianess: DMA3 lê 4 bytes de RDDATA (offset 2) e monta word little-endian antes de escrever na RAM.
- `hchpctl` renomeado de `bfrd` para capturar o registrador HCHPCTL inteiro (bits 7=BFRD, 6=BFWR, 5=SMEN). O método `drqsts_active()` encapsula a lógica de gate.
- Manifesto 0062: âncora K2 atualizada para incluir o novo braço HCHPCTL no `write8` (similar ao reparo da 0065).
- `cdrom_read.rs`: teste `hsts_drqsts_setado_quando_dados_disponiveis` atualizado para setar BFRD antes de verificar DRQSTS.
