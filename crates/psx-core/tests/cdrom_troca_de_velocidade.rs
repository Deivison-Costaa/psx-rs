use psx_core::cdrom::Cdrom;

// Relogio da CPU: 33.868.800 Hz. Os numeros de referencia vem do log de CD do DuckStation
// (logs/onda/comparativo/*/ds/*/cdrom.log): 1x->2x custa 20.321.280 ciclos (0,6 s) e
// 2x->1x custa 23.708.160 (0,7 s) quando o drive ja esta girando.
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

fn setmode(cd: &Cdrom, modo: u8) {
    comando(cd, 0x0E, &[modo]);
}

fn read_n(cd: &Cdrom, mm: u8, ss: u8, ff: u8) -> u64 {
    comando(cd, 0x02, &[mm, ss, ff]);
    comando(cd, 0x06, &[])
}

fn drive() -> Cdrom {
    let cd = Cdrom::new();
    cd.insert_disc();
    cd
}

#[test]
fn acelerar_para_dupla_velocidade_atrasa_a_proxima_leitura() {
    let cd = drive();
    setmode(&cd, 0x80);
    let ciclos = read_n(&cd, 0x00, 0x02, 0x16);
    assert!(
        ciclos > 580 * MS && ciclos < 720 * MS,
        "Setmode bit7 (06-cdrom.md L687) com o motor girando: o drive troca de rotacao antes \
         de ler. A referencia leva 639 ms deste ReadN (troca 1x->2x + salto de trilha). \
         Aqui {} ms",
        ciclos / MS
    );
}

#[test]
fn setmode_sem_mudar_a_velocidade_nao_atrasa() {
    let cd = drive();
    setmode(&cd, 0x00);
    let ciclos = read_n(&cd, 0x00, 0x02, 0x16);
    assert!(
        ciclos < 100 * MS,
        "o bit7 nao mudou: nenhuma troca de rotacao. Aqui {} ms",
        ciclos / MS
    );
}

#[test]
fn a_troca_de_velocidade_ja_concluida_nao_cobra_de_novo() {
    let cd = drive();
    setmode(&cd, 0x80);
    cd.set_clock(CLOCK);
    let ciclos = read_n(&cd, 0x00, 0x02, 0x16);
    assert!(
        ciclos < 100 * MS,
        "Setmode 1 s antes do ReadN: a rotacao ja estabilizou. Aqui {} ms",
        ciclos / MS
    );
}

#[test]
fn leitura_no_meio_da_troca_espera_so_o_que_falta() {
    let cd = drive();
    setmode(&cd, 0x80);
    cd.set_clock(300 * MS);
    let ciclos = read_n(&cd, 0x00, 0x02, 0x16);
    assert!(
        ciclos > 300 * MS && ciclos < 420 * MS,
        "300 ms da troca de 600 ms ja passaram: o ReadN espera os ~300 ms restantes mais o \
         seek. Aqui {} ms",
        ciclos / MS
    );
}

#[test]
fn desacelerar_para_velocidade_normal_demora_mais_que_acelerar() {
    let sobe = drive();
    setmode(&sobe, 0x80);
    let acelerando = read_n(&sobe, 0x00, 0x02, 0x16);

    let desce = drive();
    setmode(&desce, 0x80);
    desce.set_clock(2 * CLOCK);
    setmode(&desce, 0x00);
    let desacelerando = read_n(&desce, 0x00, 0x02, 0x16);
    assert!(
        desacelerando > acelerando + 50 * MS,
        "a referencia mede 0,7 s para 2x->1x e 0,6 s para 1x->2x. \
         acelerando={} ms desacelerando={} ms",
        acelerando / MS,
        desacelerando / MS
    );
}

#[test]
fn com_o_motor_parado_so_o_spin_up_conta() {
    let cd = drive();
    comando(&cd, 0x08, &[]);
    cd.deliver_second_now(None, None);
    ack(&cd);
    setmode(&cd, 0x80);
    let parado = read_n(&cd, 0x00, 0x02, 0x16);

    let outro = drive();
    comando(&outro, 0x08, &[]);
    outro.deliver_second_now(None, None);
    ack(&outro);
    let sem_setmode = read_n(&outro, 0x00, 0x02, 0x16);
    assert!(
        parado < sem_setmode + 20 * MS,
        "motor desligado: o spin-up ja leva o disco direto para a rotacao pedida, sem somar \
         a troca. com_setmode={} ms sem_setmode={} ms",
        parado / MS,
        sem_setmode / MS
    );
}

#[test]
fn init_volta_para_velocidade_normal_e_paga_a_troca() {
    let cd = drive();
    setmode(&cd, 0x80);
    cd.set_clock(2 * CLOCK);
    let init = comando(&cd, 0x0A, &[]);
    assert!(
        init > 600 * MS && init < 900 * MS,
        "06-cdrom.md L535-537: Init faz mode=20h; saindo de 2x o drive desacelera \
         (referencia: 0,7 s + seek). Aqui {} ms",
        init / MS
    );
    cd.deliver_second_now(None, None);
    ack(&cd);
    cd.set_clock(4 * CLOCK);
    setmode(&cd, 0x80);
    let leitura = read_n(&cd, 0x00, 0x02, 0x16);
    assert!(
        leitura > 500 * MS,
        "depois do Init o modo voltou a 1x: pedir 2x de novo troca a rotacao outra vez. \
         Aqui {} ms",
        leitura / MS
    );
}

#[test]
fn init_repetido_durante_a_desaceleracao_nao_cancela_a_resposta_pendente() {
    let cd = drive();
    setmode(&cd, 0x80);
    cd.set_clock(2 * CLOCK);
    comando(&cd, 0x0A, &[]);
    cd.take_second_request();
    cd.take_second_dirty();
    cd.write8(0, 0, None, None);
    cd.write8(1, 0x0A, None, None);
    cd.deliver_first(None, None);
    assert!(
        !cd.take_second_dirty(),
        "06-cdrom.md L538-540: Init repetido com a 2a resposta pendente e' descartado; ele \
         nao pode cancelar a resposta ja agendada (o MGS repete Init a cada 20 ms e ficava \
         preso no logo)"
    );
    assert_eq!(cd.intsts(), 0, "o Init descartado nao gera INT3 nem INT5");
    cd.deliver_second_now(None, None);
    assert_eq!(
        cd.intsts(),
        2,
        "a 2a resposta do primeiro Init chega normalmente"
    );
}
