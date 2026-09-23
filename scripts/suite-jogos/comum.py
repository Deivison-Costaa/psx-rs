"""Caminhos, constantes e leitura/validacao dos JSON de jogo da suite."""

import hashlib
import json
import os
from dataclasses import dataclass, field
from pathlib import Path

CICLOS_POR_SEGUNDO = 33_868_800
FPS_REFERENCIA = 59.94
INTERVALO_REF_QUADROS = 15
INTERVALO_NOSSO_S = 0.5
AMOSTRA_PC_S = 0.01
JANELA_S = 3.0
LIMIAR_PADRAO = 0.6
CHECAGENS_VALIDAS = ("muda", "parecido_ds", "resolucao")
BOTOES = (
    "select", "l3", "r3", "start", "up", "right", "down", "left",
    "l2", "r2", "l1", "r1", "triangle", "circle", "cross", "square",
)

AQUI = Path(__file__).resolve().parent
REPO = AQUI.parent.parent
JOGOS_DIR = AQUI / "jogos"
FACULDADE = Path("/home/ddscosta/Área de trabalho/Faculdade/Programação com Agentes")
ROMS = Path(os.environ.get("PSX_ROMS", FACULDADE / "roms" / "dados"))
ORACULOS = Path(os.environ.get("PSX_ORACULOS", FACULDADE / "tools" / "oraculos"))
REFERENCIAS = Path(os.environ.get("PSX_REFERENCIAS", ORACULOS / "referencias"))
DS_REGTEST = Path(
    os.environ.get(
        "PSX_DS_REGTEST", ORACULOS / "duckstation" / "build" / "bin" / "duckstation-regtest"
    )
)
BIOS = Path(os.environ.get("PSX_BIOS", REPO / "bios" / "SCPH1001.BIN"))


class ErroDeJogo(ValueError):
    """JSON de jogo invalido."""


@dataclass(frozen=True)
class Botao:
    b: str
    t: float
    dur: float


@dataclass(frozen=True)
class Checkpoint:
    t: float
    marco: str
    checa: tuple
    limiar: float = LIMIAR_PADRAO
    janela: float = JANELA_S


@dataclass(frozen=True)
class Jogo:
    id: str
    nome: str
    cue: str
    duracao_s: float
    botoes: tuple
    extra_cli: tuple
    memcard: bool
    checkpoints: tuple
    min_nao_zero: float
    arquivo: Path = field(compare=False)

    @property
    def cue_path(self) -> Path:
        return ROMS / self.cue

    def assinatura_roteiro(self) -> str:
        """Hash do que muda a referencia: disco, duracao, botoes e cartao."""
        dados = {
            "cue": self.cue,
            "duracao_s": self.duracao_s,
            "botoes": [[b.b, b.t, b.dur] for b in self.botoes],
            "memcard": self.memcard,
        }
        texto = json.dumps(dados, sort_keys=True).encode()
        return hashlib.sha256(texto).hexdigest()[:16]


def _exige(cond: bool, arquivo: Path, msg: str) -> None:
    if not cond:
        raise ErroDeJogo(f"{arquivo.name}: {msg}")


def _numero(v) -> bool:
    return isinstance(v, (int, float)) and not isinstance(v, bool)


def _botao(bruto: dict, arq: Path, duracao: float) -> Botao:
    _exige(isinstance(bruto, dict), arq, f"botao invalido: {bruto!r}")
    nome = str(bruto.get("b", "")).lower()
    _exige(nome in BOTOES, arq, f"botao desconhecido '{nome}' (validos: {', '.join(BOTOES)})")
    t, dur = bruto.get("t"), bruto.get("dur", 0.1)
    _exige(_numero(t) and 0 <= t < duracao, arq, f"botao {nome}: t={t!r} fora de [0, duracao_s)")
    _exige(_numero(dur) and dur > 0, arq, f"botao {nome}@{t}: dur={dur!r} tem de ser > 0")
    return Botao(nome, float(t), float(dur))


