# docs/mutantes/ — manifestos de mutação

Um arquivo por iteração: `NNNN-slug.mut`, irmão de `docs/iterations/NNNN-slug.md`. A chave
de 4 dígitos é a mesma usada por `metrics_freshness.rs` e sobrevive a sufixos (0008b, 0017c).

## Gramática

Blocos delimitados por sentinelas em **coluna 0**. Código Rust nunca começa linha com `@@`,
portanto não é necessário escape.

```
# comentário (linha ignorada)
<chave>: <valor>        # diretiva
@@DE / @@PARA / @@FIM   # sentinelas, linha exata, coluna 0
```

### Cabeçalho (antes do primeiro registro)

| Campo | Obrigatório | Descrição |
|---|---|---|
| `formato: 1` | Sim | Versão do formato |
| `iteracao: NNNN` | Sim | Deve bater com o prefixo do nome do arquivo |
| `item: X.Y` | Sim | Item do ROADMAP |
| `alvo: crates/<crate>/src/<arquivo>.rs` | Sim | Caminho default do fonte mutado |
| `teste: <target>` | Sim | Nome do target cargo (ex.: `gpu_vram_transfers`) |
| `arquivada: <motivo>` | Não | Desliga a checagem de âncora (asserência D) |

### Registros

Cada registro é iniciado por UMA destas chaves:

| Chave | Semântica |
|---|---|
| `mutante: <ID>` | Ao rodar, o teste TEM que FALHAR |
| `controle: <ID>` | Ao rodar, o teste TEM que PASSAR |
| `equivalente: <ID>` | TEM que PASSAR, exige `justificativa:` ≥80 caracteres |

Campos de registro:

| Campo | Obrigatório | Descrição |
|---|---|---|
| `rotulo: <texto>` | Sim | Mínimo 15 caracteres |
| `arquivo: <caminho>` | Não | Sobrepõe o `alvo` do cabeçalho |
| `teste: <target>` | Não | Sobrepõe o do cabeçalho (mutante cujo assassino mora em outro arquivo) |
| `justificativa: <texto>` | Só em `equivalente:` | Mínimo 80 caracteres, pode ter continuação em linhas indentadas |
| `ocorrencias: <N\|todas>` | Não (default 1) | Vale para o `@@DE` **imediatamente seguinte** |

### Edições

Uma edição é um trio `@@DE` / `@@PARA` / `@@FIM`. Um registro tem 1..n edições, aplicadas
**atomicamente** (juntas) e revertidas juntas. Não existe "tipo de mutação": inserção, remoção e
troca são a mesma coisa — o payload é multi-linha.

```
@@DE
<linhas originais>
@@PARA
<linhas substituídas>
@@FIM
```

- **Inserção:** `@@DE` vazio, `@@PARA` com o conteúdo a inserir.
- **Remoção:** `@@DE` com o conteúdo a remover, `@@PARA` vazio.
- **Troca:** ambos preenchidos.

## Regra de casamento — linha inteira

O casamento é feito sobre `"\n" + conteudo_do_arquivo + "\n"` procurando
`"\n" + de + "\n"` — **nunca substring solta**.

**Exemplo real (gpu.rs):** a linha 269 é
`self.vram[py as usize * 1024 + px as usize] = hw1;` e existem duas linhas
`self.vram[py2 as usize * 1024 + px2 as usize]`. Com casamento por substring, a âncora curta
bate 3 vezes (a primeira por prefixo da linha 269). Com casamento por linha inteira bate 2,
que é o correto. Isto elimina uma classe inteira de mutação silenciosamente errada.

## `ocorrencias:` é contrato, não dica

Declarou `ocorrencias: 2`, achou 3 → **erro duro, nada é mutado.** Declarou `ocorrencias: 1`,
achou 2 → idem. O manifesto só é aceito se TODAS as âncoras baterem exatamente.

`ocorrencias: todas` exige pelo menos 1 ocorrência, sem teto.

Este contrato resolve os dois casos reais da iter 0038: mutação que DEVE atingir vários sítios
(ypos ignorado nos três caminhos — `ocorrencias: 3`) e mutação que deve atingir ALGUNS de
muitos (stride nos caminhos de cópia mas nunca no fill — `ocorrencias: 2` com âncora específica
dos caminhos de cópia).
