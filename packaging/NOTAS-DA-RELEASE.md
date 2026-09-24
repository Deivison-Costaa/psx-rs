Primeira versão pública do **psx-rs**, um emulador de PlayStation 1 escrito em Rust, com app desktop próprio.

## Downloads

| Sistema | Arquivo |
|---|---|
| Linux (qualquer distro) | `psx-rs-*-x86_64.AppImage`: `chmod +x` e rodar |
| Linux (binários soltos) | `psx-rs-*-linux-x86_64.tar.gz` |
| Windows 10/11 | `psx-rs-*-windows-x86_64.zip`: extrair e abrir `psx-desktop.exe` |

> **A BIOS do PlayStation não vem junto** (tem direitos autorais). Use a do seu console (testado com a SCPH-1001, NTSC-U) e imagens `.cue/.bin` dos seus discos. No primeiro uso, abra **Ajustes** e aponte a BIOS e a pasta de jogos. Mais detalhes no `LEIA-ME.txt` de cada pacote.

## Jogos verificados

Uma suíte automática roda cada jogo com um roteiro de botões e compara a imagem, quadro a quadro, com o DuckStation. Nesta versão os 12 jogos testados passam:

Crash Bandicoot · CTR · Final Fantasy IX · Gran Turismo 2 (Arcade e Simulation) · Metal Gear Solid · Rayman · Resident Evil 2 · Resident Evil 3 · Tekken 3 · Tomb Raider II · Tomb Raider III

Save e load no memory card foram confirmados em Tomb Raider II, Tomb Raider III e Rayman. A troca de disco foi testada no Final Fantasy IX.

## App desktop

- Biblioteca com busca, "Continuar" e discos do mesmo jogo agrupados.
- Imagem ajustada à janela (4:3, centralizada), com opção de escala inteira. Tela cheia com **F11** ou **Alt+Enter**.
- Menu de pausa no **Esc**: salvar e carregar estado, memory card, trocar disco, controles, ajustes.
- Estados salvos (**F4**) com miniatura e data, em 10 slots.
- Gerenciador de memory card: apagar, exportar, importar (`.mcd`, `.mcr`, `.gme`, `.mem`) e formatar.
- Teclado e controle remapeáveis: basta apertar a tecla ou o botão. DualShock com analógicos e vibração. Os menus também funcionam pelo controle.
- Áudio com controle de fluxo e reamostragem cúbica.
- Configuração em `~/.config/psx-rs` e `~/.local/share/psx-rs` no Linux, e em `%APPDATA%\psx-rs` no Windows.

## Desempenho

O emulador roda a ~5,7x o tempo real num núcleo de CPU (60 s de jogo em ~10,5 s), usando ~8 MB de RAM no núcleo de emulação.

## Limitações conhecidas

- O tempo de seek do CD é mais rápido que o do hardware real, então algumas cargas terminam 0,5 a 1 s antes.
- Renderização só por software, na resolução nativa.
- Controle físico e o pacote de Windows foram testados apenas por testes automatizados e pelo build do CI.
