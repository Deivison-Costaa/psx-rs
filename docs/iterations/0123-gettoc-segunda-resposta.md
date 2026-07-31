<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0123 — gettoc-segunda-resposta

- **Data:** 2026-07-31
- **Item do roadmap:** 4.4r
- **Objetivo:** `GetTOC` (`1Eh`) caía no braço default do `send_command` e nunca armava a segunda
  resposta; o driver do kernel ficava esperando um INT2 que não vinha.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Second Responses (INT2) (or INT5 if failed) (L2002) | docs/reference/06-cdrom.md |
| psx-spx | § ReadTOC - Command 1Eh (L961) | docs/reference/06-cdrom.md |
| psx-spx | § First Response (L2047) | docs/reference/06-cdrom.md |

Da § Second Responses: `1Eh ReadTOC — INT3(late-stat), INT2(stat)`. Da § ReadTOC: o comando é
*"rather slow, the second response appears after about 1 second delay"* e *"returns only status
information"*. Da § First Response: *"The ReadTOC command is doing similar initialization, and
should have similar timing as Init command"* — daí o `0x13CCE` que a 0121 já tinha posto no
`first_response_cycles`.

## O que mudou

Um braço `0x1E` no `send_command`, com três linhas: empilha o stat, `intsts = 3`,
`pending_second = 1`. O caso 1 do `deliver_second` (que existe desde o `Init`) já é exatamente
`INT2(stat)` — não precisou de código novo do outro lado.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | teste | Que afirmar `HINTSTS == 2` e "primeiro byte == 02h" identificava a segunda resposta do `ReadTOC`. | § Second Responses: `1Eh ReadTOC → INT2(stat)` é **um** byte; `1Ah GetID → INT2(stat,flg,typ,atip,"SCEx")` são **oito**. | O mutante m2 (`pending_second = 2`, a segunda resposta do GetID) **sobreviveu** à bateria. Desde a 0122 o `GetID` com disco também responde INT2 e também começa com `02h` — os dois só se distinguem pelo tamanho da FIFO. Teste passou a afirmar o RSLRRDY baixo depois do primeiro byte. |
| 2 | processo | Que reancorar manifesto antigo fosse acidente da 0121. | — | Terceira reancoragem em três iterações: `0062/K2` e `0080/m2` na 0121, `0062/m4` na 0122, `0062/m3` aqui — esta porque `self.pending_second.set(1);` deixou de ser única no arquivo assim que o `ReadTOC` passou a usar o mesmo caso. É custo previsível de mexer num módulo com manifesto antigo, e o `ocorrencias:` pega sempre. |

## Bateria de mutação

Placar da bateria: 6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0123-gettoc-segunda-resposta.mut

| # | Mutação | Teste que pegou |
|---|---|---|
| m1 | `ReadTOC` volta a não armar segunda resposta (o defeito original) | `gettoc_arma_a_segunda_resposta_int2` |
| m2 | `ReadTOC` arma a segunda resposta do `GetID` | `segunda_resposta_do_gettoc_devolve_o_stat` (só depois de consertado — ver erro 1) |
| m3 | primeira resposta sai como INT2 | `gettoc_responde_int3_com_o_stat` |
| m4 | primeira resposta não empilha o stat | `gettoc_responde_int3_com_o_stat` |
| m5 | `ReadTOC` usa o atraso curto | `gettoc_usa_o_atraso_longo_da_primeira_resposta` |
| m6 | segunda resposta do caso 1 não baixa o BUSYSTS | `gettoc_deixa_o_drive_pronto_para_o_proximo_comando` |
| c1 | stat por variável local (cosmético) | verde |
| c2 | braço do atraso longo como match guard (cosmético) | verde |

## Placar antes → depois

Workspace: 808 → **814** testes (6 em `cdrom_gettoc.rs`), 0 falhas.

Efeito medido no boot real (SCPH1001 + Crash Bandicoot, 400 M passos):

```
  ANTES                        DEPOIS
  Test                         Test
  GetStat                      GetStat
  GetID                        GetID
  GetTOC   <-- parava aqui     GetTOC
                               GetStat, GetID
                               Setloc 00:02:04, SeekL, Setmode 80h, ReadN, Pause
                               Setloc 00:02:05, SeekL, Setmode 80h, ReadN, Pause
                               GetID    <-- para aqui
```

4 comandos → 17, e `HINTSTS==INT1` em 27 924 passos: **setores de dados estão sendo entregues pela
primeira vez no projeto**. A cadeia é a mesma da referência do DuckStation da 0120.

## Revisão cruzada (orquestrador)

Sem achados que barrem o merge.

- **O braço novo reusa o caso 1 do `deliver_second` em vez de criar um caso 6.** São a mesma
  resposta pela spec (`INT2(stat)`), e um caso novo idêntico seria código duplicado que o próximo
  mutante equivalente ia expor.
- **`first_response_cycles` não foi tocado.** Ele já tratava `0x1E` desde a 0121, e agora tem
  teste próprio (`gettoc_usa_o_atraso_longo_da_primeira_resposta`) — antes, o `0x1E` estava
  coberto só por tabela junto do `Init`.
- **Gates:** `purity`, `file_size`, `comment_density`, `roadmap_size`, `status_size`,
  `spec_citations`, `mutation_manifest`, `mutation_anchors` e `mutation_battery` verdes.
- **Árvore limpa**, `crates/psx-core/src/bin` removido antes do commit.

## Decisões e notas

- **O boot chegou a ler o disco.** Os `Setloc 00:02:04` / `00:02:05` são LBA 4 e 5 — a região dos
  **setores de licença**, onde mora a string *"Licensed by Sony Computer Entertainment"*. Depois de
  lê-los, a BIOS reemite `GetID` e para. É a verificação de licença, e ela está falhando.
- **Próximo degrau, medido no mesmo passo.** Despejando o `.bin` do disco: o byte `00Fh` do setor 4
  é `02h`, ou seja **Mode2/Form1**. Pela § Mode2/Form1 (CD-XA)
  (`docs/reference/15-cdrom-format.md` L621) esse formato tem `010h 4 Sub-Header` mais `014h 4
  Copy of Sub-Header`, e os dados de usuário começam em **`018h`** — só o Mode1 (L613) começa em
  `010h`. O nosso `read_sector_from_disc` usa `abs_sector*2352 + 0x10` fixo, então **todo setor sai
  deslocado 8 bytes**, com o sub-header colado na frente. Confirmado no setor 16 (o PVD do
  ISO9660): a partir de `010h` lê-se `00 00 09 00 00 00 09 00 01 'CD001'`, quando devia começar em
  `01 'CD001'`. Item 4.4s.
