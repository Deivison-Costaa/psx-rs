use psx_core::bus::{Bios, Bus, BusRead, BusWrite, Ram};
use psx_core::cpu::Cpu;
use psx_core::snapshot::{self, SnapshotError};

const SERIAL: &str = "SLUS-00005";
const RAM_ALVO: u32 = 0x0010_0000;
const GP0: u32 = 0x1F80_1810;
const GP1: u32 = 0x1F80_1814;
const TIMER1_CONTADOR: u32 = 0x1F80_1110;
const SPU_VOZ0_VOL: u32 = 0x1F80_1C00;
const I_MASK: u32 = 0x1F80_1074;
const DMA6_MADR: u32 = 0x1F80_10E0;

fn maquina() -> (Cpu, Bus) {
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS sintetica de 512 KiB");
    let mut bus = Bus::new(Ram::new(), bios);
    bus.sio_mut().connect_digital_pad(true);
    (Cpu::new(), bus)
}

/// Escreve em todo subsistema que o snapshot precisa carregar. Um campo que ficar de fora
/// do `Snapshot` volta com o valor rabiscado depois, e a asserção correspondente cai.
fn suja(cpu: &mut Cpu, bus: &mut Bus, semente: u32) {
    cpu.regs[8] = 0xDEAD_0000 | semente;
    cpu.regs[16] = semente.wrapping_mul(7);
    cpu.pc = 0x8001_0000 | (semente & 0xFFF) << 4;
    cpu.cop0[12] = 0x0000_0401 | semente;

    bus.write32::<BusWrite>(RAM_ALVO, 0xCAFE_0000 | semente);
    bus.write32::<BusWrite>(I_MASK, semente & 0x7FF);
    bus.write32::<BusWrite>(DMA6_MADR, 0x0010_0000 | (semente & 0xFF) << 4);
    bus.write32::<BusWrite>(TIMER1_CONTADOR, semente & 0xFFFF);
    bus.write32::<BusWrite>(GP1, 0x0500_0000 | (semente & 0x3FF));
    bus.write32::<BusWrite>(GP0, 0xA000_0000);
    bus.write32::<BusWrite>(GP0, 0x0000_0000);
    bus.write32::<BusWrite>(GP0, 0x0001_0002);
    bus.write32::<BusWrite>(GP0, semente & 0x7FFF_7FFF);
    bus.spu_mut()
        .write16(SPU_VOZ0_VOL, (semente & 0x3FFF) as u16);
    bus.gte_mut().write_control(5, 0x0BAD_0000 | semente);
    bus.sio().set_buttons(!(semente as u16 & 0x00FF));
    bus.write32::<BusWrite>(GP0, 0xE100_0000 | (semente & 0x7FF));
}

fn retrato(cpu: &Cpu, bus: &Bus) -> Vec<u32> {
    vec![
        cpu.regs[8],
        cpu.regs[16],
        cpu.pc,
        cpu.cop0[12],
        bus.read32::<BusRead>(RAM_ALVO),
        bus.read32::<BusRead>(I_MASK),
        bus.read32::<BusRead>(DMA6_MADR),
        u32::from(bus.gpu().vram_pixel(0, 0)),
        u32::from(bus.spu().read16(SPU_VOZ0_VOL)),
        bus.gte().read_control(5),
        bus.gpu().stat(),
        bus.total_cycles() as u32,
    ]
}

fn salvo(cpu: &Cpu, bus: &Bus) -> Vec<u8> {
    snapshot::salva(cpu, bus, SERIAL)
}

#[test]
fn estado_volta_inteiro_depois_de_ser_rabiscado() {
    let (mut cpu, mut bus) = maquina();
    suja(&mut cpu, &mut bus, 0x1234);
    let esperado = retrato(&cpu, &bus);
    let estado = salvo(&cpu, &bus);

    suja(&mut cpu, &mut bus, 0x5678);
    assert_ne!(retrato(&cpu, &bus), esperado, "o rabisco tem de mudar algo");

    snapshot::carrega(&mut cpu, &mut bus, &estado, SERIAL).expect("carregar");
    assert_eq!(retrato(&cpu, &bus), esperado);
}

