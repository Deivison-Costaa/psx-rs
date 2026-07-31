<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0122 — getid-disco-licenciado

- **Data:** 2026-07-31
- **Item do roadmap:** 4.4q
- **Objetivo:** o `GetID` respondia a linha **No Disk** da spec mesmo com disco dentro; passar a
  responder a linha **Licensed:Mode2** quando há disco.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § GetID - Command 1Ah (L1139) | docs/reference/06-cdrom.md |
| psx-spx | § Second Responses (INT2) (or INT5 if failed) (L2002) | docs/reference/06-cdrom.md |

A tabela da § GetID dá uma linha por estado do drive. Duas interessam:

```
  No Disk          INT3(stat)  INT5(08h,40h, 00h,00h, 00h,00h,00h,00h)
  Licensed:Mode2   INT3(stat)  INT2(02h,00h, 20h,00h, 53h,43h,45h,4xh)
```

E, sobre a quarta letra (L1170-1173): *"'SCEI' (Japan/NTSC), 'SCEA' (America/NTSC), 'SCEE'
(Europe/PAL) ... the PSX refuses to boot if it doesn't match up for the local region"*. A BIOS do
projeto é a `SCPH1001`, NTSC-U — logo `SCEA`.

## Como o item foi encontrado (antes de rodar)

Este item não veio de instrumentação: veio de **ler o código enquanto a 0121 ainda estava sendo
desenhada**. `deliver_second`, caso 2, empilhava `08h, 40h` e seis zeros incondicionalmente, sem
olhar para `disc_inserted`. Cruzando com a tabela da spec, isso é literalmente a resposta de
"disco ausente" — e a BIOS que a recebe não tem motivo nenhum para continuar o boot.

A previsão ficou escrita no doc da 0121 antes de existir código desta aqui, e a medição da 0121 a
confirmou: o shell repetia `GetStat, GetStat, GetID` a cada ~18,9 M passos, para sempre. É a
assinatura de quem pergunta, recebe "não tem disco", e tenta de novo.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | nenhum | — | — | Item previsto por leitura e confirmado pela medição da iteração anterior; a implementação passou de primeira. O que custou tempo foram **duas âncoras de mutação**: o `m4` de `0062-cdrom-regs.mut` apontava para `self.intsts.set(5);` com 16 espaços, e o `if` novo reindentou a linha; e os meus `m1`/`c2` casaram 2 vezes porque `if self.disc_inserted.get() {` também existe no `Init`. Ambas resolvidas alargando a âncora até ficar única — o `ocorrencias:` é contrato, e ele fez o trabalho dele. |

## Bateria de mutação

Placar da bateria: 6/6 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0122-getid-disco-licenciado.mut

| # | Mutação | Teste que pegou |
|---|---|---|
| m1 | ignora o disco e responde sempre No Disk (o defeito original) | `getid_com_disco_devolve_os_oito_bytes_da_spec` |
| m2 | com disco responde INT5 em vez de INT2 | `getid_com_disco_responde_int2_e_nao_int5` |
| m3 | região vai como `SCEE` | `regiao_e_scea_para_a_bios_ntsc_u` |
| m4 | flags marcam negado e ausente | `flags_dizem_licenciado_presente_e_nao_audio` |
| m5 | tipo do disco vai como Mode1/Audio | `tipo_do_disco_e_mode2` |
| m6 | primeiro byte não é o stat | `primeiro_byte_da_segunda_resposta_e_o_stat` |
| c1 | região escrita byte a byte (equivalente) | verde |
| c2 | condição do disco em variável local (cosmético) | verde |

## Placar antes → depois

Workspace: 800 → **808** testes (8 em `cdrom_getid_disco.rs`), 0 falhas.

Efeito medido no boot real (SCPH1001 + Crash Bandicoot, 400 M passos), com o `cdstate`:

| | antes | depois |
|---|---|---|
| comandos ao CD-ROM | 45 (laço) | 4 |
| padrão | `GetStat, GetStat, GetID` repetido a cada ~18,9 M passos | linear, sem repetição |
| último comando | `GetID` | **`GetTOC` (1Eh)**, no passo 88 380 174 |

**O laço de retentativa acabou.** O shell aceitou a identificação do disco e pediu o próximo
comando da cadeia. Critério de aceitação do item cumprido no que ele podia provar: a BIOS parou de
insistir. O que ele pedia literalmente (`Setloc`/`SeekL`/`ReadN` depois do `GetID`) ainda não
apareceu, porque surgiu um degrau que não estava na referência que eu tinha lido — ver abaixo.

## Revisão cruzada (orquestrador)

Sem achados que barrem o merge.

- **O caminho sem disco está protegido por teste, não por sorte.** `getid_sem_disco_continua_na_
  linha_no_disk` e `getid_sem_disco_continua_respondendo_int5` foram escritos justamente para
  ficarem verdes antes e depois; o `cdrom_regs.rs::getid_sem_disco_retorna_int5`, que já existia,
  também continua verde.
- **Região fixada, e assumido como buraco.** `SCEA` está no código, não derivado do disco. O certo
  é ler o setor de licença do `.bin`; anotado como 10.57. Com uma BIOS NTSC-U e um disco USA, a
  constante e a leitura dariam o mesmo valor — não há como o teste distinguir, e por isso não
  fingi que havia.
- **Gates:** `purity`, `file_size`, `comment_density`, `roadmap_size`, `status_size`,
  `spec_citations`, `mutation_manifest`, `mutation_anchors` e `mutation_battery` verdes.
- **Árvore limpa**, `crates/psx-core/src/bin` removido antes do commit.

## Decisões e notas

- **Previsão por leitura de código pagou.** As duas iterações anteriores (0119, 0120) foram
  diagnóstico puro, e a 0120 fechou registrando que "plausível não é provado". Aqui o caminho foi o
  inverso: o defeito foi lido no fonte, escrito no doc da 0121 como previsão, e a medição
  confirmou. Vale como contraponto à invariante 27 — nem todo bloqueio precisa de oráculo externo;
  alguns estão à vista de quem lê a tabela da spec ao lado do `match`.
- **Próximo degrau, já medido.** `GetTOC` (`1Eh`) cai no braço `_` do `send_command`: responde
  INT3 com o stat e **nunca arma segunda resposta**. A § Second Responses (L2002) de `docs/reference/06-cdrom.md` exige
  `1Eh ReadTOC — INT3(late-stat), INT2(stat)`, e a § ReadTOC (L961) do mesmo `06-cdrom.md` avisa que ela é *"rather slow,
  the second response appears after about 1 second delay"*. Sem o INT2 o driver fica esperando para
  sempre — e é exatamente onde o boot está agora. Item 4.4r.
