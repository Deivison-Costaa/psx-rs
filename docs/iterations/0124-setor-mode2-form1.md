<!-- Custo, tokens e duração NÃO entram aqui: são medidos por scripts/oc-iter.ps1 em
     docs/metricas.csv, casados por head_antes/head_depois. -->

# 0124 — setor-mode2-form1

- **Data:** 2026-07-31
- **Item do roadmap:** 4.4s
- **Objetivo:** `read_sector_from_disc` lia os dados de usuário a partir de `010h` em todo setor,
  que é o offset do **Mode1**; num disco Mode2/Form1 isso devolve o sub-header como se fosse dado e
  desloca o setor inteiro em 8 bytes.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | § Mode2/Form1 (CD-XA) (L621) | docs/reference/15-cdrom-format.md |
| psx-spx | § Mode1 (Original CDROM) (L613) | docs/reference/15-cdrom-format.md |
| psx-spx | § Mode0 (Empty) (L607) | docs/reference/15-cdrom-format.md |

```
  Mode1                        Mode2/Form1 (CD-XA)
  00Ch 4    Header (Mode=01h)  00Ch 4    Header (Mode=02h)
                               010h 4    Sub-Header
                               014h 4    Copy of Sub-Header
  010h 800h Data (2048 bytes)  018h 800h Data (2048 bytes)
```

O byte `00Fh` de cada setor diz qual dos dois é — o offset é propriedade **do setor**, não do
disco nem do `.cue`.

## Como o item foi encontrado

Despejando o `.bin` do Crash Bandicoot nos setores que a BIOS estava lendo (LBA 4, 5 e 16):

```
  setor  4: cabecalho ...0000020402  → modo 02h
     +0x10: 00 00 08 00 00 00 08 00 "          Licensed  by          Sony Com"
     +0x18:                         "          Licensed  by          Sony Computer En"
  setor 16: +0x10: 00 00 09 00 00 00 09 00 01 "CD001" ...
            +0x18:                            01 "CD001" ...
```

O PVD do ISO9660 (setor 16) começa com `01 'CD001'`. Lendo de `010h` ele vinha precedido de oito
bytes de sub-header — qualquer parser de sistema de arquivos lê lixo.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | teste | Que `setor_alem_do_fim_do_bin_nao_estoura` cobria a guarda de tamanho. | — | O mutante m6 (`offset + 0x10 > len` → `offset > len`) **sobreviveu**. O teste pedia um setor muito além do fim, que a guarda frouxa ainda rejeita. A guarda só importa quando o **início** do setor cabe no vetor e o byte de modo (`00Fh`) não — caso que só um `.bin` truncado no meio de um setor produz. Teste novo com `truncate(150 * 2352 + 5)`. |
| 2 | processo | Que eu podia inventar o mutante "offset 0x18 fixo". | — | O meta-teste recusou: essa edição já era o `m2` de `0077-acoplar-disclayout-cdrom.mut`, que **eu tinha acabado de reancorar** nesta mesma iteração. Dois manifestos com a mesma edição inflam o placar (padrão da iter 0038). O mutante saiu do 0124 e ficou um comentário no manifesto dizendo onde ele mora. |
| 3 | diagnóstico | Que corrigir o offset destravaria o boot. | — | **Não destravou.** A sequência de comandos do CD ficou byte a byte idêntica: os mesmos 17 comandos, o mesmo ponto de parada. Defeito real, provado por spec e por despejo do disco — mas não era a causa do sintoma. Invariante 26 pela segunda vez, agora do lado negativo. |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente — docs/mutantes/0124-setor-mode2-form1.mut

| # | Mutação | Teste que pegou |
|---|---|---|
| m1 | volta ao offset fixo do Mode1 (o defeito original) | `setor_mode2_form1_le_os_dados_a_partir_de_0x18` |
| m3 | byte de modo lido de `00Eh` | `o_modo_e_lido_de_cada_setor_e_nao_do_disco` |
| m4 | Mode2 pula só o sub-header, esquece a cópia | `setor_mode2_form1_le_os_dados_a_partir_de_0x18` |
| m5 | compara com Mode1 e inverte os dois offsets | `setor_mode1_continua_lendo_a_partir_de_0x10` |
| m6 | guarda de tamanho afrouxada | `setor_com_cabecalho_truncado_no_fim_do_bin_nao_estoura` (só depois de consertado — erro 1) |
| c1 | escolha do offset como `match` (equivalente) | verde |
| c2 | modo em variável local (cosmético) | verde |

`0077-acoplar-disclayout-cdrom.mut` foi reancorado no formato novo e **a bateria dele foi rodada
de novo**: 5/5 mortos, 2/2 controles verdes.

## Placar antes → depois

