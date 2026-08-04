use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const LARGURA: usize = 1024;
const ALTURA: usize = 512;

fn dir_unico(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("relogio")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("psx-vram-png-{tag}-{nanos}"));
    fs::create_dir_all(&dir).expect("criar diretorio temporario");
    dir
}

fn vram_crua_com_cantos() -> Vec<u8> {
    let mut crua = vec![0u8; LARGURA * ALTURA * 2];
    let escreve = |crua: &mut Vec<u8>, x: usize, y: usize, px: u16| {
        let i = (y * LARGURA + x) * 2;
        crua[i] = (px & 0xFF) as u8;
        crua[i + 1] = (px >> 8) as u8;
    };
    escreve(&mut crua, 0, 0, 0x001F);
    escreve(&mut crua, 1, 0, 0x03E0);
    escreve(&mut crua, 2, 0, 0x7C00);
    escreve(&mut crua, 3, 0, 0x7FFF);
    escreve(&mut crua, LARGURA - 1, ALTURA - 1, 0x8000 | 0x001F);
    crua
}

fn converte(crua: &[u8], dir: &Path) -> (std::process::Output, PathBuf) {
    let entrada = dir.join("tela.vram");
    let saida = dir.join("tela.png");
    fs::write(&entrada, crua).expect("escrever raw");
    let out = Command::new(env!("CARGO_BIN_EXE_psx-cli"))
        .arg("--vram-to-png")
        .arg(&entrada)
        .arg(&saida)
        .output()
        .expect("executar psx-cli");
    (out, saida)
}

fn decodifica(png_path: &Path) -> (u32, u32, Vec<u8>) {
    let arquivo = std::io::BufReader::new(fs::File::open(png_path).expect("abrir png"));
    let decoder = png::Decoder::new(arquivo);
    let mut reader = decoder.read_info().expect("ler cabecalho png");
    let mut dados = vec![0u8; reader.output_buffer_size().expect("tamanho do buffer")];
    let info = reader.next_frame(&mut dados).expect("decodificar png");
    dados.truncate(info.buffer_size());
    (info.width, info.height, dados)
}

fn pixel(dados: &[u8], x: usize, y: usize) -> [u8; 3] {
    let i = (y * LARGURA + x) * 3;
    [dados[i], dados[i + 1], dados[i + 2]]
}

#[test]
fn converte_bgr555_para_png_rgb_1024x512() {
    let dir = dir_unico("ok");
    let (out, saida) = converte(&vram_crua_com_cantos(), &dir);
    assert!(
        out.status.success(),
        "conversao deve sair 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );

    let (w, h, dados) = decodifica(&saida);
    assert_eq!((w, h), (1024, 512), "gabaritos do ps1-tests sao 1024x512");
    assert_eq!(pixel(&dados, 0, 0), [248, 0, 0], "0x001F e vermelho puro");
    assert_eq!(pixel(&dados, 1, 0), [0, 248, 0], "0x03E0 e verde puro");
    assert_eq!(pixel(&dados, 2, 0), [0, 0, 248], "0x7C00 e azul puro");
    assert_eq!(pixel(&dados, 3, 0), [248, 248, 248], "0x7FFF e branco");
    assert_eq!(pixel(&dados, 4, 0), [0, 0, 0], "0x0000 e preto");
    assert_eq!(
        pixel(&dados, LARGURA - 1, ALTURA - 1),
        [248, 0, 0],
        "bit 15 (mask) nao muda a cor exibida"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn raw_de_tamanho_errado_reprova_com_mensagem() {
    let dir = dir_unico("curto");
    let (out, saida) = converte(&vec![0u8; 1000], &dir);
    assert!(
        !out.status.success(),
        "raw de 1000 bytes nao e uma VRAM de 1 MiB e tem de reprovar"
    );
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("1048576"),
        "a mensagem deve dizer o tamanho esperado (1048576 bytes)"
    );
    assert!(!saida.exists(), "nao pode deixar PNG pela metade no disco");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn entrada_ausente_reprova_sem_criar_saida() {
    let dir = dir_unico("ausente");
    let saida = dir.join("tela.png");
    let out = Command::new(env!("CARGO_BIN_EXE_psx-cli"))
        .arg("--vram-to-png")
        .arg(dir.join("nao-existe.vram"))
        .arg(&saida)
        .output()
        .expect("executar psx-cli");
    assert!(!out.status.success());
    assert!(!saida.exists());
    let _ = fs::remove_dir_all(&dir);
}
