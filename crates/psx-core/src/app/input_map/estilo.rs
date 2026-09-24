use super::Entrada;

const SONY: u16 = 0x054c;
const NINTENDO: u16 = 0x057e;

/// Como o fabricante escreve os botoes de face: o mesmo botao fisico "de baixo" e o
/// Cruz num DualShock, o A num Xbox e o B num Switch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Estilo {
    PlayStation,
    #[default]
    Xbox,
    Nintendo,
}

impl Estilo {
    pub fn detecta(nome: &str, fabricante: Option<u16>) -> Estilo {
        let nome = nome.to_ascii_lowercase();
        let tem = |palavras: &[&str]| palavras.iter().any(|p| nome.contains(p));
        if fabricante == Some(SONY)
            || tem(&[
                "sony",
                "playstation",
                "dualshock",
                "dualsense",
                "ps3",
                "ps4",
                "ps5",
            ])
        {
            Estilo::PlayStation
        } else if fabricante == Some(NINTENDO) || tem(&["nintendo", "switch", "joy-con"]) {
            Estilo::Nintendo
        } else {
            Estilo::Xbox
        }
    }

    fn faces(&self) -> [&'static str; 4] {
        match self {
            Estilo::PlayStation => ["✖ Cruz", "○ Círculo", "⏶ Triângulo", "◻ Quadrado"],
            Estilo::Xbox => ["A", "B", "Y", "X"],
            Estilo::Nintendo => ["B", "A", "X", "Y"],
        }
    }

    fn ombros(&self) -> [&'static str; 4] {
        match self {
            Estilo::PlayStation => ["L1", "L2", "R1", "R2"],
            Estilo::Xbox => ["LB", "LT", "RB", "RT"],
            Estilo::Nintendo => ["L", "ZL", "R", "ZR"],
        }
    }
}

fn eixo(numero: u8, positivo: bool) -> String {
    let lado = if numero < 2 { "esquerdo" } else { "direito" };
    let seta = match (numero % 2 == 0, positivo) {
        (true, false) => "⬅",
        (true, true) => "➡",
        (false, false) => "⬇",
        (false, true) => "⬆",
    };
    if numero < 4 {
        format!("Analógico {lado} {seta}")
    } else {
        format!("Eixo {numero} {seta}")
    }
}

impl Entrada {
    pub fn rotulo(&self, estilo: Estilo) -> String {
        let [sul, leste, norte, oeste] = estilo.faces();
        let [l1, l2, r1, r2] = estilo.ombros();
        let fixo = match self {
            Entrada::Sul => sul,
            Entrada::Leste => leste,
            Entrada::Norte => norte,
            Entrada::Oeste => oeste,
            Entrada::L1 => l1,
            Entrada::L2 => l2,
            Entrada::R1 => r1,
            Entrada::R2 => r2,
            Entrada::L3 => "Clique do analógico esquerdo",
            Entrada::R3 => "Clique do analógico direito",
            Entrada::Select => "Select/Back",
            Entrada::Start => "Start",
            Entrada::Modo => "Botão central (Home/PS)",
            Entrada::DpadCima => "Direcional ⬆",
            Entrada::DpadBaixo => "Direcional ⬇",
            Entrada::DpadEsquerda => "Direcional ⬅",
            Entrada::DpadDireita => "Direcional ➡",
            Entrada::EixoNegativo(n) => return eixo(*n, false),
            Entrada::EixoPositivo(n) => return eixo(*n, true),
        };
        fixo.to_string()
    }
}
