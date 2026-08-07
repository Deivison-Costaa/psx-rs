mod support;

use psx_core::bus::{Bus, BusRead, BusWrite};
use psx_core::cdrom_bin_cue::{DiscLayout, TrackInfo, TrackType};
use support::asm;

const CD_BASE: u32 = 0x1F80_1800;
const ESPERA_PRIMEIRA_RESPOSTA: u32 = 0x1_4000;
// § 06-cdrom.md L2077-2078: o primeiro setor do ReadN chega depois da BUSCA (distancia +
// variacao por seek); os goldens de delay(N) contam da CADENCIA em diante, nao dela.
// § INT1 Rate (06-cdrom.md L2093-2101): SystemClock*930h/4/44100Hz em velocidade normal.
const CADENCIA: u32 = 451_584;
const FRAMES: usize = 96;

// Traducao de "delay(N)" da spec (06-cdrom.md L2136-2168) para tempo concreto. Os golden
// values dizem em que setor o controlador estava quando a CPU voltou a processar:
//   delay(1) -> pula de 0:2:1 para 0:2:6 => o mais novo COMPLETO era o 6, logo o relogio
//               tinha de estar dentro do intervalo do setor 6 (entre 6 e 7 cadencias);
//   delay(2) -> slot do setor 1 ja sobrescrito pelo 9 (1+8) e o mais novo era o 11;
//   delay(3) -> slot do setor 1 sobrescrito pelo 17 (1+8+8) e o mais novo completo era 16.
// Como a contagem comeca no INT1 do setor 1 (uma cadencia depois do primeiro setor), os
// atrasos abaixo sao 5.5, 10.5 e 15.5 cadencias — meia cadencia para cair no MEIO do
// intervalo alvo em vez de na fronteira.
const ATRASO_CURTO: u32 = 11 * (CADENCIA / 2);
const ATRASO_MEDIO: u32 = 21 * (CADENCIA / 2);
const ATRASO_LONGO: u32 = 31 * (CADENCIA / 2);

fn cd_read(bus: &Bus, offset: u32) -> u8 {
    bus.read8::<BusRead>(CD_BASE + offset)
}

fn cd_write(bus: &mut Bus, offset: u32, val: u8) {
    bus.write8::<BusWrite>(CD_BASE + offset, val);
}

fn set_bank(bus: &mut Bus, b: u8) {
    cd_write(bus, 0, b);
}

fn send_command(bus: &mut Bus, cmd: u8) {
    set_bank(bus, 0);
    cd_write(bus, 1, cmd);
    bus.tick_timers(ESPERA_PRIMEIRA_RESPOSTA);
}

fn param_write(bus: &mut Bus, val: u8) {
    set_bank(bus, 0);
    cd_write(bus, 2, val);
}

fn ack(bus: &mut Bus) {
    set_bank(bus, 1);
    cd_write(bus, 3, 0x07);
    set_bank(bus, 0);
}

fn hintsts(bus: &mut Bus) -> u8 {
    set_bank(bus, 1);
    let val = cd_read(bus, 3);
    set_bank(bus, 0);
    val & 0x07
}

fn result_read(bus: &mut Bus) -> u8 {
    set_bank(bus, 0);
    cd_read(bus, 1)
}

fn layout() -> DiscLayout {
    DiscLayout {
        bin_path: "test.bin".to_string(),
        tracks: vec![TrackInfo {
            number: 1,
            file: "test.bin".to_string(),
            start_lba: 150,
            track_type: TrackType::Mode1_2048,
            index01_mm: 0,
            index01_ss: 2,
            index01_ff: 0,
            index00_mm: None,
            index00_ss: None,
            index00_ff: None,
            pregap_mm: None,
            pregap_ss: None,
            pregap_ff: None,
        }],
    }
}

fn int_to_bcd(v: u8) -> u8 {
    ((v / 10) << 4) | (v % 10)
}

