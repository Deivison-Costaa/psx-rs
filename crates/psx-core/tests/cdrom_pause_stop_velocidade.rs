use psx_core::cdrom::Cdrom;

// 06-cdrom.md "Second Response" (L2066-2075): medias de hardware em ciclos da CPU.
const PAUSE_1X: u64 = 0x021_181C;
const PAUSE_1X_MIN: u64 = 0x020_EAEF;
const PAUSE_1X_MAX: u64 = 0x021_6E3C;
const PAUSE_2X: u64 = 0x010_BD93;
const PAUSE_2X_MIN: u64 = 0x010_477A;
const PAUSE_2X_MAX: u64 = 0x011_B302;
const PAUSE_PARADO: u64 = 0x1DF2;
const STOP_1X: u64 = 0x0D3_8ACA;
const STOP_2X: u64 = 0x18A_6076;
const STOP_2X_MIN: u64 = 0x184_476B;
const STOP_2X_MAX: u64 = 0x192_B306;
const CLOCK: u64 = 33_868_800;
const MS: u64 = CLOCK / 1000;

fn ack(cd: &Cdrom) {
    cd.write8(0, 1, None, None);
    cd.write8(3, 0x07, None, None);
    cd.write8(0, 0, None, None);
}

fn comando(cd: &Cdrom, op: u8, params: &[u8]) -> u64 {
    cd.write8(0, 0, None, None);
    for &p in params {
        cd.write8(2, p, None, None);
    }
    cd.write8(1, op, None, None);
    cd.deliver_first(None, None);
    let ciclos = cd.second_response_cycles();
    ack(cd);
    ciclos
}

fn lendo_em(modo: u8) -> Cdrom {
    lendo_em_msf(modo, [0x00, 0x02, 0x10])
}

fn lendo_em_msf(modo: u8, msf: [u8; 3]) -> Cdrom {
    let cd = Cdrom::new();
    cd.insert_disc();
    comando(&cd, 0x0E, &[modo]);
    cd.set_clock(2 * CLOCK);
    comando(&cd, 0x02, &msf);
    comando(&cd, 0x06, &[]);
    cd.deliver_second_now(None, None);
    ack(&cd);
    cd
}

#[test]
fn pause_lendo_em_dupla_velocidade_leva_o_tempo_de_2x() {
    let cd = lendo_em(0x80);
    let ciclos = comando(&cd, 0x09, &[]);
    assert!(
        (PAUSE_2X_MIN..=PAUSE_2X_MAX).contains(&ciclos),
        "Pause (double speed) mede {PAUSE_2X:#x} ciclos (~5 setores a 2x), nao o valor de 1x \
         {PAUSE_1X:#x}. Aqui {ciclos:#x}"
    );
}

#[test]
fn pause_lendo_em_velocidade_normal_continua_com_o_tempo_de_1x() {
    let cd = lendo_em(0x00);
    let ciclos = comando(&cd, 0x09, &[]);
    assert!(
        (PAUSE_1X_MIN..=PAUSE_1X_MAX).contains(&ciclos),
        "Pause (single speed) mede {PAUSE_1X:#x} no inicio do disco. Aqui {ciclos:#x}"
    );
}

#[test]
fn pause_na_borda_externa_do_disco_demora_mais() {
    let cd = lendo_em_msf(0x80, [0x50, 0x00, 0x00]);
    let ciclos = comando(&cd, 0x09, &[]);
    assert!(
        ciclos > 80 * MS && ciclos < 120 * MS,
        "a referencia (DuckStation) leva ~102 ms num Pause a 2x por volta de 50:00, contra \
         ~39 ms no inicio do disco. Aqui {} ms",
        ciclos / MS
    );
}

#[test]
fn pause_ja_pausado_nao_depende_da_velocidade() {
    let cd = lendo_em(0x80);
    comando(&cd, 0x09, &[]);
    cd.deliver_second_now(None, None);
    ack(&cd);
    let ciclos = comando(&cd, 0x09, &[]);
    assert_eq!(
        ciclos, PAUSE_PARADO,
        "Pause (when paused) = {PAUSE_PARADO:#x}"
    );
}

#[test]
fn stop_em_dupla_velocidade_demora_mais_que_em_1x() {
    let cd = lendo_em(0x80);
    let ciclos = comando(&cd, 0x08, &[]);
    assert!(
        (STOP_2X_MIN..=STOP_2X_MAX).contains(&ciclos),
        "Stop (double speed) mede {STOP_2X:#x} ciclos: o motor freia de uma rotacao maior. \
         Aqui {ciclos:#x}"
    );
}

#[test]
fn stop_em_velocidade_normal_continua_com_o_tempo_de_1x() {
    let cd = lendo_em(0x00);
    let ciclos = comando(&cd, 0x08, &[]);
    assert_eq!(ciclos, STOP_1X, "Stop (single speed) = {STOP_1X:#x}");
}
