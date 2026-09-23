#!/usr/bin/env python3
"""Suite de jogos: roda o psx-cli com roteiros de botoes e compara com o DuckStation.

Uso: python3 scripts/suite-jogos/suite.py [--jogos id1,id2] [--refs] [--cli CAMINHO]
                                          [--saida logs/suite/<nome>] [--jobs 1|2]
Veja scripts/suite-jogos/README.md.
"""

import argparse
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from threading import Lock

sys.path.insert(0, str(Path(__file__).resolve().parent))

import checagens  # noqa: E402
import nosso  # noqa: E402
import referencia  # noqa: E402
import relatorio  # noqa: E402
from comum import BIOS, CICLOS_POR_SEGUNDO, REPO, ErroDeJogo, Jogo, lista_jogos  # noqa: E402

SATURACAO_MAXIMA = 0.001
MAX_JOBS = 2
_TRAVA_LOG = Lock()


def log(msg: str) -> None:
    with _TRAVA_LOG:
        print(f"{time.strftime('%H:%M:%S')} {msg}", file=sys.stderr, flush=True)


def _commit() -> str:
    r = subprocess.run(["git", "-C", str(REPO), "rev-parse", "--short", "HEAD"],
                       capture_output=True, text=True, check=False)
    return r.stdout.strip() or "?"


def _referencia(jogo: Jogo, res: relatorio.ResultadoJogo):
    try:
        ref = referencia.carrega(jogo)
    except (FileNotFoundError, ValueError) as e:
        res.falhas.append(f"referencia: {e}")
        return None
    if referencia.desatualizada(ref, jogo):
        res.falhas.append("referencia desatualizada (botoes/duracao/cartao mudaram); rode com --refs")
    elif not referencia.cobre(ref, jogo):
        res.falhas.append("referencia nao cobre a duracao do jogo; rode com --refs")
    return ref


def _checa_execucao(jogo: Jogo, ex: nosso.Execucao, res: relatorio.ResultadoJogo) -> None:
    if ex.timeout:
        res.falhas.append(f"timeout: psx-cli nao terminou em {ex.segundos_de_parede:.0f} s")
    elif ex.rc != 0:
        res.falhas.append(f"psx-cli saiu com codigo {ex.rc} (ver stderr.log)")
    if ex.panico:
        res.falhas.append(f"panic: {ex.panico[:160]}")
    alvo = int(jogo.duracao_s * CICLOS_POR_SEGUNDO)
    if not ex.timeout and ex.ciclos < alvo:
        res.falhas.append(f"parou em {ex.ciclos / CICLOS_POR_SEGUNDO:.1f} s de {jogo.duracao_s:g} s")
    if ex.audio_nao_silencio is None:
        res.falhas.append("audio: psx-cli nao gravou audio.raw")
    else:
        if ex.audio_nao_silencio < jogo.min_nao_zero:
            res.falhas.append(f"audio: {ex.audio_nao_silencio * 100:.1f}% nao silencioso "
                              f"< minimo {jogo.min_nao_zero * 100:.0f}%")
        if (ex.audio_saturacao or 0) >= SATURACAO_MAXIMA:
            res.falhas.append(f"audio: saturacao {ex.audio_saturacao * 100:.3f}% >= 0,1%")
    res.execucao = {
        "rc": ex.rc,
        "timeout": ex.timeout,
        "segundos_de_parede": round(ex.segundos_de_parede, 1),
        "segundos_emulados": ex.ciclos / CICLOS_POR_SEGUNDO,
        "frames": len(ex.frames),
        "amostras_pc": len(ex.pcs),
        "audio_nao_silencio": ex.audio_nao_silencio,
        "audio_saturacao": ex.audio_saturacao,
        "audio_min": jogo.min_nao_zero,
        "pasta": str(ex.pasta),
    }