/// Cada setor carrega o proprio MSF no cabecalho (00Ch-00Fh do setor cru), que e' o que a
/// spec chama de "sector header for 0:2:N" nos casos medidos.
fn disco_com_cabecalho_por_setor() -> Vec<u8> {
    let mut bin = vec![0u8; FRAMES * 2352];
    for frame in 0..FRAMES {
        let base = frame * 2352;
        for b in bin.iter_mut().skip(base + 1).take(10) {
            *b = 0xFF;
        }
        let ff = (frame % 75) as u8;
        let ss = 2 + (frame / 75) as u8;
        bin[base + 0x0C] = 0x00;
        bin[base + 0x0D] = int_to_bcd(ss);
        bin[base + 0x0E] = int_to_bcd(ff);
        bin[base + 0x0F] = 0x01;
    }
    bin
}

fn bus_com_disco() -> Bus {
    let mut bus = asm::bus_with_bios_empty();
    bus.inject_disc(layout(), disco_com_cabecalho_por_setor());
    bus.cdrom_mut().insert_disc();
    bus
}

/// Setmode(20h) para o buffer de dados comecar no cabecalho do setor
/// (§ Setmode 06-cdrom.md L685-703: bit5 = WholeSectorExceptSyncBytes), Setloc(0:2:0) e
/// ReadN, com todos os INT3 ja reconhecidos.
fn setmode_setloc_e_read(bus: &mut Bus) {
    param_write(bus, 0x20);
    send_command(bus, 0x0E);
    let _ = result_read(bus);
    ack(bus);

    param_write(bus, 0x00);
    param_write(bus, 0x02);
    param_write(bus, 0x00);
    send_command(bus, 0x02);
    let _ = result_read(bus);
    ack(bus);

    send_command(bus, 0x06);
    let _ = result_read(bus);
    ack(bus);
}

/// Pede o setor (BFRD) e le os tres primeiros bytes do buffer: o cabecalho amm/ass/asect.
fn le_cabecalho(bus: &mut Bus) -> (u8, u8, u8) {
    set_bank(bus, 0);
    cd_write(bus, 3, 0x80);
    let mm = cd_read(bus, 2);
    let ss = cd_read(bus, 2);
    let ff = cd_read(bus, 2);
    (mm, ss, ff)
}

/// "Process INT1" da spec: consome o status, pede o dado, le o cabecalho e da ack.
fn processa_int1(bus: &mut Bus) -> (u8, u8, u8) {
    assert_eq!(
        hintsts(bus),
        1,
        "esperava um INT1 de setor de dado pendente neste ponto"
    );
    let _ = result_read(bus);
    let cabecalho = le_cabecalho(bus);
    ack(bus);
    cabecalho
}

/// § GetlocL (06-cdrom.md L1057-1059): "the GetlocL command returns the header and
/// subheader of the <newest> buffered sector".
fn le_resposta_getloc(bus: &mut Bus) -> (u8, u8, u8) {
    assert_eq!(hintsts(bus), 3, "GetlocL responde INT3, nao INT5 de erro");
    let mm = result_read(bus);
    let ss = result_read(bus);
    let ff = result_read(bus);
    (mm, ss, ff)
}

/// § Sector Buffer Test Cases (06-cdrom.md L2129-2135): processando cada INT1 na hora, os
/// setores saem em sequencia.
#[test]
fn sem_atraso_os_setores_saem_em_sequencia() {
    let mut bus = bus_com_disco();
    setmode_setloc_e_read(&mut bus);

    let ciclos_da_busca = bus.cdrom().second_response_cycles() as u32;
    bus.tick_timers(ciclos_da_busca);
    assert_eq!(processa_int1(&mut bus), (0x00, 0x02, 0x00));
    bus.tick_timers(CADENCIA);
    assert_eq!(processa_int1(&mut bus), (0x00, 0x02, 0x01));
    bus.tick_timers(CADENCIA);
    assert_eq!(processa_int1(&mut bus), (0x00, 0x02, 0x02));
    bus.tick_timers(CADENCIA);
    assert_eq!(
        processa_int1(&mut bus),
        (0x00, 0x02, 0x03),
        "sem atraso o controlador entrega 0:2:0, 0:2:1, 0:2:2, 0:2:3 (06-cdrom.md L2130-2134)"
    );
}

