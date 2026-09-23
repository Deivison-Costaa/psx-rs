mod support;

use psx_core::bus::{Bus, BusRead, BusWrite};
use psx_core::cdrom_bin_cue::{DiscLayout, TrackInfo, TrackType};
use psx_core::cpu::Cpu;
use psx_core::snapshot;
use support::asm;

// Esperas em ciclos de CPU (33,8688 MHz). O drive nao tem tempo exato de freio/partida na
// spec; as janelas aqui so exigem a ordem de grandeza (fracao de segundo ate uns segundos).
const CD_BASE: u32 = 0x1F80_1800;
const ESPERA_PRIMEIRA_RESPOSTA: u32 = 0x2_0000;
const SEGUNDO: u32 = 33_868_800;

fn cd_read(bus: &Bus, offset: u32) -> u8 {
    bus.read8::<BusRead>(CD_BASE + offset)
}

fn cd_write(bus: &mut Bus, offset: u32, val: u8) {
    bus.write8::<BusWrite>(CD_BASE + offset, val);
}

fn hintsts(bus: &mut Bus) -> u8 {
    cd_write(bus, 0, 1);
    let val = cd_read(bus, 3);
    cd_write(bus, 0, 0);
    val & 0x7
}

fn ack(bus: &mut Bus) {
    cd_write(bus, 0, 1);
    cd_write(bus, 3, 0x1F);
    cd_write(bus, 0, 0);
}

fn resposta(bus: &mut Bus) -> Vec<u8> {
    cd_write(bus, 0, 0);
    let mut fora = Vec::new();
    while cd_read(bus, 0) & (1 << 5) != 0 && fora.len() < 16 {
        fora.push(cd_read(bus, 1));
    }
    fora
}

fn espera(bus: &mut Bus, ciclos: u32) {
    let mut falta = ciclos;
    while falta > 0 {
        let passo = falta.min(0x1000);
        bus.tick_timers(passo);
        falta -= passo;
    }
}

fn comando(bus: &mut Bus, cmd: u8, params: &[u8]) -> (u8, Vec<u8>) {
    cd_write(bus, 0, 0);
    for p in params {
        cd_write(bus, 2, *p);
    }
    cd_write(bus, 1, cmd);
    espera(bus, ESPERA_PRIMEIRA_RESPOSTA);
    let int = hintsts(bus);
    let bytes = resposta(bus);
    ack(bus);
    (int, bytes)
}

fn getstat(bus: &mut Bus) -> u8 {
    let (int, bytes) = comando(bus, 0x01, &[]);
    assert_eq!(int, 3, "Nop sempre responde INT3(stat)");
    bytes[0]
}

