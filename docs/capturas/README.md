# Capturas de execução

Dumps de VRAM convertidos para PNG. Servem como **prova datada** do que o emulador desenhava
num commit específico — sem o comando que a gerou, uma captura não prova nada.

Todas foram geradas em **2026-08-01**, no commit `760eb75` (merge da iteração 0146), com:

```sh
cargo build --release -p psx-cli
psx-cli --bios bios/SCPH1001.BIN --disc <CUE> --max-steps <N> --dump-vram <saida>.raw
```

O `.raw` é a VRAM inteira: 1024×512 pixels, 16 bits por pixel, RGB555 little-endian
(`crates/psx-cli/src/main.rs`, `write_vram_dump`). A conversão para PNG é direta, expandindo
cada canal de 5 para 8 bits (`<< 3`), sem correção de gama nem qualquer ajuste.

| arquivo | disco | passos | o que mostra |
|---|---|---|---|
| `crash-vram-150M.png` | Crash Bandicoot (USA) | 150 M | tela de licença da BIOS renderizada: "PlayStation™ / Licensed by Sony Computer Entertainment America / SCEA™" |
| `crash-vram-400M.png` | Crash Bandicoot (USA) | 400 M | framebuffer **vazio**, mas texturas do jogo carregadas no canto inferior direito da VRAM |
| `rayman-vram-150M.png` | Rayman (USA), só a track de dados | 150 M | tela de licença da BIOS |
| `rayman-vram-400M.png` | Rayman (USA), só a track de dados | 400 M | **tela da Ubi Soft desenhada pelo próprio jogo**, com o degradê do arco-íris |

## Como ler o `crash-vram-400M.png`

O padrão colorido que parece ruído **é** dado real: textura indexada por CLUT (4 ou 8 bits por
pixel) exibida como cor direta de 16 bits. Num dump cru ela aparece assim mesmo. Dá para
distinguir estrutura repetida do tamanho de sprites. A leitura é: o Crash carrega seus assets
na VRAM e congela **antes** de desenhar — coerente com o diagnóstico do ROADMAP 4.5.

## Sobre os discos

Nenhuma imagem de disco e nenhuma BIOS é versionada (`.gitignore`, e é regra do projeto). Estas
capturas são saída do nosso emulador, guardadas em resolução de VRAM apenas para documentar
progresso de emulação.

O Rayman foi rodado com um `.cue` reduzido, contendo **só a track 01 (dados)**. O `.cue` original
tem 51 tracks (uma de dados e 50 de áudio CD-DA) e o nosso `parse_cue` guarda um único
`bin_path`: cada linha `FILE` sobrescreve a anterior, então sobraria a track 51, que é áudio.
Limitação registrada no ROADMAP.
