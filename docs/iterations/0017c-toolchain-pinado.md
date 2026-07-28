# 0017c — Toolchain pinado (local == CI)

- **Data:** 2026-07-27
- **Item do roadmap:** 0.4 (infra de CI; fora da escada do M1)
- **Objetivo:** Tirar a versão do compilador das mãos de "stable" dos dois lados e pinar em
  um lugar só, para que o portão local do trabalhador signifique o que a CI vai medir.
- **Autor:** orquestrador (Claude). Sem código de emulador.

## Motivação

A iter 0016 abriu PR com a CI vermelha por `clippy::manual_checked_ops`, lint que o clippy
local não conhecia: stable local em 1.92.0 (2025-12-08), CI em 1.97. O trabalhador rodou o
passo 7 do protocolo e viu verde; a auto-remediação do loop rodou `clippy --fix` e não achou
o que corrigir — as duas coisas pelo mesmo motivo. Diagnóstico completo em
`docs/iterations/0016-cpu-mult-div.md` § Achado 1.

Decisão do usuário: pinar, em vez de depender de alguém lembrar de rodar `rustup update`.

## Spec consultada

Não se aplica — item de infra, sem hardware envolvido.

## O que entrou

- `rust-toolchain.toml` na raiz: `channel = "1.97.1"`, `components = ["rustfmt", "clippy"]`,
  `profile = "minimal"`.
- `ci.yml`: sai `dtolnay/rust-toolchain@stable`, entra `rustup toolchain install` sem
  argumento — que resolve a versão pelo arquivo pinado.
- `crates/psx-core/tests/toolchain_pin.rs`: dois meta-testes. O primeiro reprova canal
  flutuante (`stable`/`beta`/`nightly`) e exige `rustfmt` e `clippy` declarados; o segundo
  reprova `ci.yml` que escolha toolchain por conta própria.

O segundo teste é o que faz o pin valer alguma coisa: sem ele, bastava alguém reintroduzir
um `@stable` no workflow para os dois lados divergirem de novo em silêncio.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que era | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | `rustup toolchain install X --component rustfmt clippy` aceita lista | `--component` é por ocorrência; `clippy` virou nome de toolchain | `error: invalid value 'clippy' for '[TOOLCHAIN]...'` |

## Bateria de mutação

Placar: **4/4 mutantes pegos, 2/2 controles verdes**.

| # | Mutação | Resultado |
|---|---|---|
| 1 | `channel = "stable"` | Pego |
| 2 | Remover a linha `components` | Pego |
| 3 | Reintroduzir `uses: dtolnay/rust-toolchain@stable` no ci.yml | Pego |
| 4 | Fixar `toolchain: 1.97.1` num action do workflow (versão em dois lugares) | Pego |

Controles: comentário novo no `rust-toolchain.toml` → verde; comentário novo no `ci.yml` →
verde (e `ci_workflow.rs` intacto).

## Placar antes → depois

149 testes → **151** (2 meta-testes novos).

## Decisões e notas

- **Custo do determinismo:** lint e correção novos do compilador só chegam quando alguém
  subir o `channel`. Isso vira item de ROADMAP com iteração própria — troca a versão, roda o
  portão inteiro, registra o que quebrou. Nunca como efeito colateral de outro item.
- O rustup trata `1.97.1` como toolchain distinto de `stable`, mesmo sendo a mesma versão:
  pinar custou um download de ~250 MB na máquina de desenvolvimento, feito em paralelo com a
  iteração 0017 para não custar espera.
- A remediação automática nº 3 do `oc-loop.ps1` (`fmt` + `clippy --fix` quando o check fica
  vermelho) continua útil para desvio de formatação, mas o caso que a motivou — "CI com
  toolchain mais novo que o local" — deixa de existir. O cabeçalho do script já previa esse
  caso e prescrevia o remédio errado; corrigido junto.