#[test]
fn snapshot_de_estado_restaurado_e_byte_a_byte_o_mesmo() {
    let (mut cpu, mut bus) = maquina();
    suja(&mut cpu, &mut bus, 0x0F0F);
    let estado = salvo(&cpu, &bus);

    suja(&mut cpu, &mut bus, 0x7070);
    snapshot::carrega(&mut cpu, &mut bus, &estado, SERIAL).expect("carregar");

    assert_eq!(salvo(&cpu, &bus), estado);
}

#[test]
fn maquina_restaurada_continua_executando_igual() {
    let (mut cpu, mut bus) = maquina();
    for _ in 0..2000 {
        cpu.step(&mut bus);
    }
    let estado = salvo(&cpu, &bus);
    for _ in 0..5000 {
        cpu.step(&mut bus);
    }
    let depois = retrato(&cpu, &bus);
    let estado_depois = salvo(&cpu, &bus);

    snapshot::carrega(&mut cpu, &mut bus, &estado, SERIAL).expect("carregar");
    for _ in 0..5000 {
        cpu.step(&mut bus);
    }
    assert_eq!(retrato(&cpu, &bus), depois);
    assert_eq!(
        salvo(&cpu, &bus),
        estado_depois,
        "rodar os mesmos 5000 passos a partir do estado restaurado tem de levar a maquina \
         INTEIRA ao mesmo lugar — inclusive a fila de eventos do scheduler, que nenhum \
         retrato de doze valores enxerga"
    );
}

#[test]
fn cabecalho_tem_magico_versao_e_serial() {
    let (cpu, bus) = maquina();
    let estado = salvo(&cpu, &bus);
    assert_eq!(&estado[0..8], snapshot::MAGICO);
    assert_eq!(
        u32::from_le_bytes([estado[8], estado[9], estado[10], estado[11]]),
        snapshot::VERSAO
    );
    assert_eq!(snapshot::serial_de(&estado).as_deref(), Some(SERIAL));
}

#[test]
fn magico_errado_e_recusado() {
    let (mut cpu, mut bus) = maquina();
    let mut estado = salvo(&cpu, &bus);
    estado[2] = b'Z';
    assert_eq!(
        snapshot::carrega(&mut cpu, &mut bus, &estado, SERIAL),
        Err(SnapshotError::Magico)
    );
    assert_eq!(snapshot::serial_de(&estado), None);
}

#[test]
fn versao_diferente_e_recusada_dizendo_qual() {
    let (mut cpu, mut bus) = maquina();
    let mut estado = salvo(&cpu, &bus);
    estado[8..12].copy_from_slice(&(snapshot::VERSAO + 9).to_le_bytes());
    assert_eq!(
        snapshot::carrega(&mut cpu, &mut bus, &estado, SERIAL),
        Err(SnapshotError::Versao(snapshot::VERSAO + 9))
    );
}

#[test]
fn snapshot_de_outro_jogo_e_recusado() {
    let (mut cpu, mut bus) = maquina();
    let estado = salvo(&cpu, &bus);
    let erro = snapshot::carrega(&mut cpu, &mut bus, &estado, "SCUS-94900");
    assert_eq!(
        erro,
        Err(SnapshotError::Serial {
            esperado: "SCUS-94900".to_string(),
            achado: SERIAL.to_string(),
        })
    );
}

#[test]
fn arquivo_truncado_nao_derruba_o_emulador() {
    let (mut cpu, mut bus) = maquina();
    let estado = salvo(&cpu, &bus);
    let cortado = &estado[..estado.len() / 2];
    assert_eq!(
        snapshot::carrega(&mut cpu, &mut bus, cortado, SERIAL),
        Err(SnapshotError::Corrompido)
    );
    assert_eq!(
        snapshot::carrega(&mut cpu, &mut bus, &[], SERIAL),
        Err(SnapshotError::Corrompido)
    );
}

#[test]
fn estado_recusado_nao_mexe_na_maquina() {
    let (mut cpu, mut bus) = maquina();
    suja(&mut cpu, &mut bus, 0x2222);
    let antes = retrato(&cpu, &bus);
    let mut estado = salvo(&cpu, &bus);
    estado[0] = b'X';
    let _ = snapshot::carrega(&mut cpu, &mut bus, &estado, SERIAL);
    assert_eq!(retrato(&cpu, &bus), antes);
}

