mod support;

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use psx_core::gte::Gte;

const LOG: &str = "tests/exes/ps1-tests/gte-fuzz/gte_valid_0xc0ffee_50.log";

#[derive(Debug, Clone)]
struct Caso {
    comando: String,
    palavra: u32,
    entrada: [u32; 64],
    saida: [u32; 64],
}

fn caminho() -> Option<PathBuf> {
    let p = support::repo_root().join(LOG);
    p.exists().then_some(p)
}

fn hex(s: &str) -> Option<u32> {
    u32::from_str_radix(s.trim().trim_start_matches("0x"), 16).ok()
}

fn indice(linha: &str) -> Option<(usize, u32)> {
    let (esq, dir) = linha.split_once('=')?;
    let n = esq
        .split_once("r[")?
        .1
        .split_once(']')?
        .0
        .trim()
        .parse()
        .ok()?;
    Some((n, hex(dir)?))
}

/// `GTE 0x01 RTPS (sf=0, lm=1, tx=2, vx=1, mx=0)` -> palavra de funcao do COP2.
fn palavra_de_comando(linha: &str) -> Option<(String, u32)> {
    let resto = linha.strip_prefix("GTE ")?;
    let (op_txt, resto) = resto.split_once(' ')?;
    let op = hex(op_txt)?;
    let (nome, campos) = resto.split_once(" (")?;
    let mut palavra = op & 0x3F;
    for campo in campos.trim_end_matches(')').split(',') {
        let (chave, valor) = campo.split_once('=')?;
        let v: u32 = valor.trim().parse().ok()?;
        palavra |= match chave.trim() {
            "sf" => v << 19,
            "lm" => v << 10,
            "tx" => v << 13,
            "vx" => v << 15,
            "mx" => v << 17,
            _ => 0,
        };
    }
    Some((format!("{op:#04x} {nome}"), palavra))
}

fn carrega() -> Vec<Caso> {
    let Some(p) = caminho() else {
        return Vec::new();
    };
    let texto = fs::read_to_string(p).expect("log de fuzz do GTE");
    let mut casos = Vec::new();
    let mut entrada = [0u32; 64];
    let mut saida = [0u32; 64];
    let mut pendente: Option<(String, u32)> = None;
    let mut lidos_saida = 0usize;

    for linha in texto.lines() {
        let l = linha.trim();
        if let Some(resto) = l.strip_prefix("> ") {
            if let Some((n, v)) = indice(resto) {
                if n < 64 {
                    entrada[n] = v;
                }
            }
        } else if let Some(resto) = l.strip_prefix("< ") {
            if let Some((n, v)) = indice(resto) {
                if n < 64 {
                    saida[n] = v;
                    lidos_saida += 1;
                }
            }
            if lidos_saida == 64 {
                if let Some((comando, palavra)) = pendente.take() {
                    casos.push(Caso {
                        comando,
                        palavra,
                        entrada,
                        saida,
                    });
                }
                lidos_saida = 0;
            }
        } else if l.starts_with("GTE 0x") && l.contains('(') && l.contains("sf=") {
            pendente = palavra_de_comando(l);
            lidos_saida = 0;
        }
    }
    casos
}

fn roda(caso: &Caso) -> [u32; 64] {
    let mut gte = Gte::new();
    for (i, v) in caso.entrada.iter().enumerate() {
        if i < 32 {
            gte.write_data(i, *v);
        } else {
            gte.write_control(i - 32, *v);
        }
    }
    gte.execute_command(caso.palavra);
    let mut saida = [0u32; 64];
    for (i, slot) in saida.iter_mut().enumerate() {
        *slot = if i < 32 {
            gte.read_data(i)
        } else {
            gte.read_control(i - 32)
        };
    }
    saida
}

struct Placar {
    total: usize,
    exatos: usize,
    por_comando: BTreeMap<String, (usize, usize)>,
    divergencias: BTreeMap<usize, usize>,
}

fn mede(casos: &[Caso]) -> Placar {
    let mut placar = Placar {
        total: 0,
        exatos: 0,
        por_comando: BTreeMap::new(),
        divergencias: BTreeMap::new(),
    };
    for caso in casos {
        let medido = roda(caso);
        let ok = medido == caso.saida;
        placar.total += 1;
        placar.exatos += usize::from(ok);
        let e = placar
            .por_comando
            .entry(caso.comando.clone())
            .or_insert((0, 0));
        e.1 += 1;
        e.0 += usize::from(ok);
        if !ok {
            for i in 0..64 {
                if medido[i] != caso.saida[i] {
                    *placar.divergencias.entry(i).or_insert(0) += 1;
                }
            }
        }
    }
    placar
}

#[test]
fn placar_do_fuzz_de_hardware_do_gte() {
    let casos = carrega();
    if casos.is_empty() {
        eprintln!("{LOG} ausente (gitignored) — teste ignorado");
        return;
    }
    let placar = mede(&casos);
    for (cmd, (ok, total)) in &placar.por_comando {
        eprintln!("# GTE-FUZZ {cmd}: {ok}/{total}");
    }
    let mut piores: Vec<_> = placar.divergencias.iter().collect();
    piores.sort_by_key(|(_, n)| std::cmp::Reverse(**n));
    for (reg, n) in piores.iter().take(8) {
        eprintln!("# GTE-FUZZ registrador r[{reg}] diverge em {n} casos");
    }
    eprintln!(
        "# GTE-FUZZ TOTAL {}/{} ({:.1}%)",
        placar.exatos,
        placar.total,
        100.0 * placar.exatos as f64 / placar.total as f64
    );
    assert_eq!(
        placar.total, 1100,
        "o log tem 22 comandos x 50 casos; um numero diferente e erro de parse"
    );
    let falhos: Vec<&String> = placar
        .por_comando
        .iter()
        .filter(|(_, (ok, total))| ok != total)
        .map(|(cmd, _)| cmd)
        .collect();
    assert_eq!(
        placar.exatos, placar.total,
        "o GTE tem de bater com o hardware em todos os 64 registradores. Comandos com \
         divergencia: {falhos:?}"
    );
}

/// Primeira divergencia de um comando, campo a campo — usado para depurar o placar.
#[test]
fn primeira_divergencia_por_comando() {
    let casos = carrega();
    if casos.is_empty() {
        eprintln!("{LOG} ausente (gitignored) — teste ignorado");
        return;
    }
    let mut ja_visto: BTreeMap<String, bool> = BTreeMap::new();
    for caso in &casos {
        if ja_visto.contains_key(&caso.comando) {
            continue;
        }
        let medido = roda(caso);
        if medido == caso.saida {
            continue;
        }
        ja_visto.insert(caso.comando.clone(), true);
        eprintln!(
            "# DIVERGE {} (palavra {:#010x})",
            caso.comando, caso.palavra
        );
        for i in 0..64 {
            if medido[i] != caso.saida[i] {
                eprintln!(
                    "#   r[{i}] esperado {:#010x} medido {:#010x}",
                    caso.saida[i], medido[i]
                );
            }
        }
    }
}
