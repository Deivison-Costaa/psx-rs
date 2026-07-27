fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 || (args.len() == 2 && args[1] == "--version") {
        println!("psx-cli {}", env!("CARGO_PKG_VERSION"));
    }
}
