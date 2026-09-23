"""Referencias do DuckStation (duckstation-regtest com o patch local de entrada/dump).

As capturas sao frames de jogos comerciais: ficam em REFERENCIAS/<id>/, fora do git.
"""

import json
import math
import os
import re
import shutil
import subprocess
import time
from dataclasses import dataclass
from pathlib import Path

from comum import (
    DS_REGTEST, FPS_REFERENCIA, INTERVALO_REF_QUADROS, REFERENCIAS, Jogo,
)

TIMEOUT_DS_S = 1800
QUADRO_RE = re.compile(r"frame_(\d+)\.png$")


@dataclass(frozen=True)
class QuadroRef:
    t: float
    caminho: Path


@dataclass(frozen=True)
class Referencia:
    pasta: Path
    meta: dict
    quadros: tuple

    def na_janela(self, t: float, janela: float) -> list:
        return [q for q in self.quadros if abs(q.t - t) <= janela]


def quadro_de(t: float) -> int:
    return int(round(t * FPS_REFERENCIA))


def ds_input(jogo: Jogo) -> str:
    """Roteiro de botoes em segundos -> REGTEST_INPUT (frames do regtest, intervalo inclusivo)."""
    partes = []
    for b in jogo.botoes:
        ini = quadro_de(b.t)
        fim = max(ini, quadro_de(b.t + b.dur) - 1)
        partes.append(f"{b.b}@{ini}-{fim}")
    return ",".join(partes)


def _settings(jogo: Jogo) -> str:
    s = ["BIOS/PatchFastBoot=false"]
    if not jogo.memcard:
        s.append("MemoryCards/Card1Type=None")
    return ";".join(s)


def gera(jogo: Jogo, log) -> Referencia:
    if not DS_REGTEST.exists():
        raise RuntimeError(f"duckstation-regtest nao encontrado em {DS_REGTEST} (PSX_DS_REGTEST)")
    if any(a.startswith("--swap-disc") for a in jogo.extra_cli):
        log(f"[{jogo.id}] AVISO: o regtest nao troca disco; a referencia so cobre o 1o disco")
    destino = REFERENCIAS / jogo.id
    tmp = REFERENCIAS / f".{jogo.id}.tmp"
    shutil.rmtree(tmp, ignore_errors=True)
    tmp.mkdir(parents=True)
    quadros = quadro_de(jogo.duracao_s) + 1
    env = dict(os.environ)
    env.update(
        REGTEST_SETTINGS=_settings(jogo),
        REGTEST_INPUT=ds_input(jogo),
        REGTEST_RAM_AT="",
        REGTEST_VRAM_AT="",
    )
    args = [
        str(DS_REGTEST), "-log", "Info", "-dumpdir", str(tmp), "-frames", str(quadros),
        "-dumpinterval", str(INTERVALO_REF_QUADROS), "--", str(jogo.cue_path),
    ]
    log(f"[{jogo.id}] DuckStation: {quadros} quadros, dump a cada {INTERVALO_REF_QUADROS}")
    inicio = time.monotonic()
    with open(tmp / "console.log", "wb") as saida:
        rc = subprocess.run(args, env=env, stdout=saida, stderr=subprocess.STDOUT,
                            timeout=TIMEOUT_DS_S, check=False).returncode
    if rc != 0:
        raise RuntimeError(f"duckstation-regtest saiu com {rc}; veja {tmp / 'console.log'}")
    sub = [p for p in tmp.iterdir() if p.is_dir()]
    if len(sub) != 1:
        raise RuntimeError(f"saida inesperada do regtest em {tmp}")
    novos = tmp / "frames"
    novos.mkdir()
    for png in sub[0].glob("frame_*.png"):
        png.rename(novos / png.name)
    for extra in ("duckstation.log",):
        if (sub[0] / extra).exists():
            (sub[0] / extra).rename(tmp / extra)
    shutil.rmtree(sub[0])
    meta = {
        "id": jogo.id,
        "assinatura": jogo.assinatura_roteiro(),
        "fps": FPS_REFERENCIA,
        "intervalo_quadros": INTERVALO_REF_QUADROS,
        "quadros": quadros,
        "ds_input": ds_input(jogo),
        "ds_settings": _settings(jogo),
        "regtest": str(DS_REGTEST),
        "gerado_em": time.strftime("%Y-%m-%d %H:%M:%S"),
        "segundos_de_parede": round(time.monotonic() - inicio, 1),
    }
    (tmp / "meta.json").write_text(json.dumps(meta, indent=2), encoding="utf-8")
    shutil.rmtree(destino, ignore_errors=True)
    tmp.rename(destino)
    ref = carrega(jogo)
    log(f"[{jogo.id}] referencia: {len(ref.quadros)} quadros em {destino}")
    return ref


def carrega(jogo: Jogo) -> Referencia:
    pasta = REFERENCIAS / jogo.id
    meta_arq = pasta / "meta.json"
    if not meta_arq.exists():
        raise FileNotFoundError(f"sem referencia em {pasta}; rode com --refs")
    meta = json.loads(meta_arq.read_text(encoding="utf-8"))
    fps = float(meta.get("fps", FPS_REFERENCIA))
    quadros = []
    for png in (pasta / "frames").glob("frame_*.png"):
        m = QUADRO_RE.search(png.name)
        if m:
            quadros.append(QuadroRef(int(m.group(1)) / fps, png))
    quadros.sort(key=lambda q: q.t)
    return Referencia(pasta, meta, tuple(quadros))


def desatualizada(ref: Referencia, jogo: Jogo) -> bool:
    return ref.meta.get("assinatura") != jogo.assinatura_roteiro()


def cobre(ref: Referencia, jogo: Jogo) -> bool:
    return bool(ref.quadros) and ref.quadros[-1].t >= jogo.duracao_s - math.ceil(
        INTERVALO_REF_QUADROS / FPS_REFERENCIA
    )