Workspace: 814 → **821** testes (7 em `cdrom_setor_mode2.rs`), 0 falhas.

Efeito medido no boot real: **nenhum na sequência de comandos** — os mesmos 17, o mesmo ponto de
parada, `HINTSTS==INT1` nos mesmos 27 924 passos.

## Onde o boot está agora (medido nesta iteração)

Duas medições novas, e as duas dizem que **o bloqueio saiu do CD-ROM**.

**1. A última troca com o drive está completa.** Janela do terceiro `GetID` (passo 89 878 152):

```
  89878152  W porta1 val=0x1A          ; GetID
  89906800  R porta1 banco1            ; INT3: 1 byte (o stat)
  89906834  W porta3 banco1 val=0x07   ; ack
  89907161..89907198  R porta1 x8      ; INT2: os OITO bytes da resposta licenciada
  89907442  W porta3 banco1 val=0x07   ; ack
  — nenhum acesso ao CD em 310 M passos —
```

A BIOS consumiu os oito bytes, reconheceu o disco e **parou de perguntar**. Não sobrou comando
pendente, resposta não lida nem IRQ pendurada.

**2. A tela de licença renderiza inteira.** Despejo da VRAM ao fim de 400 M passos (preservado em
`psx-estado/referencias/0124-vram-apos-licenca.png`): o logotipo **SONY COMPUTER ENTERTAINMENT**
aparece completo e correto, com o losango em degradê, sobre o fundo cinza — e as texturas do
logotipo "PlayStation" já estão carregadas na metade direita da VRAM. `display LIGADO: 640x478`,
318 278 pixels não-zero.

**3. O TTY do kernel para em `SetGraphDebug`.** 725 bytes, terminando em:

```
  System Controller ROM Version 97/01/10 c2
  bad hankaku code 0x4 / 0xed / 0xbd
  SetGraphDebug:level:1,type:0 reverse:0
```

Nenhuma linha sobre `SYSTEM.CNF` nem sobre executável — a referência do DuckStation registra
`Executable path: 'SCUS_949.00'` neste ponto.

**4. O PC final está num laço de contagem do shell**, não do driver de CD:

```
  0x800422D8: 25290001  addiu $t1,$t1,1
  0x800422DC: 1531FF5F  bne   $t1,$s1,0x8004205C
```

## Revisão cruzada (orquestrador)

Sem achados que barrem o merge.

- **O `.bin` sintético dos testes antigos é Mode0** (vetor zerado, byte `00Fh` = 0), então cai no
  ramo `else` e continua lendo de `010h`. Os 11 testes de `cdrom_read.rs`/`cdrom_dma.rs` que
  dependem dele ficaram verdes sem uma linha de mudança — e agora há teste explícito
  (`setor_com_modo_zero_usa_o_offset_do_mode1`) dizendo que isso é intencional.
- **A guarda de tamanho vem antes de ler o byte de modo.** Ler `bin[offset + 0x0F]` para decidir o
  offset é indexação nova num `&[u8]`; sem a guarda ela pode estourar em `.bin` truncado.
- **Gates:** `purity`, `file_size`, `comment_density`, `roadmap_size`, `status_size`,
  `spec_citations`, `mutation_manifest`, `mutation_anchors` e `mutation_battery` verdes.
- **Árvore limpa.** `crates/psx-core/src/bin` removido; o PNG que o `rodajogo` escreveu na raiz do
  repositório foi **conferido antes** e movido para `psx-estado/referencias/`, não apagado.

## Decisões e notas

- **Defeito confirmado não é causa confirmada — de novo.** A 0121 fechou a invariante 26 pelo lado
  positivo (sintoma sumiu). Esta iteração fecha pelo lado negativo: spec, despejo do disco e sete
  testes provam que o offset estava errado, e mesmo assim o boot não andou um passo. As duas
  metades da invariante agora têm caso registrado.
- **Quatro itens seguidos, quatro degraus.** 0121 (`GetID` aparece) → 0122 (laço de retentativa
  acaba) → 0123 (setores são lidos) → 0124 (dados dos setores corretos). O CD-ROM saiu do caminho
  crítico: a tela de licença renderiza e o drive está em silêncio por opção da BIOS, não por
  bloqueio nosso.
- **Próximo degrau é de subsistema diferente.** O shell fica no laço `0x8004205C..0x800422DC`
  (`bne $t1,$s1` com `addiu $t1,$t1,1`), depois de imprimir `SetGraphDebug` e antes de qualquer
  menção a `SYSTEM.CNF`. Item 4.4t: instrumentar esse laço — quem são `$t1` e `$s1`, que tabela ele
  varre — e diferenciar contra a referência do DuckStation, que a esta altura já carregou
  `SCUS_949.00`.
