use std::collections::HashMap;

use psx_core::app::library::{
    self, Extensao, Regiao, dados_do_setor, identifica, procura_no_diretorio, raiz_do_pvd,
    regiao_da_licenca, rotulo_do_volume, serial_do_boot,
};

const SETOR: usize = 2048;

fn linha_de_licenca(segunda: &str) -> Vec<u8> {
    let mut s = vec![0u8; SETOR];
    s[0x00..0x20].copy_from_slice(b"          Licensed  by          ");
    let bytes = segunda.as_bytes();
    s[0x20..0x20 + bytes.len()].copy_from_slice(bytes);
    s
}

fn escreve_u32_duplo(dst: &mut [u8], valor: u32) {
    dst[0..4].copy_from_slice(&valor.to_le_bytes());
    dst[4..8].copy_from_slice(&valor.to_be_bytes());
}

fn registro(nome: &[u8], lba: u32, tamanho: u32, flags: u8, sistema: usize) -> Vec<u8> {
    let pad = usize::from(nome.len() % 2 == 0);
    let len_dr = 33 + nome.len() + pad + sistema;
    let mut r = vec![0u8; len_dr];
    r[0x00] = len_dr as u8;
    escreve_u32_duplo(&mut r[0x02..0x0A], lba);
    escreve_u32_duplo(&mut r[0x0A..0x12], tamanho);
    r[0x19] = flags;
    r[0x1C..0x1E].copy_from_slice(&1u16.to_le_bytes());
    r[0x1E..0x20].copy_from_slice(&1u16.to_be_bytes());
    r[0x20] = nome.len() as u8;
    r[0x21..0x21 + nome.len()].copy_from_slice(nome);
    r
}

fn pvd(rotulo: &str, raiz_lba: u32, raiz_tam: u32) -> Vec<u8> {
    let mut s = vec![0u8; SETOR];
    s[0x00] = 0x01;
    s[0x01..0x06].copy_from_slice(b"CD001");
    s[0x06] = 0x01;
    s[0x08..0x08 + 11].copy_from_slice(b"PLAYSTATION");
    let r = rotulo.as_bytes();
    s[0x28..0x28 + r.len()].copy_from_slice(r);
    for b in s[0x28 + r.len()..0x48].iter_mut() {
        *b = b' ';
    }
    let raiz = registro(&[0x00], raiz_lba, raiz_tam, 0x02, 0);
    s[0x9C..0x9C + raiz.len()].copy_from_slice(&raiz);
    s
}

fn diretorio_raiz() -> Vec<u8> {
    let mut d = Vec::new();
    d.extend_from_slice(&registro(&[0x00], 22, 2048, 0x02, 0));
    d.extend_from_slice(&registro(&[0x01], 22, 2048, 0x02, 0));
    d.extend_from_slice(&registro(b"SYSTEM.CNF;1", 23, 68, 0x00, 14));
    d.extend_from_slice(&registro(b"SLUS_007.27;1", 24, 4096, 0x00, 14));
    d.resize(SETOR, 0);
    d
}

fn system_cnf() -> Vec<u8> {
    let texto = "BOOT = cdrom:\\SLUS_007.27;1\r\nTCB = 4\r\nEVENT = 10\r\nSTACK = 801FFF00\r\n";
    let mut s = texto.as_bytes().to_vec();
    s.resize(SETOR, 0);
    s
}

fn disco_falso() -> HashMap<u32, Vec<u8>> {
    let mut d = HashMap::new();
    d.insert(
        4u32,
        linha_de_licenca("Sony Computer Entertainment Amer  ica "),
    );
    d.insert(16u32, pvd("CRASH", 22, 2048));
    d.insert(22u32, diretorio_raiz());
    d.insert(23u32, system_cnf());
    d
}

#[test]
fn licenca_de_cada_regiao_sai_do_setor_4() {
    assert_eq!(
        regiao_da_licenca(&linha_de_licenca("Sony Computer Entertainment Amer  ica ")),
        Regiao::America
    );
    assert_eq!(
        regiao_da_licenca(&linha_de_licenca("Sony Computer Entertainment Euro pe   ")),
        Regiao::Europa
    );
    assert_eq!(
        regiao_da_licenca(&linha_de_licenca("Sony Computer Entertainment Inc.\n")),
        Regiao::Japao
    );
}

#[test]
fn setor_sem_licenca_nao_inventa_regiao() {
    assert_eq!(regiao_da_licenca(&vec![0u8; SETOR]), Regiao::Desconhecida);
    assert_eq!(regiao_da_licenca(&[]), Regiao::Desconhecida);
}

#[test]
fn pvd_entrega_raiz_e_rotulo() {
    let s = pvd("CRASH", 22, 2048);
    assert_eq!(
        raiz_do_pvd(&s),
        Some(Extensao {
            lba: 22,
            tamanho: 2048
        })
    );
    assert_eq!(rotulo_do_volume(&s).as_deref(), Some("CRASH"));
}

#[test]
fn pvd_sem_assinatura_cd001_e_recusado() {
    let mut s = pvd("CRASH", 22, 2048);
    s[0x03] = b'X';
    assert_eq!(raiz_do_pvd(&s), None);
    assert_eq!(rotulo_do_volume(&s), None);
}

#[test]
fn pvd_de_tipo_diferente_de_1_e_recusado() {
    let mut s = pvd("CRASH", 22, 2048);
    s[0x00] = 0xFF;
    assert_eq!(raiz_do_pvd(&s), None);
}

