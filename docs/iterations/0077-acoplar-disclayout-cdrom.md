# 0077 — acoplar-disclayout-cdrom

- **Data:** 2026-07-29
- **Item do roadmap:** 4.3b
- **Objetivo:** acoplar DiscLayout e dados do .bin a entrega de setores no Cdrom, substituindo o buffer stub por dados reais do BIN.

## Revisão do PR anterior

Revisão do PR anterior (#90, iter 0076): sem achados. Padrões conferidos:
1. Teste que não mede — asserções concretas (bit 15 = 1 ou 0); bateria 5/5 confirma cobertura
2. Parâmetro não consumido — sem novos comandos GP0
3. Regra de borda trocada — sem rasterização
4. Campo de bit lido errado — val & 1 para GP1(09h) bit 0, correto
5. Panic ou laço ilimitado — sem unwrap/expect/unsafe
6. Citação de spec — confere-citacoes.ps1 verde
7. Escopo transbordado — itens adicionados ao ROADMAP solicitados pelo projeto
8. Portão que não mede — bateria 5/5, .resultado rastreado
9. Manifesto arquivado — 0050 e 0074 ancoras reparadas (não arquivadas)

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| psx-spx | ReadN/ReadS (L924) | docs/reference/06-cdrom.md |
| psx-spx | CDROM Incoming Data / Buffer Overrun Timings — Copy Data to Main RAM (L940) | docs/reference/06-cdrom.md |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | Que `assert_ne!` com "(stat & 0x01) != 0" testava "sem erro"; o Setloc sem disco retorna stat com bit 0 setado, então bit 0 = 0 significa sucesso | A spec define stat bit0 como error flag | Teste falhou: Setloc com disco retorna stat bit0=0, mas a asserção esperava != 0 |
| 2 | manifesto | Que a ancora `*b = (i as u8).wrapping_add(1);` sobreviveria ilesa no manifesto 0065-cdrom-read.mut depois de embrulhar o stub em `else {}` | A indentaçao aumentou 4 espaços (de 20 para 24) por causa do novo nível de `if/else` | mutation_anchors reprovou m5: ancora esperada 1 vez, encontrada 0 |

## Bateria de mutação

Placar da bateria: 5/5 mutantes mortos, 2/2 controles verdes, 0 equivalente - ./docs/mutantes/0077-acoplar-disclayout-cdrom.mut

| Registro | Rótulo | Testes que pegaram, conforme o `.resultado` |
|---|---|---|
| m1 | troca minuto com segundo no calculo do setor absoluto | read_n_retorna_dados_do_bin_no_setor_correto |
| m2 | data_start com offset 0x18 em vez de 0x10 no setor raw | read_n_retorna_dados_do_bin_no_setor_correto |
| m3 | bcd_to_int inverte dezena com unidade | read_n_retorna_dados_do_bin_no_setor_correto |
| m4 | multiplicador de bytes por setor 2353 em vez de 2352 | read_n_retorna_dados_do_bin_no_setor_correto |
| m5 | sempre usa stub — read_sector_from_disc sempre retorna None | read_n_retorna_dados_do_bin_no_setor_correto |
| c1 | renomeia abs_sector para sector em todo o escopo (cosmetico) | sobreviveu |
| c2 | acrescenta comentario antes do calculo do setor (cosmetico) | sobreviveu |

## Placar antes → depois

Workspace: **563** → **564** testes (+1: `read_n_retorna_dados_do_bin_no_setor_correto`).

## Revisão cruzada (orquestrador)

<!-- Preenchido pelo orquestrador. -->

## Decisões e notas

1. O Cdrom não armazena `DiscLayout` e `bin_data` diretamente (usaria `RefCell`, não usado no projeto).
   Os dados de disco são armazenados no `Bus` (campos `disc_layout: Option<DiscLayout>` e
   `disc_bin: Option<Vec<u8>>`) e passados como referências imutáveis via `Cdrom::write8` →
   `deliver_second`. Rust aceita borrows disjuntos de campos diferentes em `&mut self`.
2. O `DiscLayout` é recebido mas ainda não usado para validação de track — o `_layout` em
   `read_sector_from_disc` é placeholder para suporte a multi-track no futuro. A implementação
   atual usa o setor absoluto direto (BCD→int→frame→offset no BIN), que funciona para discos
   de 1 track.
3. Os testes existentes (stub) continuam funcionando porque `Bus::new()` não injeta disco
   (`disc_layout` e `disc_bin` são `None`), e o `deliver_second` cai no fallback de stub.
4. A ancora `m5` de `docs/mutantes/0065-cdrom-read.mut` precisou de reparo: a indentaçao de
   `*b = (i as u8).wrapping_add(1);` mudou de 20 para 24 espaços por causa do novo bloco
   `if/else` em `deliver_second` case 5.