#[test]
fn imagem_do_disco_nao_entra_no_snapshot() {
    let (cpu, mut bus) = maquina();
    let vazio = salvo(&cpu, &bus).len();

    let layout = psx_core::cdrom_bin_cue::parse_cue(
        "FILE \"jogo.bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\n",
    );
    bus.inject_disc(layout, vec![0x5Au8; 8 * 1024 * 1024]);
    let com_disco = salvo(&cpu, &bus).len();

    assert!(
        com_disco < vazio + 1024,
        "o snapshot cresceu {} bytes com um disco de 8 MiB: a imagem esta indo junto",
        com_disco - vazio
    );
}

#[test]
fn disco_e_bios_sobrevivem_a_restauracao() {
    let (mut cpu, mut bus) = maquina();
    let layout = psx_core::cdrom_bin_cue::parse_cue(
        "FILE \"jogo.bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\n",
    );
    bus.inject_disc(layout, vec![0x5Au8; 4 * 2352]);
    let estado = salvo(&cpu, &bus);

    snapshot::carrega(&mut cpu, &mut bus, &estado, SERIAL).expect("carregar");

    assert!(
        bus.tem_disco(),
        "a imagem do disco nao pode sumir na restauracao"
    );
    assert_eq!(
        bus.bios_size(),
        0x80000,
        "a BIOS nao pode sumir na restauracao"
    );
}

fn caminho<'a>(candidatos: &'a [&'a str]) -> Option<&'a str> {
    candidatos
        .iter()
        .copied()
        .find(|p| std::path::Path::new(p).exists())
}

/// Prova em maquina de verdade — BIOS e disco reais, nao a sintetica dos outros testes.
/// Se ignora sozinho sem os arquivos (sao gitignored), entao NAO conta para a bateria:
/// quem cobre os mutantes da 0194 sao os testes acima.
#[test]
fn save_state_de_maquina_real_leva_ao_mesmo_lugar() {
    let Some(bios_path) = caminho(&["bios/SCPH1001.BIN", "../../bios/SCPH1001.BIN"]) else {
        return;
    };
    let Some(cue) = caminho(&[
        "../roms/extraido/Crash Bandicoot (USA).cue",
        "../../roms/extraido/Crash Bandicoot (USA).cue",
        "../../../roms/extraido/Crash Bandicoot (USA).cue",
    ]) else {
        return;
    };
    let bios = Bios::from_bytes(std::fs::read(bios_path).expect("ler BIOS")).expect("BIOS valida");
    let mut bus = Bus::new(Ram::new(), bios);
    let mut layout =
        psx_core::cdrom_bin_cue::parse_cue(&std::fs::read_to_string(cue).expect("cue"));
    let pasta = std::path::Path::new(cue).parent().expect("pasta do cue");
    let mut setores = Vec::new();
    let mut bin = Vec::new();
    for arquivo in layout.arquivos_em_ordem() {
        let dados = std::fs::read(pasta.join(&arquivo)).expect("ler bin");
        setores.push((dados.len() / 2352) as u32);
        bin.extend_from_slice(&dados);
    }
    layout.atribui_lbas_absolutos(&setores);
    bus.inject_disc(layout, bin);
    bus.cdrom_mut().insert_disc();

    let mut cpu = Cpu::new();
    for _ in 0..20_000_000 {
        cpu.step(&mut bus);
    }
    let estado = snapshot::salva(&cpu, &bus, "SCUS-94900");
    for _ in 0..2_000_000 {
        cpu.step(&mut bus);
    }
    let esperado = snapshot::salva(&cpu, &bus, "SCUS-94900");

    snapshot::carrega(&mut cpu, &mut bus, &estado, "SCUS-94900").expect("carregar");
    for _ in 0..2_000_000 {
        cpu.step(&mut bus);
    }
    assert_eq!(
        snapshot::salva(&cpu, &bus, "SCUS-94900"),
        esperado,
        "2 M passos a partir do estado restaurado tem de dar a MESMA maquina"
    );
}
