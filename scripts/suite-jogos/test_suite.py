"""Testes das checagens da suite com imagens sinteticas: python3 -m unittest scripts/suite-jogos/test_suite.py"""

import json
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np
from PIL import Image

sys.path.insert(0, str(Path(__file__).resolve().parent))

import checagens  # noqa: E402
import comum  # noqa: E402
import nosso  # noqa: E402
import referencia  # noqa: E402
from referencia import QuadroRef, Referencia  # noqa: E402


def _png(pasta: Path, nome: str, arr: np.ndarray) -> Path:
    p = pasta / nome
    Image.fromarray(arr.astype(np.uint8)).save(p)
    return p


def _cena(semente: int, tam=(224, 512)) -> np.ndarray:
    rng = np.random.default_rng(semente)
    base = rng.integers(0, 256, size=(-(-tam[0] // 16), -(-tam[1] // 16), 3))
    return np.kron(base, np.ones((16, 16, 1)))[: tam[0], : tam[1]]


class TestChecagens(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.d = Path(self._tmp.name)
        checagens.cinza.cache_clear()
        checagens.tamanho.cache_clear()

    def tearDown(self):
        self._tmp.cleanup()

    def _ref(self, quadros):
        return Referencia(self.d, {}, tuple(QuadroRef(t, p) for t, p in quadros))

    def test_mesma_cena_passa_e_cena_diferente_reprova(self):
        a = _png(self.d, "a.png", _cena(1))
        b = _png(self.d, "b.png", _cena(2))
        self.assertGreater(checagens.similaridade(a, a), 0.99)
        self.assertLess(checagens.similaridade(a, b), 0.3)

    def test_parecido_escolhe_o_melhor_frame_da_janela(self):
        nosso = _png(self.d, "n.png", _cena(1))
        outro = _png(self.d, "o.png", _cena(2))
        igual = _png(self.d, "i.png", _cena(1))
        longe = _png(self.d, "l.png", _cena(1))
        cp = comum.Checkpoint(10.0, "x", ("parecido_ds", "resolucao"))
        ref = self._ref([(9.0, outro), (12.5, igual), (20.0, longe)])
        r = checagens.avalia_checkpoint(cp, [(10.0, nosso)], ref, [])
        self.assertTrue(r.aprovado, r.falhas)
        self.assertEqual(r.ref_t, 12.5)
        ref_ruim = self._ref([(9.0, outro), (20.0, longe)])
        r = checagens.avalia_checkpoint(cp, [(10.0, nosso)], ref_ruim, [])
        self.assertFalse(r.aprovado)
        self.assertIn("parecido_ds", r.falhas[0])

    def test_resolucao_diferente_reprova(self):
        nosso = _png(self.d, "n.png", _cena(1, (240, 368)))
        ref = _png(self.d, "r.png", _cena(1, (240, 365)))
        cp = comum.Checkpoint(5.0, "x", ("resolucao",))
        r = checagens.avalia_checkpoint(cp, [(5.0, nosso)], self._ref([(5.0, ref)]), [])
        self.assertEqual(r.falhas, ["resolucao: nossa (368, 240) != referencia (365, 240)"])

    def test_muda_reprova_imagem_congelada_e_laco_pequeno(self):
        a = _png(self.d, "a.png", _cena(1))
        b = _png(self.d, "b.png", _cena(2))
        cp = comum.Checkpoint(3.0, "x", ("muda",))
        pcs_vivos = [(t / 100, 0x80010000 + (t * 97 % 4096) * 4) for t in range(301)]
        pcs_presos = [(t / 100, 0x80020000 + (t % 4) * 4) for t in range(301)]
        congelado = [(t * 0.5, a) for t in range(7)]
        vivo = [(t * 0.5, a if t % 2 else b) for t in range(7)]
        self.assertTrue(checagens.avalia_checkpoint(cp, vivo, None, pcs_vivos).aprovado)
        r = checagens.avalia_checkpoint(cp, congelado, None, pcs_vivos)
        self.assertIn("congelada", r.falhas[0])
        r = checagens.avalia_checkpoint(cp, vivo, None, pcs_presos)
        self.assertIn("laco pequeno", r.falhas[0])

    def test_tela_preta_nao_casa_com_preta_se_a_janela_tem_conteudo(self):
        preta = np.zeros((224, 512, 3))
        nosso = _png(self.d, "n.png", preta)
        ref_preta = _png(self.d, "rp.png", preta)
        ref_mapa = _png(self.d, "rm.png", _cena(3))
        cp = comum.Checkpoint(10.0, "x", ("parecido_ds",))
        r = checagens.avalia_checkpoint(
            cp, [(10.0, nosso)], self._ref([(9.0, ref_preta), (10.0, ref_mapa)]), []
        )
        self.assertFalse(r.aprovado)
        r = checagens.avalia_checkpoint(cp, [(10.0, nosso)], self._ref([(9.0, ref_preta)]), [])
        self.assertTrue(r.aprovado)
        self.assertTrue(r.avisos)

    def test_sem_frame_nosso_reprova(self):
        ref = _png(self.d, "r.png", _cena(1))
        cp = comum.Checkpoint(30.0, "x", ("parecido_ds",))
        r = checagens.avalia_checkpoint(cp, [(10.0, ref)], self._ref([(30.0, ref)]), [])
        self.assertIn("sem frame nosso", r.falhas[0])


class TestJogoJson(unittest.TestCase):
    def _escreve(self, d: Path, dados: dict) -> Path:
        p = d / f"{dados['id']}.json"
        p.write_text(json.dumps(dados), encoding="utf-8")
        return p

    def test_valida_e_converte_para_o_regtest(self):
        with tempfile.TemporaryDirectory() as t:
            dados = {"id": "x", "nome": "X", "cue": "x.cue", "duracao_s": 20,
                     "botoes": [{"b": "cross", "t": 10, "dur": 1}, {"b": "start", "t": 5}],
                     "checkpoints": [{"t": 15, "marco": "m"}]}
            jogo = comum.carrega_jogo(self._escreve(Path(t), dados))
            self.assertEqual([b.b for b in jogo.botoes], ["start", "cross"])
            self.assertEqual(referencia.ds_input(jogo), "start@300-305,cross@599-658")
            self.assertEqual(jogo.checkpoints[0].checa, comum.CHECAGENS_VALIDAS)

    def test_recusa_botao_desconhecido_e_checkpoint_fora(self):
        with tempfile.TemporaryDirectory() as t:
            base = {"id": "x", "nome": "X", "cue": "x.cue", "duracao_s": 20,
                    "checkpoints": [{"t": 15}]}
            with self.assertRaises(comum.ErroDeJogo):
                comum.carrega_jogo(self._escreve(Path(t), {**base, "botoes": [{"b": "z", "t": 1}]}))
            with self.assertRaises(comum.ErroDeJogo):
                comum.carrega_jogo(self._escreve(Path(t), {**base, "checkpoints": [{"t": 25}]}))

    def test_assinatura_muda_com_os_botoes(self):
        with tempfile.TemporaryDirectory() as t:
            base = {"id": "x", "nome": "X", "cue": "x.cue", "duracao_s": 20,
                    "checkpoints": [{"t": 15}]}
            a = comum.carrega_jogo(self._escreve(Path(t), base)).assinatura_roteiro()
            b = comum.carrega_jogo(self._escreve(Path(t), {**base, "botoes": [{"b": "start", "t": 1}]}))
            c = comum.carrega_jogo(self._escreve(Path(t), {**base, "checkpoints": [{"t": 10}]}))
            self.assertNotEqual(a, b.assinatura_roteiro())
            self.assertEqual(a, c.assinatura_roteiro())

    def test_crash_json_do_repositorio_e_valido(self):
        jogo = comum.carrega_jogo(comum.JOGOS_DIR / "crash.json")
        self.assertGreaterEqual(len(jogo.checkpoints), 5)

    def test_todos_os_json_do_repositorio_sao_validos(self):
        for arq in sorted(comum.JOGOS_DIR.glob("*.json")):
            with self.subTest(jogo=arq.stem):
                jogo = comum.carrega_jogo(arq)
                self.assertGreaterEqual(len(jogo.checkpoints), 4)

    def test_extra_cli_troca_a_marca_pela_pasta_das_roms(self):
        with tempfile.TemporaryDirectory() as t:
            dados = {"id": "x", "nome": "X", "cue": "x.cue", "duracao_s": 20,
                     "extra_cli": ["--swap-disc", "{PSX_ROMS}/D2/d2.cue@10s:3s"],
                     "checkpoints": [{"t": 15}]}
            jogo = comum.carrega_jogo(self._escreve(Path(t), dados))
            args = nosso.argumentos(jogo, Path("cli"), Path(t), None)
            self.assertEqual(args[-2:], ["--swap-disc", f"{comum.ROMS}/D2/d2.cue@10s:3s"])


if __name__ == "__main__":
    unittest.main()