/// § Sector Buffer (06-cdrom.md L2112-2117): "after processing the INT1 for the oldest
/// sector ... it appears to jump directly to INT1 for the newest sector (skipping all
/// other unprocessed sectors)". Caso medido em L2140-2146.
#[test]
fn atraso_curto_entrega_o_mais_antigo_e_pula_pro_mais_novo() {
    let mut bus = bus_com_disco();
    setmode_setloc_e_read(&mut bus);

    let ciclos_da_busca = bus.cdrom().second_response_cycles() as u32;
    bus.tick_timers(ciclos_da_busca);
    assert_eq!(processa_int1(&mut bus), (0x00, 0x02, 0x00));

    bus.tick_timers(CADENCIA);
    bus.tick_timers(ATRASO_CURTO);

    assert_eq!(
        processa_int1(&mut bus),
        (0x00, 0x02, 0x01),
        "o slot do setor 1 ainda nao foi sobrescrito: o INT1 mais antigo entrega 0:2:1"
    );
    assert_eq!(
        processa_int1(&mut bus),
        (0x00, 0x02, 0x06),
        "e o proximo INT1 pula direto pro setor mais novo, 0:2:6, sem passar pelos \
         2..5 (06-cdrom.md L2143-2145)"
    );
    bus.tick_timers(CADENCIA);
    assert_eq!(
        processa_int1(&mut bus),
        (0x00, 0x02, 0x07),
        "com a leitura de novo em dia, volta a sequencia: 0:2:7"
    );
}

/// § Sector Buffer (06-cdrom.md L2147-2157): a PROVA dos 8 slots — "sector buffer can hold
/// 8 sectors (as the sector 1 slot is overwritten by sector 9)".
#[test]
fn atraso_medio_mostra_o_slot_do_setor_1_sobrescrito_pelo_setor_9() {
    let mut bus = bus_com_disco();
    setmode_setloc_e_read(&mut bus);

    let ciclos_da_busca = bus.cdrom().second_response_cycles() as u32;
    bus.tick_timers(ciclos_da_busca);
    assert_eq!(processa_int1(&mut bus), (0x00, 0x02, 0x00));

    bus.tick_timers(CADENCIA);
    bus.tick_timers(ATRASO_MEDIO);

    assert_eq!(
        processa_int1(&mut bus),
        (0x00, 0x02, 0x09),
        "o INT1 mais antigo aponta pro SLOT do setor 1, e esse slot ja carrega o setor \
         9 (1+8): e' isso que prova que o ring tem 8 posicoes (06-cdrom.md L2154-2155)"
    );
    assert_eq!(
        processa_int1(&mut bus),
        (0x00, 0x02, 0x11),
        "depois do mais antigo vem o mais novo, 0:2:11 (BCD)"
    );
    bus.tick_timers(CADENCIA);
    assert_eq!(processa_int1(&mut bus), (0x00, 0x02, 0x12));
}

/// § Sector Buffer (06-cdrom.md L2158-2168): caso especial em que o setor 17 aparece duas
/// vezes — "the sector 1 slot (which was overwritten by sector 9, and apparently then half
/// overwritten by sector 17)".
#[test]
fn atraso_longo_repete_o_setor_17_como_a_spec_mediu() {
    let mut bus = bus_com_disco();
    setmode_setloc_e_read(&mut bus);

    let ciclos_da_busca = bus.cdrom().second_response_cycles() as u32;
    bus.tick_timers(ciclos_da_busca);
    assert_eq!(processa_int1(&mut bus), (0x00, 0x02, 0x00));

    bus.tick_timers(CADENCIA);
    bus.tick_timers(ATRASO_LONGO);

    assert_eq!(
        processa_int1(&mut bus),
        (0x00, 0x02, 0x17),
        "o slot do setor 1 esta recebendo o setor 17 (1+8+8) neste instante"
    );
    assert_eq!(
        processa_int1(&mut bus),
        (0x00, 0x02, 0x16),
        "o pulo vai pro mais novo COMPLETO, 0:2:16 — o 17 ainda estava chegando"
    );
    bus.tick_timers(CADENCIA);
    assert_eq!(
        processa_int1(&mut bus),
        (0x00, 0x02, 0x17),
        "e o 17, agora completo, aparece de novo (06-cdrom.md L2164-2166)"
    );
    bus.tick_timers(CADENCIA);
    assert_eq!(processa_int1(&mut bus), (0x00, 0x02, 0x18));
}

