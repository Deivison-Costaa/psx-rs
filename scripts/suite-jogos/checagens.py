"""Comparacao de imagens e checagens por checkpoint (muda, parecido_ds, resolucao)."""

from dataclasses import dataclass, field
from functools import lru_cache
from pathlib import Path

import numpy as np
from PIL import Image

LADO = (160, 120)
BLOCO = 8
DIFERENCA_MINIMA_MUDA = 1.0
PC_SPAN_LACO_PEQUENO = 0x100
DESVIO_UNIFORME = 4.0
C1 = (0.01 * 255) ** 2
C2 = (0.03 * 255) ** 2


@lru_cache(maxsize=4096)
def cinza(caminho: Path) -> np.ndarray:
    with Image.open(caminho) as im:
        return np.asarray(im.convert("L").resize(LADO, Image.Resampling.BOX), dtype=np.float64)


@lru_cache(maxsize=4096)
def tamanho(caminho: Path) -> tuple:
    with Image.open(caminho) as im:
        return im.size


def _blocos(a: np.ndarray) -> np.ndarray:
    h, w = a.shape
    return a[: h - h % BLOCO, : w - w % BLOCO].reshape(h // BLOCO, BLOCO, w // BLOCO, BLOCO)


def ssim(a: np.ndarray, b: np.ndarray) -> float:
    """SSIM medio em blocos 8x8 sem sobreposicao."""
    x, y = _blocos(a), _blocos(b)
    mx, my = x.mean(axis=(1, 3)), y.mean(axis=(1, 3))
    vx, vy = x.var(axis=(1, 3)), y.var(axis=(1, 3))
    cov = ((x - mx[:, None, :, None]) * (y - my[:, None, :, None])).mean(axis=(1, 3))
    s = ((2 * mx * my + C1) * (2 * cov + C2)) / ((mx**2 + my**2 + C1) * (vx + vy + C2))
    return float(s.mean())


def correlacao(a: np.ndarray, b: np.ndarray) -> float:
    """Correlacao normalizada; imagens quase uniformes valem 1 se tem o mesmo brilho."""
    sa, sb = a.std(), b.std()
    if sa < DESVIO_UNIFORME or sb < DESVIO_UNIFORME:
        if sa < DESVIO_UNIFORME and sb < DESVIO_UNIFORME:
            return max(0.0, 1.0 - abs(a.mean() - b.mean()) / 32.0)
        return 0.0
    return float(((a - a.mean()) * (b - b.mean())).mean() / (sa * sb))


def similaridade(nosso: Path, ref: Path) -> float:
    a, b = cinza(nosso), cinza(ref)
    return max(0.0, 0.5 * correlacao(a, b) + 0.5 * ssim(a, b))


def uniforme(caminho: Path) -> bool:
    return float(cinza(caminho).std()) < DESVIO_UNIFORME


def diferenca_media(a: Path, b: Path) -> float:
    return float(np.abs(cinza(a) - cinza(b)).mean())


@dataclass
class ResultadoCheckpoint:
    t: float
    marco: str
    checa: tuple
    nosso: Path | None = None
    ref: Path | None = None
    ref_t: float | None = None
    score: float | None = None
    limiar: float = 0.0
    tam_nosso: tuple | None = None
    tam_ref: tuple | None = None
    muda_dif: float | None = None
    pc_span: int | None = None
    pc_amostras: int = 0
    falhas: list = field(default_factory=list)
    avisos: list = field(default_factory=list)

    @property
    def aprovado(self) -> bool:
        return not self.falhas

    def como_dict(self) -> dict:
        return {
            "t": self.t,
            "marco": self.marco,
            "checa": list(self.checa),
            "aprovado": self.aprovado,
            "primeira_falha": self.falhas[0] if self.falhas else None,
            "falhas": self.falhas,
            "avisos": self.avisos,
            "score": None if self.score is None else round(self.score, 3),
            "limiar": self.limiar,
            "ref_t": None if self.ref_t is None else round(self.ref_t, 2),
            "tam_nosso": self.tam_nosso,
            "tam_ref": self.tam_ref,
            "muda_dif": None if self.muda_dif is None else round(self.muda_dif, 2),
            "pc_span": self.pc_span,
            "pc_amostras": self.pc_amostras,
            "nosso": str(self.nosso) if self.nosso else None,
            "ref": str(self.ref) if self.ref else None,
        }


def _melhor_ref(nosso: Path, candidatos: list):
    melhor = None
    for q in candidatos:
        s = similaridade(nosso, q.caminho)
        if melhor is None or s > melhor[0]:
            melhor = (s, q)
    return melhor


def _checa_muda(r: ResultadoCheckpoint, janela_nossa: list, pcs: list) -> None:
    difs = [diferenca_media(a, b) for a, b in zip(janela_nossa, janela_nossa[1:])]
    r.muda_dif = max(difs) if difs else None
    if r.muda_dif is None:
        r.falhas.append("muda: menos de 2 frames nossos na janela")
    elif r.muda_dif < DIFERENCA_MINIMA_MUDA:
        r.falhas.append(f"muda: imagem congelada (dif max {r.muda_dif:.2f} < {DIFERENCA_MINIMA_MUDA})")
    r.pc_amostras = len(pcs)
    if pcs:
        r.pc_span = max(pcs) - min(pcs)
        if r.pc_span < PC_SPAN_LACO_PEQUENO:
            r.falhas.append(
                f"muda: CPU presa num laco pequeno (PCs em 0x{min(pcs):08X}..0x{max(pcs):08X})"
            )
    else:
        r.falhas.append("muda: sem amostras de PC na janela")


def avalia_checkpoint(cp, nossos: list, ref, pcs: list) -> ResultadoCheckpoint:
    """nossos: lista ordenada de (t, png); pcs: lista de (t, pc)."""
    r = ResultadoCheckpoint(cp.t, cp.marco, cp.checa, limiar=cp.limiar)
    recentes = [(t, p) for (t, p) in nossos if cp.t - 1.0 - 1e-6 <= t <= cp.t + 1e-6]
    r.nosso = recentes[-1][1] if recentes else None
    if r.nosso is not None:
        r.tam_nosso = tamanho(r.nosso)
    candidatos = ref.na_janela(cp.t, cp.janela) if ref is not None else []
    com_conteudo = [q for q in candidatos if not uniforme(q.caminho)]
    if com_conteudo:
        candidatos = com_conteudo
    if r.nosso is not None and candidatos:
        s, q = _melhor_ref(r.nosso, candidatos)
        r.score, r.ref, r.ref_t, r.tam_ref = s, q.caminho, q.t, tamanho(q.caminho)
        if "parecido_ds" in cp.checa and uniforme(q.caminho):
            r.avisos.append("janela da referencia so tem telas uniformes: parecido_ds pouco informativo")
    if "muda" in cp.checa:
        janela_nossa = [p for (t, p) in nossos if cp.t - cp.janela - 1e-6 <= t <= cp.t + 1e-6]
        janela_pcs = [pc for (t, pc) in pcs if cp.t - cp.janela <= t <= cp.t]
        _checa_muda(r, janela_nossa, janela_pcs)
    if "parecido_ds" in cp.checa or "resolucao" in cp.checa:
        if r.nosso is None:
            r.falhas.append("sem frame nosso exibido perto do checkpoint (display desligado?)")
        elif not candidatos:
            r.falhas.append("sem frame de referencia na janela")
    if "parecido_ds" in cp.checa and r.score is not None and r.score < cp.limiar:
        r.falhas.append(
            f"parecido_ds: {r.score:.3f} < {cp.limiar} (melhor ref em t={r.ref_t:.2f}s)"
        )
    if "resolucao" in cp.checa and r.tam_ref is not None and r.tam_nosso != r.tam_ref:
        r.falhas.append(f"resolucao: nossa {r.tam_nosso} != referencia {r.tam_ref}")
    return r
