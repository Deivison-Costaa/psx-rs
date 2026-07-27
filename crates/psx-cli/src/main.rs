use std::io::Read;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 || (args.len() == 2 && args[1] == "--version") {
        println!("psx-cli {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    if args.len() == 3 && args[1] == "--bios" {
        let path = &args[2];
        let mut file = match std::fs::File::open(path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!(
                    "Erro: nao foi possivel ler o arquivo BIOS '{}': {}",
                    path, e
                );
                std::process::exit(1);
            }
        };
        let mut data = Vec::new();
        if let Err(e) = file.read_to_end(&mut data) {
            eprintln!("Erro: falha ao ler '{}': {}", path, e);
            std::process::exit(1);
        }
        match psx_core::bus::Bios::from_bytes(data) {
            Ok(bios) => {
                use sha2::Digest;
                let hash = sha2::Sha256::digest(bios.raw());
                println!("BIOS: {} bytes, SHA-256: {:x}", bios.size(), hash);
            }
            Err(e) => {
                eprintln!("Erro: BIOS invalida: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }
    eprintln!("Uso: psx-cli [--version | --bios <caminho>]");
    std::process::exit(1);
}
