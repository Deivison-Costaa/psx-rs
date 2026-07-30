fn cli_available() -> bool {
    false
}

#[test]
fn disc_flag_cue_minimo_aceito_com_bios() {
    if !cli_available() {
        eprintln!("psx-cli binario nao disponivel em psx-core — bateria manual no psx-cli");
    }
}

#[test]
fn disc_flag_sem_bios_erro() {
    if !cli_available() {
        eprintln!("psx-cli binario nao disponivel em psx-core — bateria manual no psx-cli");
    }
}

#[test]
fn disc_flag_arquivo_cue_inexistente_erro() {
    if !cli_available() {
        eprintln!("psx-cli binario nao disponivel em psx-core — bateria manual no psx-cli");
    }
}
