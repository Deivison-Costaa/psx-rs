"""Executa o psx-cli num jogo com o roteiro do JSON e le o que ele produziu."""

import re
import subprocess
import time
from dataclasses import dataclass, field
from pathlib import Path

import numpy as np

from comum import (
    AMOSTRA_PC_S, BIOS, CICLOS_POR_SEGUNDO, INTERVALO_NOSSO_S, ROMS, Jogo, cartao_formatado,
)

FATOR_TIMEOUT = 8
TIMEOUT_MINIMO_S = 600
SILENCIO_PICO = 16
BLOCO_AUDIO = 441
TAXA_AUDIO = 44_100
AMOSTRA_RE = re.compile(r"^sample pc=0x([0-9A-Fa-f]{8}) step=\d+ cyc=(\d+)")
FRAME_RE = re.compile(r"^o-(\d+)-fb\.png$")
CICLOS_RE = re.compile(r"^# ciclos emulados: (\d+)")
SABOTAGENS = ("sem-botoes", "atrasa-botoes")
MARCA_ROMS = "{PSX_ROMS}"


@dataclass
class Execucao:
    pasta: Path
    rc: int | None = None
    timeout: bool = False
    segundos_de_parede: float = 0.0
    ciclos: int = 0
    panico: str | None = None
    frames: list = field(default_factory=list)
    pcs: list = field(default_factory=list)
    audio_nao_silencio: float | None = None
    audio_saturacao: float | None = None
    audio_segundos: float = 0.0


def argumentos(jogo: Jogo, cli: Path, pasta: Path, sabotagem: str | None) -> list:
    args = [
        str(cli), "--bios", str(BIOS), "--disc", str(jogo.cue_path), "--pad",
        "--max-time", f"{jogo.duracao_s}s",
        "--dump-vram-every", f"{INTERVALO_NOSSO_S}s", str(pasta / "frames" / "o"),
        "--dump-audio", str(pasta / "audio.raw"),
        "--sample-pcs", f"0s:{jogo.duracao_s}s:{AMOSTRA_PC_S}s",
    ]
    if jogo.memcard:
        args += ["--memcard", str(pasta / "card.mcd")]
    atraso = 5.0 if sabotagem == "atrasa-botoes" else 0.0
    if sabotagem != "sem-botoes":
        for b in jogo.botoes:
            args += ["--press", f"{b.b}@{b.t + atraso}s:{b.dur}s"]
    return args + [a.replace(MARCA_ROMS, str(ROMS)) for a in jogo.extra_cli]


def _le_stderr(caminho: Path, ex: Execucao) -> None:
    with open(caminho, encoding="utf-8", errors="replace") as f:
        for linha in f:
            m = AMOSTRA_RE.match(linha)
            if m:
                ex.pcs.append((int(m.group(2)) / CICLOS_POR_SEGUNDO, int(m.group(1), 16)))
                continue
            m = CICLOS_RE.match(linha)
            if m:
                ex.ciclos = int(m.group(1))
            elif "panicked" in linha and ex.panico is None:
                ex.panico = linha.strip()


def _le_frames(pasta: Path, ex: Execucao) -> None:
    for png in (pasta / "frames").glob("o-*-fb.png"):
        m = FRAME_RE.match(png.name)
        if m:
            ex.frames.append((int(m.group(1)) * INTERVALO_NOSSO_S, png))
    ex.frames.sort()
    for crua in (pasta / "frames").glob("o-*.vram"):
        crua.unlink()


def _le_audio(caminho: Path, ex: Execucao) -> None:
    if not caminho.exists():
        return
    pcm = np.fromfile(caminho, dtype="<i2")
    if pcm.size == 0:
        ex.audio_nao_silencio, ex.audio_saturacao = 0.0, 0.0
        return
    ex.audio_segundos = pcm.size / 2 / TAXA_AUDIO
    ex.audio_saturacao = float(np.mean((pcm >= 32767) | (pcm <= -32768)))
    n = pcm.size // (2 * BLOCO_AUDIO)
    if n == 0:
        ex.audio_nao_silencio = 0.0
        return
    blocos = np.abs(pcm[: n * 2 * BLOCO_AUDIO].astype(np.int32)).reshape(n, -1)
    ex.audio_nao_silencio = float(np.mean(blocos.max(axis=1) >= SILENCIO_PICO))


def executa(jogo: Jogo, cli: Path, pasta: Path, sabotagem: str | None, log) -> Execucao:
    (pasta / "frames").mkdir(parents=True, exist_ok=True)
    if jogo.memcard:
        (pasta / "card.mcd").write_bytes(cartao_formatado())
    args = argumentos(jogo, cli, pasta, sabotagem)
    (pasta / "comando.txt").write_text(" ".join(f"'{a}'" for a in args) + "\n", encoding="utf-8")
    ex = Execucao(pasta)
    limite = max(TIMEOUT_MINIMO_S, jogo.duracao_s * FATOR_TIMEOUT)
    log(f"[{jogo.id}] psx-cli: {jogo.duracao_s:.0f} s emulados (timeout {limite:.0f} s)")
    inicio = time.monotonic()
    with open(pasta / "stdout.log", "wb") as out, open(pasta / "stderr.log", "wb") as err:
        proc = subprocess.Popen(args, stdout=out, stderr=err)
        try:
            ex.rc = proc.wait(timeout=limite)
        except subprocess.TimeoutExpired:
            ex.timeout = True
            proc.kill()
            proc.wait()
    ex.segundos_de_parede = time.monotonic() - inicio
    _le_stderr(pasta / "stderr.log", ex)
    _le_frames(pasta, ex)
    _le_audio(pasta / "audio.raw", ex)
    log(f"[{jogo.id}] psx-cli: rc={ex.rc} em {ex.segundos_de_parede:.0f} s, {len(ex.frames)} frames")
    return ex