/// § Sector Buffer VS GetlocL Response Tests (06-cdrom.md L2172-2180).
#[test]
fn getloc_l_logo_apos_o_int1_devolve_o_mesmo_setor() {
    let mut bus = bus_com_disco();
    setmode_setloc_e_read(&mut bus);

    let ciclos_da_busca = bus.cdrom().second_response_cycles() as u32;
    bus.tick_timers(ciclos_da_busca);
    assert_eq!(processa_int1(&mut bus), (0x00, 0x02, 0x00));

    send_command(&mut bus, 0x10);
    assert_eq!(
        le_resposta_getloc(&mut bus),
        (0x00, 0x02, 0x00),
        "sem atraso, o mais novo do buffer e' o mesmo que o INT1 acabou de entregar"
    );
    ack(&mut bus);

    bus.tick_timers(CADENCIA);
    assert_eq!(processa_int1(&mut bus), (0x00, 0x02, 0x01));
    bus.tick_timers(CADENCIA);
    assert_eq!(processa_int1(&mut bus), (0x00, 0x02, 0x02));
    bus.tick_timers(CADENCIA);
    assert_eq!(processa_int1(&mut bus), (0x00, 0x02, 0x03));
}

/// § Sector Buffer VS GetlocL Response Tests (06-cdrom.md L2182-2191): com atraso ANTES do
/// GetlocL, o INT1 entrega o mais ANTIGO (0:2:1) e o GetlocL responde o mais NOVO (0:2:6).
#[test]
fn getloc_l_depois_do_atraso_devolve_o_mais_novo_e_o_int1_o_mais_antigo() {
    let mut bus = bus_com_disco();
    setmode_setloc_e_read(&mut bus);

    let ciclos_da_busca = bus.cdrom().second_response_cycles() as u32;
    bus.tick_timers(ciclos_da_busca);
    assert_eq!(processa_int1(&mut bus), (0x00, 0x02, 0x00));

    bus.tick_timers(CADENCIA);
    bus.tick_timers(ATRASO_CURTO);

    send_command(&mut bus, 0x10);
    assert_eq!(
        processa_int1(&mut bus),
        (0x00, 0x02, 0x01),
        "o INT1 pendente vem antes da resposta do comando retido"
    );
    assert_eq!(
        le_resposta_getloc(&mut bus),
        (0x00, 0x02, 0x06),
        "o GetlocL nao devolve o setor que o INT1 acabou de entregar, e sim o mais NOVO \
         do buffer (06-cdrom.md L1057-1059)"
    );
    ack(&mut bus);

    assert_eq!(processa_int1(&mut bus), (0x00, 0x02, 0x06));
    bus.tick_timers(CADENCIA);
    assert_eq!(processa_int1(&mut bus), (0x00, 0x02, 0x07));
}

/// § Sector Buffer VS GetlocL Response Tests (06-cdrom.md L2193-2202): com o atraso DEPOIS
/// do GetlocL, a resposta ja estava travada em 0:2:0 e o INT1 seguinte pula pro 0:2:5 — o
/// setor 1 foi perdido sem aviso porque o INT3 ocupava a linha (L2118-2126).
#[test]
fn getloc_l_antes_do_atraso_trava_a_resposta_e_o_int1_seguinte_pula() {
    let mut bus = bus_com_disco();
    setmode_setloc_e_read(&mut bus);

    let ciclos_da_busca = bus.cdrom().second_response_cycles() as u32;
    bus.tick_timers(ciclos_da_busca);
    assert_eq!(processa_int1(&mut bus), (0x00, 0x02, 0x00));

    send_command(&mut bus, 0x10);
    assert_eq!(le_resposta_getloc(&mut bus), (0x00, 0x02, 0x00));

    bus.tick_timers(ATRASO_CURTO);
    ack(&mut bus);

    assert_eq!(
        processa_int1(&mut bus),
        (0x00, 0x02, 0x05),
        "os setores 1..4 sumiram sem flag de overrun nem INT de erro"
    );
    bus.tick_timers(CADENCIA);
    assert_eq!(processa_int1(&mut bus), (0x00, 0x02, 0x06));
    bus.tick_timers(CADENCIA);
    assert_eq!(processa_int1(&mut bus), (0x00, 0x02, 0x07));
}
