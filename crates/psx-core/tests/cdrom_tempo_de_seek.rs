use psx_core::cdrom::Cdrom;

// Relogio da CPU: 33.868.800 Hz. Serve so pra traduzir ciclo em milissegundo nas mensagens
// e nos limites de ordem de grandeza cobrados aqui.
const CLOCK: u64 = 33_868_800;
const MS: u64 = CLOCK / 1000;

fn ack(cd: &Cdrom) {
    cd.write8(0, 1, None, None);
    cd.write8(3, 0x07, None, None);
    cd.write8(0, 0, None, None);
}

fn setloc(cd: &Cdrom, mm: u8, ss: u8, ff: u8) {
    cd.write8(0, 0, None, None);
    cd.write8(2, mm, None, None);
    cd.write8(2, ss, None, None);
    cd.write8(2, ff, None, None);
    cd.write8(1, 0x02, None, None);
    cd.deliver_first(None, None);
    ack(cd);
}

fn seek(cd: &Cdrom, mm: u8, ss: u8, ff: u8) -> u64 {
    setloc(cd, mm, ss, ff);
    cd.write8(1, 0x15, None, None);
    cd.deliver_first(None, None);
    let ciclos = cd.second_response_cycles();
    ack(cd);
    cd.deliver_second_now(None, None);
    ack(cd);
    ciclos
}

fn drive() -> Cdrom {
    let cd = Cdrom::new();
    cd.insert_disc();
    cd
}

#[test]
fn seek_longo_custa_muito_mais_que_seek_curto() {
    let curto = seek(&drive(), 0x00, 0x02, 0x00);
    let longo = seek(&drive(), 0x59, 0x00, 0x00);
    assert!(
        longo > curto * 4,
        "06-cdrom.md L2077-2078: a 2a resposta de Seek 'depend[s] on seek time'. \
         Da mesma origem (00:00:00), buscar 00:02:00 (150 quadros) e buscar 59:00:00 \
         (265.500 quadros) NAO pode custar quase o mesmo. curto={curto} longo={longo}"
    );
}

#[test]
fn seek_curto_tem_ordem_de_grandeza_de_dezenas_de_milissegundos() {
    let curto = seek(&drive(), 0x00, 0x02, 0x00);
    assert!(
        curto > 10 * MS,
        "06-cdrom.md L2070: um Pause em single speed ja custa 021181Ch (~64 ms). Um SEEK \
         inteiro nao pode ser ~0,5 ms. curto={curto} ciclos (~{} ms)",
        curto / MS
    );
    assert!(
        curto < 100 * MS,
        "seek curto nao pode passar de ~100 ms. curto={curto} ciclos"
    );
}

#[test]
fn seek_longo_fica_na_casa_das_centenas_de_milissegundos() {
    let longo = seek(&drive(), 0x59, 0x00, 0x00);
    assert!(
        longo > 100 * MS && longo < 1000 * MS,
        "varredura quase completa do disco: centenas de ms, nunca segundos. \
         longo={longo} ciclos (~{} ms)",
        longo / MS
    );
}

#[test]
fn seeks_de_mesma_distancia_nao_repetem_o_mesmo_tempo() {
    let cd = drive();
    let mut tempos = Vec::new();
    for _ in 0..3 {
        seek(&cd, 0x10, 0x00, 0x00);
        tempos.push(seek(&cd, 0x20, 0x00, 0x00));
    }
    assert!(
        tempos[0] != tempos[1] || tempos[1] != tempos[2],
        "06-cdrom.md L2069-2076: toda temporizacao medida do drive tem FAIXA (GetID \
         0004922h..0004c2bh), nao valor unico. Tres seeks da MESMA distancia devolveram \
         exatamente o mesmo numero: {tempos:?}"
    );
}

#[test]
fn a_variacao_do_seek_e_pequena_perto_do_nominal() {
    let cd = drive();
    let mut tempos = Vec::new();
    for _ in 0..8 {
        seek(&cd, 0x10, 0x00, 0x00);
        tempos.push(seek(&cd, 0x20, 0x00, 0x00));
    }
    let min = tempos.iter().copied().min().unwrap_or(0);
    let max = tempos.iter().copied().max().unwrap_or(0);
    assert!(
        max - min < min / 8,
        "a faixa medida na spec fica dentro de poucos por cento da media \
         (L2069: 4922h..4c2bh em torno de 4a00h). Aqui min={min} max={max}"
    );
}

