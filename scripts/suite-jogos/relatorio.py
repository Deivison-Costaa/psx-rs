"""Relatorio da suite: relatorio.md, relatorio.json e folha de contato por jogo."""

import json
from dataclasses import dataclass, field
from pathlib import Path

from PIL import Image, ImageDraw

LARGURA_QUADRO = 320
ALTURA_QUADRO = 240
ALTURA_LEGENDA = 34
VERDE = (60, 200, 90)
VERMELHO = (230, 70, 60)
AMARELO = (240, 220, 80)
FUNDO = (28, 28, 30)


@dataclass
class ResultadoJogo:
    id: str
    nome: str
    falhas: list = field(default_factory=list)
    avisos: list = field(default_factory=list)
    checkpoints: list = field(default_factory=list)
    execucao: dict = field(default_factory=dict)
    folha: Path | None = None

    @property
    def aprovado(self) -> bool:
        return not self.falhas and all(c.aprovado for c in self.checkpoints)

    @property
    def primeira_falha(self) -> str | None:
        if self.falhas:
            return self.falhas[0]
        for c in self.checkpoints:
            if c.falhas:
                return f"t={c.t:g}s: {c.falhas[0]}"
        return None

    def como_dict(self) -> dict:
        return {
            "id": self.id,
            "nome": self.nome,
            "resultado": "APROVADO" if self.aprovado else "REPROVADO",
            "primeira_falha": self.primeira_falha,
            "falhas_do_jogo": self.falhas,
            "avisos": self.avisos,
            "execucao": self.execucao,
            "checkpoints": [c.como_dict() for c in self.checkpoints],
            "folha_de_contato": str(self.folha) if self.folha else None,
        }


def _encaixa(caminho: Path | None) -> Image.Image:
    if caminho is None or not Path(caminho).exists():
        vazio = Image.new("RGB", (LARGURA_QUADRO, ALTURA_QUADRO), (60, 0, 0))
        ImageDraw.Draw(vazio).text((10, 10), "sem frame", fill=(255, 255, 255))
        return vazio
    with Image.open(caminho) as im:
        return im.convert("RGB").resize((LARGURA_QUADRO, ALTURA_QUADRO))


def folha_de_contato(res: ResultadoJogo, saida: Path) -> Path:
    linhas = max(1, len(res.checkpoints))
    altura = 22 + linhas * (ALTURA_QUADRO + ALTURA_LEGENDA)
    folha = Image.new("RGB", (2 * LARGURA_QUADRO + 12, altura), FUNDO)
    d = ImageDraw.Draw(folha)
    d.text((6, 4), f"{res.nome}: psx-rs (esquerda) | DuckStation, melhor frame na janela (direita)",
           fill=(230, 230, 230))
    for i, c in enumerate(res.checkpoints):
        y = 22 + i * (ALTURA_QUADRO + ALTURA_LEGENDA)
        cor = VERDE if c.aprovado else VERMELHO
        score = "-" if c.score is None else f"{c.score:.2f}"
        ref_t = "-" if c.ref_t is None else f"{c.ref_t:.2f}s"
        d.text((6, y), f"t={c.t:g}s  {c.marco}", fill=cor)
        motivo = "OK" if c.aprovado else c.falhas[0]
        d.text((6, y + 14), f"score={score} ref_t={ref_t}  {motivo}"[:110],
               fill=cor if c.aprovado else AMARELO)
        folha.paste(_encaixa(c.nosso), (4, y + ALTURA_LEGENDA))
        folha.paste(_encaixa(c.ref), (LARGURA_QUADRO + 8, y + ALTURA_LEGENDA))
    folha.save(saida)
    return saida


def _linha_md(c) -> str:
    score = "-" if c.score is None else f"{c.score:.3f}"
    ref_t = "-" if c.ref_t is None else f"{c.ref_t:.2f}"
    tam = "-" if c.tam_nosso is None else f"{c.tam_nosso[0]}x{c.tam_nosso[1]}"
    tam_ref = "-" if c.tam_ref is None else f"{c.tam_ref[0]}x{c.tam_ref[1]}"
    estado = "ok" if c.aprovado else "**FALHA**"
    motivo = "; ".join(c.falhas + [f"aviso: {a}" for a in c.avisos]) or ""
    return (f"| {c.t:g} | {c.marco} | {', '.join(c.checa)} | {estado} | {score} / {c.limiar:g} "
            f"| {ref_t} | {tam} / {tam_ref} | {motivo} |")


def escreve(resultados: list, pasta: Path, contexto: dict) -> tuple:
    pasta.mkdir(parents=True, exist_ok=True)
    aprovados = sum(r.aprovado for r in resultados)
    dados = {
        "contexto": contexto,
        "aprovados": aprovados,
        "total": len(resultados),
        "jogos": [r.como_dict() for r in resultados],
    }
    arq_json = pasta / "relatorio.json"
    arq_json.write_text(json.dumps(dados, indent=2, ensure_ascii=False), encoding="utf-8")
    md = [
        "# Suite de jogos: psx-rs x DuckStation",
        "",
        f"- Binario: `{contexto.get('cli')}`",
        f"- Commit: `{contexto.get('commit')}`",
        f"- Inicio: {contexto.get('inicio')}  |  duracao: {contexto.get('segundos', 0):.0f} s",
    ]
    if contexto.get("sabotagem"):
        md.append(f"- **Sabotagem ligada: `{contexto['sabotagem']}`** (execucao deliberadamente quebrada)")
    md += [f"- Resultado: **{aprovados}/{len(resultados)} aprovados**", "",
           "| Jogo | Resultado | Primeira falha |", "|---|---|---|"]
    for r in resultados:
        md.append(f"| {r.nome} (`{r.id}`) | {'APROVADO' if r.aprovado else '**REPROVADO**'} "
                  f"| {r.primeira_falha or ''} |")
    for r in resultados:
        ex = r.execucao
        md += ["", f"## {r.nome} (`{r.id}`): {'APROVADO' if r.aprovado else 'REPROVADO'}", ""]
        md.append(f"- Execucao: rc={ex.get('rc')}, {ex.get('segundos_de_parede', 0):.0f} s de parede, "
                  f"{ex.get('segundos_emulados', 0):.1f} s emulados, {ex.get('frames', 0)} frames")
        if ex.get("audio_nao_silencio") is not None:
            md.append(f"- Audio: {ex['audio_nao_silencio'] * 100:.1f}% nao silencioso "
                      f"(minimo {ex.get('audio_min', 0) * 100:.0f}%), saturacao "
                      f"{ex.get('audio_saturacao', 0) * 100:.3f}%")
        for f in r.falhas:
            md.append(f"- **Falha:** {f}")
        for a in r.avisos:
            md.append(f"- Aviso: {a}")
        if r.folha:
            md.append(f"- Folha de contato: `{r.folha.name}`")
        md += ["", "| t (s) | Marco | Checagens | Estado | Score / limiar | Ref t (s) "
               "| Resolucao nossa / ref | Motivo |", "|---|---|---|---|---|---|---|---|"]
        md += [_linha_md(c) for c in r.checkpoints]
    arq_md = pasta / "relatorio.md"
    arq_md.write_text("\n".join(md) + "\n", encoding="utf-8")
    return arq_md, arq_json
