use psx_core::bus::{Bios, Bus, BusRead, BusWrite, Ram};
use psx_core::cpu::Cpu;
use psx_core::snapshot::{self, DiscoGravado, Metadados, SnapshotError};

const SERIAL: &str = "SLUS-01251";
const DISCO_1: &str = "/jogos/FF9 (Disc 1)/FF9 (Disc 1).cue";
const DISCO_2: &str = "/jogos/FF9 (Disc 2)/FF9 (Disc 2).cue";
const RAM_ALVO: u32 = 0x0010_0000;

fn maquina() -> (Cpu, Bus) {
    let bios = Bios::from_bytes(vec![0u8; 0x80000]).expect("BIOS sintetica de 512 KiB");
    (Cpu::new(), Bus::new(Ram::new(), bios))
}

fn com_disco(caminho: &str, serial_do_disco: &str) -> Metadados {
    Metadados {
        serial: SERIAL.to_string(),
        disco: Some(DiscoGravado {
            serial: serial_do_disco.to_string(),
            caminho: caminho.to_string(),
        }),
    }
}

#[test]
fn disco_da_bandeja_fica_gravado_no_estado() {
    let (mut cpu, mut bus) = maquina();
    let metadados = com_disco(DISCO_2, "SLUS-01295");
    let estado = snapshot::salva_com(&cpu, &bus, &metadados).expect("codificar");
    assert_eq!(snapshot::metadados_de(&estado), Ok(metadados));
    assert_eq!(snapshot::serial_de(&estado).as_deref(), Some(SERIAL));
    snapshot::carrega(&mut cpu, &mut bus, &estado, SERIAL).expect("carregar");
}

#[test]
fn so_pede_troca_quando_o_disco_gravado_nao_e_o_da_bandeja() {
    let metadados = com_disco(DISCO_2, "SLUS-01295");
    assert_eq!(
        metadados
            .disco_diferente_de(DISCO_1)
            .map(|d| d.caminho.as_str()),
        Some(DISCO_2),
        "estado salvo no disco 2, bandeja com o disco 1: o disco 2 tem de voltar"
    );
    assert_eq!(metadados.disco_diferente_de(DISCO_2), None);
    assert_eq!(
        Metadados::sem_disco(SERIAL).disco_diferente_de(DISCO_1),
        None,
        "estado sem disco gravado mantem o disco atual"
    );
}

#[test]
fn metadados_de_arquivo_estranho_sao_recusados() {
    assert_eq!(snapshot::metadados_de(&[]), Err(SnapshotError::Corrompido));
    assert_eq!(
        snapshot::metadados_de(b"NAOESTADO000"),
        Err(SnapshotError::Magico)
    );
}

/// Versao 3: o mesmo corpo sem a tag do `Option` do disco. Save state antigo continua valendo.
#[test]
fn estado_da_versao_3_carrega_sem_disco() {
    let (mut cpu, mut bus) = maquina();
    bus.write32::<BusWrite>(RAM_ALVO, 0xCAFE_F00D);
    let mut antigo = snapshot::salva(&cpu, &bus, SERIAL).expect("codificar");
    let tag_do_disco = 12 + 8 + SERIAL.len();
    assert_eq!(antigo[tag_do_disco], 0, "tag de `None`");
    antigo.remove(tag_do_disco);
    antigo[8..12].copy_from_slice(&snapshot::VERSAO_SEM_DISCO.to_le_bytes());

    bus.write32::<BusWrite>(RAM_ALVO, 0);
    assert_eq!(
        snapshot::metadados_de(&antigo),
        Ok(Metadados::sem_disco(SERIAL))
    );
    snapshot::carrega(&mut cpu, &mut bus, &antigo, SERIAL).expect("carregar v3");
    assert_eq!(bus.read32::<BusRead>(RAM_ALVO), 0xCAFE_F00D);
}