#[test]
fn mesma_sequencia_de_seeks_e_reproduzivel() {
    let roteiro = |cd: &Cdrom| {
        let mut v = Vec::new();
        for _ in 0..6 {
            v.push(seek(cd, 0x05, 0x10, 0x00));
            v.push(seek(cd, 0x30, 0x00, 0x00));
            v.push(seek(cd, 0x00, 0x02, 0x00));
        }
        v
    };
    let a = roteiro(&drive());
    let b = roteiro(&drive());
    assert_eq!(
        a, b,
        "R3/save state: a variacao por seek tem que sair de PRNG deterministico no estado. \
         Duas execucoes iguais deram sequencias diferentes."
    );
}

#[test]
fn seek_conta_a_distancia_a_partir_de_onde_a_cabeca_parou() {
    let cd = drive();
    seek(&cd, 0x30, 0x00, 0x00);
    let vizinho = seek(&cd, 0x30, 0x01, 0x00);
    let cd2 = drive();
    let longe = seek(&cd2, 0x30, 0x01, 0x00);
    assert!(
        vizinho * 4 < longe,
        "depois de um seek a cabeca esta no alvo: o seek seguinte pra 1 segundo adiante e' \
         curto. vizinho={vizinho} longe={longe}"
    );
}

// Faixas medidas no log de CD do DuckStation (logs/onda/comparativo/*/ds/*/cdrom.log):
// ~51 ms por salto de trilha curto, ~100 ms ate alguns milhares de setores, e o treno
// (sled) de 340 ms a 17,5 mil setores ate 800 ms a 263 mil.
fn ms(ciclos: u64) -> u64 {
    ciclos / MS
}

fn setmode(cd: &Cdrom, modo: u8) {
    cd.write8(0, 0, None, None);
    cd.write8(2, modo, None, None);
    cd.write8(1, 0x0E, None, None);
    cd.deliver_first(None, None);
    ack(cd);
}

#[test]
fn seek_de_meio_disco_leva_meio_segundo() {
    let cd = drive();
    seek(&cd, 0x00, 0x02, 0x00);
    let t = seek(&cd, 0x30, 0x02, 0x00);
    assert!(
        (480..680).contains(&ms(t)),
        "30 minutos de distancia (135 mil setores): a referencia leva ~575 ms. Aqui {} ms",
        ms(t)
    );
}

#[test]
fn seek_de_disco_inteiro_leva_perto_de_800_ms() {
    let cd = drive();
    seek(&cd, 0x00, 0x02, 0x00);
    let t = seek(&cd, 0x58, 0x30, 0x00);
    assert!(
        (700..900).contains(&ms(t)),
        "58 minutos de distancia: a referencia leva ~800 ms. Aqui {} ms",
        ms(t)
    );
}

#[test]
fn seek_de_poucos_minutos_ja_usa_o_treno() {
    let cd = drive();
    seek(&cd, 0x00, 0x02, 0x00);
    let t = seek(&cd, 0x04, 0x02, 0x00);
    assert!(
        (300..420).contains(&ms(t)),
        "4 minutos (18 mil setores): a referencia leva ~340 ms. Aqui {} ms",
        ms(t)
    );
}

#[test]
fn seek_de_dois_segundos_custa_um_salto_de_trilha() {
    let cd = drive();
    seek(&cd, 0x00, 0x02, 0x00);
    let t = seek(&cd, 0x00, 0x04, 0x00);
    assert!(
        (40..70).contains(&ms(t)),
        "150 setores: a referencia leva ~51 ms. Aqui {} ms",
        ms(t)
    );
}

#[test]
fn seek_de_meio_minuto_custa_dois_saltos() {
    let cd = drive();
    seek(&cd, 0x00, 0x02, 0x00);
    let t = seek(&cd, 0x00, 0x32, 0x00);
    assert!(
        (85..120).contains(&ms(t)),
        "2250 setores: a referencia leva ~100 ms. Aqui {} ms",
        ms(t)
    );
}

#[test]
fn avancar_poucos_setores_so_espera_eles_passarem() {
    let cd = drive();
    setmode(&cd, 0x80);
    cd.set_clock(CLOCK);
    seek(&cd, 0x00, 0x02, 0x00);
    let t = seek(&cd, 0x00, 0x02, 0x04);
    assert!(
        (20..40).contains(&ms(t)),
        "4 setores adiante em 2x: a cabeca so espera o disco girar (~27 ms na referencia). \
         Aqui {} ms",
        ms(t)
    );
}
