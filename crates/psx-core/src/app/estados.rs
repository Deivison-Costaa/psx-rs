use crate::app::saves::serial_limpo;

pub const SLOTS: u8 = 10;
pub const MINIATURA_LARGURA: usize = 160;
pub const MINIATURA_ALTURA: usize = 120;

const BYTES_POR_PIXEL: usize = 4;
const SEGUNDOS_POR_DIA: i64 = 86_400;
const DIAS_POR_ERA: i64 = 146_097;
const DIAS_ATE_1970_DESDE_0000_03_01: i64 = 719_468;

pub fn nome_do_estado(serial: &str, slot: u8) -> String {
    format!("{}-{slot}.state", serial_limpo(serial))
}

pub fn nome_da_miniatura(serial: &str, slot: u8) -> String {
    format!("{}-{slot}.png", serial_limpo(serial))
}

/// Amostragem por vizinho mais proximo para 160x120 fixo: a tela do PS1 e sempre 4:3,
/// qualquer que seja a resolucao do modo de video.
pub fn miniatura(rgba: &[u8], largura: usize, altura: usize) -> Option<Vec<u8>> {
    if largura == 0 || altura == 0 || rgba.len() != largura * altura * BYTES_POR_PIXEL {
        return None;
    }
    let mut fora = Vec::with_capacity(MINIATURA_LARGURA * MINIATURA_ALTURA * BYTES_POR_PIXEL);
    for y in 0..MINIATURA_ALTURA {
        let sy = y * altura / MINIATURA_ALTURA;
        for x in 0..MINIATURA_LARGURA {
            let sx = x * largura / MINIATURA_LARGURA;
            let i = (sy * largura + sx) * BYTES_POR_PIXEL;
            fora.extend_from_slice(&rgba[i..i + 3]);
            fora.push(u8::MAX);
        }
    }
    Some(fora)
}

/// Dias desde 1970 para ano/mes/dia (algoritmo `civil_from_days` de Howard Hinnant).
fn data_civil(dias: i64) -> (i64, i64, i64) {
    let z = dias + DIAS_ATE_1970_DESDE_0000_03_01;
    let era = z.div_euclid(DIAS_POR_ERA);
    let doe = z - era * DIAS_POR_ERA;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let dia = doy - (153 * mp + 2) / 5 + 1;
    let mes = if mp < 10 { mp + 3 } else { mp - 9 };
    let ano = yoe + era * 400 + i64::from(mes <= 2);
    (ano, mes, dia)
}

pub fn data_hora(segundos: i64) -> String {
    let dias = segundos.div_euclid(SEGUNDOS_POR_DIA);
    let resto = segundos.rem_euclid(SEGUNDOS_POR_DIA);
    let (ano, mes, dia) = data_civil(dias);
    format!(
        "{dia:02}/{mes:02}/{ano:04} {:02}:{:02}",
        resto / 3600,
        (resto % 3600) / 60
    )
}

pub fn deslocamento_de(texto: &str) -> Option<i64> {
    let t = texto.trim();
    let (sinal, digitos) = match t.as_bytes().first()? {
        b'+' => (1, &t[1..]),
        b'-' => (-1, &t[1..]),
        _ => return None,
    };
    if digitos.len() != 4 || !digitos.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let horas: i64 = digitos[..2].parse().ok()?;
    let minutos: i64 = digitos[2..].parse().ok()?;
    Some(sinal * (horas * 3600 + minutos * 60))
}