def roda_jogo(jogo: Jogo, args, saida: Path) -> relatorio.ResultadoJogo:
    res = relatorio.ResultadoJogo(jogo.id, jogo.nome)
    if not jogo.cue_path.exists():
        res.falhas.append(f"disco nao encontrado: {jogo.cue_path} (PSX_ROMS)")
        return res
    ref = _referencia(jogo, res)
    pasta = saida / jogo.id
    try:
        ex = nosso.executa(jogo, args.cli, pasta, args.sabotagem, log)
    except OSError as e:
        res.falhas.append(f"nao consegui executar o psx-cli: {e}")
        return res
    _checa_execucao(jogo, ex, res)
    ref_ok = ref if ref is not None and ref.quadros else None
    for cp in jogo.checkpoints:
        r = checagens.avalia_checkpoint(cp, ex.frames, ref_ok, ex.pcs)
        res.checkpoints.append(r)
        res.avisos += [f"t={cp.t:g}s: {a}" for a in r.avisos]
    res.folha = relatorio.folha_de_contato(res, saida / f"{jogo.id}-folha.png")
    log(f"[{jogo.id}] {'APROVADO' if res.aprovado else 'REPROVADO'}"
        + ("" if res.aprovado else f": {res.primeira_falha}"))
    return res


def _argumentos(argv: list):
    p = argparse.ArgumentParser(description="Suite de jogos psx-rs x DuckStation")
    p.add_argument("--jogos", help="ids separados por virgula (padrao: todos em jogos/)")
    p.add_argument("--refs", action="store_true", help="(re)gera as referencias do DuckStation")
    p.add_argument("--so-refs", action="store_true", help="so gera referencias, nao roda o psx-cli")
    p.add_argument("--cli", type=Path, default=REPO / "target" / "release" / "psx-cli")
    p.add_argument("--saida", type=Path, help="pasta do relatorio (padrao logs/suite/<data-hora>)")
    p.add_argument("--jobs", type=int, default=1, help=f"jogos em paralelo (1..{MAX_JOBS})")
    p.add_argument("--sabotagem", choices=nosso.SABOTAGENS,
                   help="quebra a execucao de proposito para provar que as checagens reprovam")
    args = p.parse_args(argv)
    if not 1 <= args.jobs <= MAX_JOBS:
        p.error(f"--jobs tem de estar entre 1 e {MAX_JOBS} (maquina compartilhada)")
    args.cli = args.cli.resolve()
    if args.saida is None:
        args.saida = REPO / "logs" / "suite" / time.strftime("%Y%m%d-%H%M%S")
    args.saida = args.saida.resolve()
    return args


def main(argv: list) -> int:
    args = _argumentos(argv)
    try:
        jogos = lista_jogos(args.jogos.split(",") if args.jogos else None)
    except ErroDeJogo as e:
        log(f"ERRO: {e}")
        return 2
    if args.refs or args.so_refs:
        for jogo in jogos:
            try:
                referencia.gera(jogo, log)
            except (RuntimeError, OSError, subprocess.TimeoutExpired) as e:
                log(f"[{jogo.id}] ERRO ao gerar referencia: {e}")
                return 2
        if args.so_refs:
            return 0
    for obrigatorio in (args.cli, BIOS):
        if not obrigatorio.exists():
            log(f"ERRO: nao encontrei {obrigatorio}")
            return 2
    inicio, t0 = time.strftime("%Y-%m-%d %H:%M:%S"), time.monotonic()
    with ThreadPoolExecutor(max_workers=args.jobs) as pool:
        resultados = list(pool.map(lambda j: roda_jogo(j, args, args.saida), jogos))
    contexto = {
        "cli": str(args.cli), "commit": _commit(), "inicio": inicio,
        "segundos": time.monotonic() - t0, "sabotagem": args.sabotagem,
        "jogos": [j.id for j in jogos],
    }
    md, _ = relatorio.escreve(resultados, args.saida, contexto)
    aprovados = sum(r.aprovado for r in resultados)
    log(f"{aprovados}/{len(resultados)} aprovados; relatorio em {md}")
    return 0 if aprovados == len(resultados) else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