fn layout() -> DiscLayout {
    DiscLayout {
        bin_path: "t.bin".to_string(),
        tracks: vec![TrackInfo {
            number: 1,
            file: "t.bin".to_string(),
            start_lba: 0,
            track_type: TrackType::Mode2_2352,
            index01_mm: 0,
            index01_ss: 0,
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

fn imagem(marca: u8) -> Vec<u8> {
    let mut bin = vec![0u8; 4 * 2352];
    for setor in 0..4 {
        let base = setor * 2352;
        for b in bin.iter_mut().skip(base + 1).take(10) {
            *b = 0xFF;
        }
        let quadro = 150 + setor as u8;
        bin[base + 0x0C] = 0x00;
        bin[base + 0x0D] = 0x02;
        bin[base + 0x0E] = quadro - 150;
        bin[base + 0x0F] = 0x02;
        bin[base + 0x18] = marca;
    }
    bin
}

fn bus_com_disco() -> Bus {
    let mut bus = asm::bus_with_bios_empty();
    bus.inject_disc(layout(), imagem(0xAA));
    bus.cdrom_mut().insert_disc();
    cd_write(&mut bus, 0, 1);
    cd_write(&mut bus, 2, 0x1F);
    cd_write(&mut bus, 0, 0);
    bus
}

fn abre(bus: &mut Bus) -> (u8, Vec<u8>) {
    bus.open_lid();
    espera(bus, 0x100);
    let int = hintsts(bus);
    let bytes = resposta(bus);
    ack(bus);
    (int, bytes)
}

#[test]
fn abrir_a_tampa_dispara_int5_com_erro_08_mesmo_sem_comando() {
    let mut bus = bus_com_disco();
    let (int, bytes) = abre(&mut bus);
    assert_eq!(
        int, 5,
        "06-cdrom.md (Status code): 'When the shell is opened, INT5 is triggered regardless \
         of whether a command was executing or not'"
    );
    assert_eq!(
        bytes,
        vec![0x01, 0x08],
        "erro 08h = 'Drive door became opened'; o 1o byte medido em hardware (ps1-tests \
         cdrom/disc-swap psx.log: 'Got IRQ 5 (expected 5), status 0x01') e so o bit de erro"
    );
    assert!(bus.lid_open());
}

#[test]
fn getstat_com_tampa_aberta_mostra_shell_e_motor_ate_o_disco_parar() {
    let mut bus = bus_com_disco();
    abre(&mut bus);
    assert_eq!(
        getstat(&mut bus),
        0x12,
        "psx.log do disc-swap: logo depois de abrir, Getstat = 12h (shell aberto, disco \
         ainda girando)"
    );
    espera(&mut bus, 2 * SEGUNDO);
    assert_eq!(
        getstat(&mut bus),
        0x10,
        "psx.log do disc-swap: depois o motor para e sobra so o bit 4 (10h)"
    );
    assert_eq!(
        getstat(&mut bus),
        0x10,
        "06-cdrom.md (Nop): o Nop so zera o bit 4 'unless the shell is still opened'"
    );
}

#[test]
fn comandos_que_precisam_do_disco_falham_com_int5_80_enquanto_aberta() {
    let mut bus = bus_com_disco();
    abre(&mut bus);
    espera(&mut bus, 2 * SEGUNDO);
    for (cmd, params) in [
        (0x02u8, vec![0x00u8, 0x02, 0x00]),
        (0x06, vec![]),
        (0x15, vec![]),
        (0x11, vec![]),
        (0x1A, vec![]),
    ] {
        let (int, bytes) = comando(&mut bus, cmd, &params);
        assert_eq!(
            (int, bytes),
            (5, vec![0x11, 0x80]),
            "comando {cmd:02X}h com a porta aberta: 06-cdrom.md (GetID) 'Door Open \
             INT5(11h,80h)'; o erro 80h vale para 02h..09h, 0Bh..0Dh, 10h..16h, 1Ah, 1Bh"
        );
    }
}

#[test]
fn fechar_a_tampa_mantem_o_bit_4_ate_o_primeiro_getstat_e_gira_de_novo() {
    let mut bus = bus_com_disco();
    abre(&mut bus);
    espera(&mut bus, 2 * SEGUNDO);
    bus.close_lid();
    assert!(!bus.lid_open());
    assert_eq!(
        getstat(&mut bus),
        0x10,
        "06-cdrom.md (Status code): bit 4 = 'Once shell open (0=Closed, 1=Is/was Open)'; o \
         Nop devolve o stat antes de zerar"
    );
    assert_eq!(
        getstat(&mut bus),
        0x00,
        "psx.log do disc-swap termina em 'Getstat: 0x00': porta fechada, bit 4 zerado pelo \
         Nop anterior, motor ainda em spin-up (bit 1 = 0 'or in spin-up phase')"
    );
    espera(&mut bus, 4 * SEGUNDO);
    assert_eq!(
        getstat(&mut bus),
        0x02,
        "disco girando de novo: motor ligado"
    );
}

#[test]
fn getid_durante_o_spin_up_responde_int5_01_80() {
    let mut bus = bus_com_disco();
    abre(&mut bus);
    espera(&mut bus, 2 * SEGUNDO);
    bus.close_lid();
    getstat(&mut bus);
    let (int, bytes) = comando(&mut bus, 0x1A, &[]);
    assert_eq!(
        (int, bytes),
        (5, vec![0x01, 0x80]),
        "06-cdrom.md (GetID): 'Spin-up INT5(01h,80h)'"
    );
}

#[test]
fn leitura_em_andamento_para_quando_a_tampa_abre() {
    let mut bus = bus_com_disco();
    comando(&mut bus, 0x02, &[0x00, 0x02, 0x00]);
    comando(&mut bus, 0x06, &[]);
    abre(&mut bus);
    for _ in 0..40 {
        espera(&mut bus, SEGUNDO / 20);
        assert_eq!(
            hintsts(&mut bus),
            0,
            "com a porta aberta o disco para: nenhuma INT1 de setor depois do INT5"
        );
    }
}

#[test]
fn trocar_o_disco_com_a_tampa_aberta_faz_o_drive_ler_o_disco_novo() {
    let mut bus = bus_com_disco();
    abre(&mut bus);
    bus.swap_disc(layout(), imagem(0x55));
    bus.close_lid();
    espera(&mut bus, 4 * SEGUNDO);
    getstat(&mut bus);
    assert_eq!(getstat(&mut bus), 0x02);

    comando(&mut bus, 0x0E, &[0x00]);
    comando(&mut bus, 0x02, &[0x00, 0x02, 0x00]);
    let (int, _) = comando(&mut bus, 0x06, &[]);
    assert_eq!(int, 3);
    let mut int1 = 0;
    for _ in 0..80 {
        espera(&mut bus, SEGUNDO / 40);
        int1 = hintsts(&mut bus);
        if int1 != 0 {
            break;
        }
    }
    assert_eq!(int1, 1, "o ReadN no disco novo entrega setor");
    cd_write(&mut bus, 0, 0);
    cd_write(&mut bus, 3, 0x80);
    let primeiro = cd_read(&bus, 2);
    assert_eq!(
        primeiro, 0x55,
        "o setor tem de vir da imagem nova, nao da antiga (AAh)"
    );
}

#[test]
fn ejetar_deixa_o_drive_sem_disco_depois_de_fechar() {
    let mut bus = bus_com_disco();
    abre(&mut bus);
    bus.eject_disc();
    bus.close_lid();
    espera(&mut bus, 4 * SEGUNDO);
    getstat(&mut bus);
    assert_eq!(
        getstat(&mut bus),
        0x00,
        "sem disco o motor nao volta a girar"
    );
    let (int, bytes) = comando(&mut bus, 0x02, &[0x00, 0x02, 0x00]);
    assert_eq!(
        (int, bytes),
        (5, vec![0x01, 0x80]),
        "06-cdrom.md: 80h 'also appears if no disk inserted at all'"
    );
}

#[test]
fn save_state_com_a_porta_aberta_volta_com_a_porta_aberta_e_o_motor_freando() {
    let mut bus = bus_com_disco();
    abre(&mut bus);
    let estado = snapshot::salva(&Cpu::new(), &bus, "SLUS-00000").expect("codificar");

    let mut outro = bus_com_disco();
    let mut cpu = Cpu::new();
    snapshot::carrega(&mut cpu, &mut outro, &estado, "SLUS-00000").expect("carregar");
    assert!(outro.lid_open());
    assert_eq!(getstat(&mut outro), 0x12);
    espera(&mut outro, 2 * SEGUNDO);
    assert_eq!(
        getstat(&mut outro),
        0x10,
        "o fim do freio do motor e um evento do scheduler e tem de viajar no save state"
    );
}