#[test]
fn diretorio_acha_o_arquivo_pelo_nome_com_versao() {
    let d = diretorio_raiz();
    assert_eq!(
        procura_no_diretorio(&d, "SYSTEM.CNF;1"),
        Some(Extensao {
            lba: 23,
            tamanho: 68
        })
    );
    assert_eq!(
        procura_no_diretorio(&d, "SLUS_007.27;1"),
        Some(Extensao {
            lba: 24,
            tamanho: 4096
        })
    );
    assert_eq!(procura_no_diretorio(&d, "AUSENTE.DAT;1"), None);
}

#[test]
fn diretorio_avanca_por_len_dr_e_nao_por_33_mais_o_nome() {
    let d = diretorio_raiz();
    let primeiro_com_system_use = 0x21 + 12 + 1 + 14;
    assert!(primeiro_com_system_use > 33 + 12);
    assert_eq!(
        procura_no_diretorio(&d, "SLUS_007.27;1").map(|e| e.lba),
        Some(24),
        "o registro seguinte ao SYSTEM.CNF so e alcancado somando LEN_DR, que inclui os \
         14 bytes de System Use do CD-XA"
    );
}

#[test]
fn registro_de_tamanho_zero_encerra_o_diretorio() {
    let mut d = vec![0u8; SETOR];
    let r = registro(b"DEPOIS.DAT;1", 99, 8, 0x00, 0);
    d[64..64 + r.len()].copy_from_slice(&r);
    assert_eq!(procura_no_diretorio(&d, "DEPOIS.DAT;1"), None);
}

#[test]
fn boot_vira_serial_normalizado() {
    assert_eq!(
        serial_do_boot("BOOT = cdrom:\\SLUS_007.27;1\n").as_deref(),
        Some("SLUS-00727")
    );
    assert_eq!(
        serial_do_boot("BOOT=cdrom:SLES_012.34;1\n").as_deref(),
        Some("SLES-01234")
    );
    assert_eq!(
        serial_do_boot("TCB = 4\nBOOT = cdrom:\\\\SCPS_100.01;1\nEVENT = 10\n").as_deref(),
        Some("SCPS-10001")
    );
}

#[test]
fn system_cnf_sem_boot_nao_produz_serial() {
    assert_eq!(serial_do_boot("TCB = 4\nEVENT = 10\n"), None);
    assert_eq!(serial_do_boot("BOOT = \n"), None);
}

#[test]
fn identifica_o_disco_inteiro_a_partir_dos_setores() {
    let disco = disco_falso();
    let id = identifica(|lba| disco.get(&lba).cloned());
    assert_eq!(id.serial.as_deref(), Some("SLUS-00727"));
    assert_eq!(id.regiao, Regiao::America);
    assert_eq!(id.rotulo.as_deref(), Some("CRASH"));
    assert_eq!(id.boot.as_deref(), Some("cdrom:\\SLUS_007.27;1"));
}

#[test]
fn disco_sem_pvd_nao_derruba_a_identificacao() {
    let mut disco = disco_falso();
    disco.remove(&16);
    let id = identifica(|lba| disco.get(&lba).cloned());
    assert_eq!(id.serial, None);
    assert_eq!(id.rotulo, None);
    assert_eq!(id.regiao, Regiao::America);
}

#[test]
fn identifica_le_apenas_os_setores_necessarios() {
    let disco = disco_falso();
    let mut lidos = Vec::new();
    let _ = identifica(|lba| {
        lidos.push(lba);
        disco.get(&lba).cloned()
    });
    assert!(lidos.len() <= 6, "leu {} setores: {lidos:?}", lidos.len());
    assert!(lidos.contains(&library::SETOR_LICENCA));
    assert!(lidos.contains(&library::SETOR_PVD));
}

#[test]
fn diretorio_maior_que_um_setor_e_percorrido_inteiro() {
    let mut primeiro = vec![0u8; SETOR];
    let ponta = registro(b"OUTRO.DAT;1", 90, 4, 0x00, 0);
    primeiro[0..ponta.len()].copy_from_slice(&ponta);
    let mut segundo = vec![0u8; SETOR];
    let alvo = registro(b"SYSTEM.CNF;1", 40, 68, 0x00, 14);
    segundo[0..alvo.len()].copy_from_slice(&alvo);

    let mut disco = HashMap::new();
    disco.insert(
        4u32,
        linha_de_licenca("Sony Computer Entertainment Euro pe   "),
    );
    disco.insert(16u32, pvd("RAYMAN", 22, 4096));
    disco.insert(22u32, primeiro);
    disco.insert(23u32, segundo);
    disco.insert(40u32, system_cnf());

    let id = identifica(|lba| disco.get(&lba).cloned());
    assert_eq!(
        id.serial.as_deref(),
        Some("SLUS-00727"),
        "o SYSTEM.CNF esta no SEGUNDO setor da raiz; parar no primeiro perde o arquivo"
    );
    assert_eq!(id.regiao, Regiao::Europa);
}

#[test]
fn setor_bruto_de_2352_tem_os_dados_no_offset_certo_por_modo() {
    let mut mode1 = vec![0u8; 2352];
    mode1[0x0F] = 0x01;
    mode1[0x10] = 0xAA;
    assert_eq!(dados_do_setor(&mode1).map(|d| d[0]), Some(0xAA));
    assert_eq!(dados_do_setor(&mode1).map(<[u8]>::len), Some(2048));

    let mut mode2 = vec![0u8; 2352];
    mode2[0x0F] = 0x02;
    mode2[0x18] = 0xBB;
    assert_eq!(dados_do_setor(&mode2).map(|d| d[0]), Some(0xBB));

    let plano = vec![0x77u8; 2048];
    assert_eq!(dados_do_setor(&plano).map(|d| d[0]), Some(0x77));
    assert_eq!(dados_do_setor(&[0u8; 100]), None);
}
