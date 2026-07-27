# STATUS

> Memória do projeto entre iterações. O contexto do agente é descartado a cada iteração;
> este arquivo não. Teto de 16 KB imposto por `status_size.rs`. Curto e verdadeiro.

## Última iteração concluída

**0008** — orquestração (ROADMAP 0.8): SKILL.md fonte única (12 passos, bateria de mutação
inclusa, trabalhador NÃO mergeia), oc-iter.ps1 (métricas automáticas), oc-loop.ps1.

## Próxima tarefa

**ROADMAP 0.9** — carregamento de BIOS. Em `crates/psx-core/src/bus.rs`: tipo `Bios` com
`Bios::from_bytes(Vec<u8>) -> Result<Bios, BiosError>` exigindo exatamente 512 KiB
(0x80000), acesso `read32(offset)` little-endian; R3: o core recebe bytes, NUNCA lê arquivo.
Em `crates/psx-cli`: flag `--bios <path>` lê o arquivo, repassa ao core e imprime tamanho +
SHA-256 (I/O mora no CLI). Spec: `docs/reference/01-memory-map.md` (BIOS em
KSEG1 0xBFC00000, região de 512 KiB) — leia o índice e vá direto à seção Memory Map.
Teste: `psx-core/tests/bus_bios.rs` (512 KiB ok; 256 KiB e vazio → erro; read32 de offsets
conhecidos de um blob sintético). Armadilha: não validar hash no core (BIOS varia por
região/versão — validação de identidade é do CLI/app, não do hardware).

## Repositório

- `main` protegida a partir da iter 0004; 1 PR por item; merge commit (nunca squash);
  commits test→feat→docs; título de PR validado pela CI.
- Iterações são cronológicas e nem sempre na ordem dos itens (0003↔item 0.5, p.ex.);
  o vínculo real está no título do PR e no doc da iteração.

## Placar de testes

Workspace: 8 testes (todos meta-testes de processo). EXEs de teste e scoreboard chegam nos
itens 0.7 e 1.11; ainda não existe emulador.

## Bloqueios

(nenhum)

## Invariantes

(nenhuma ainda — nascem com o código; índice com âncoras quando existirem)

## Notas

1. BIOS local: `bios/SCPH1001.BIN` (MD5 924E392ED05558FFDB115408C263DCCF), gitignored,
   validada na iter 0009 (item 0.9). Nunca commitar.