def _checkpoint(bruto: dict, arq: Path, duracao: float) -> Checkpoint:
    _exige(isinstance(bruto, dict), arq, f"checkpoint invalido: {bruto!r}")
    t = bruto.get("t")
    _exige(_numero(t) and 0 < t <= duracao, arq, f"checkpoint t={t!r} fora de (0, duracao_s]")
    checa = tuple(bruto.get("checa", CHECAGENS_VALIDAS))
    ruins = [c for c in checa if c not in CHECAGENS_VALIDAS]
    _exige(not ruins, arq, f"checkpoint t={t}: checagens desconhecidas {ruins}")
    limiar = bruto.get("limiar", LIMIAR_PADRAO)
    janela = bruto.get("janela", JANELA_S)
    _exige(_numero(limiar) and 0 <= limiar <= 1, arq, f"checkpoint t={t}: limiar em [0, 1]")
    _exige(_numero(janela) and janela > 0, arq, f"checkpoint t={t}: janela > 0")
    return Checkpoint(float(t), str(bruto.get("marco", "")), checa, float(limiar), float(janela))


def carrega_jogo(arquivo: Path) -> Jogo:
    try:
        bruto = json.loads(arquivo.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as e:
        raise ErroDeJogo(f"{arquivo.name}: nao li o JSON: {e}") from e
    _exige(isinstance(bruto, dict), arquivo, "o topo tem de ser um objeto")
    for campo in ("id", "nome", "cue", "duracao_s"):
        _exige(campo in bruto, arquivo, f"falta o campo '{campo}'")
    duracao = bruto["duracao_s"]
    _exige(_numero(duracao) and duracao > 0, arquivo, "duracao_s tem de ser > 0")
    _exige(bruto["id"] == arquivo.stem, arquivo, f"id '{bruto['id']}' difere do nome do arquivo")
    botoes = tuple(
        sorted((_botao(b, arquivo, duracao) for b in bruto.get("botoes", [])), key=lambda b: b.t)
    )
    checkpoints = tuple(
        sorted(
            (_checkpoint(c, arquivo, duracao) for c in bruto.get("checkpoints", [])),
            key=lambda c: c.t,
        )
    )
    _exige(bool(checkpoints), arquivo, "sem checkpoints")
    extra = bruto.get("extra_cli", [])
    _exige(isinstance(extra, list) and all(isinstance(x, str) for x in extra), arquivo,
           "extra_cli tem de ser lista de strings")
    audio = bruto.get("audio", {}) or {}
    min_nao_zero = audio.get("min_nao_zero", 0.0)
    _exige(_numero(min_nao_zero) and 0 <= min_nao_zero <= 1, arquivo, "audio.min_nao_zero em [0, 1]")
    return Jogo(
        id=bruto["id"],
        nome=bruto["nome"],
        cue=bruto["cue"],
        duracao_s=float(duracao),
        botoes=botoes,
        extra_cli=tuple(extra),
        memcard=bool(bruto.get("memcard", False)),
        checkpoints=checkpoints,
        min_nao_zero=float(min_nao_zero),
        arquivo=arquivo,
    )


def lista_jogos(ids: list | None) -> list:
    arquivos = sorted(JOGOS_DIR.glob("*.json"))
    if ids:
        existentes = {a.stem: a for a in arquivos}
        faltando = [i for i in ids if i not in existentes]
        if faltando:
            raise ErroDeJogo(f"jogos sem JSON em {JOGOS_DIR}: {', '.join(faltando)}")
        arquivos = [existentes[i] for i in ids]
    return [carrega_jogo(a) for a in arquivos]


def cartao_formatado() -> bytes:
    """Memory card de 128 KiB formatado e vazio (o que o DuckStation insere num slot novo)."""
    quadro = 128
    cartao = bytearray(128 * 1024)

    def poe(i: int, dados: bytes) -> None:
        q = bytearray(quadro)
        q[: len(dados)] = dados
        x = 0
        for b in q[:127]:
            x ^= b
        q[127] = x
        cartao[i * quadro:(i + 1) * quadro] = q

    poe(0, b"MC")
    for i in range(1, 16):
        poe(i, bytes([0xA0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0xFF]))
    for i in range(16, 36):
        poe(i, bytes([0xFF] * 4 + [0] * 4 + [0xFF, 0xFF]))
    poe(63, b"MC")
    return bytes(cartao)
