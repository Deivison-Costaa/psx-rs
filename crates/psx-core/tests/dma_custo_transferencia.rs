use psx_core::dma::Dma;

// § DMA Transfer Rates (docs/reference/04-dma.md L217-227): custo por canal em "clks per
// 100h(=256) words" -- a razao PRECISA, nao o "N clks/word" arredondado do cabecalho.
// Valores lidos direto do bloco de texto da spec (grep -n), nao do indice (offset, nao
// linha real) nem do "N clks/word" isolado (que e' so o arredondamento pra 1 casa).
//
// MDEC.IN/OUT, GPU, OTC: 110h=272 por 100h=256 palavras (~17/16, "1 clk/word")
// CDROM (default da BIOS -- jogos que reconfiguram pra 40 sao achado aberto, nao modelado
// aqui: a spec nao da a formula do registrador de memory control que decide): 1800h=6144
// por 256 (24/1 exato, "24 clks/word")
// SPU: 420h=1056 por 256 (33/8, "4 clks/word" arredondado)
// PIO: 1400h=5120 por 256 (20/1 exato, "20 clks/word")

const MDEC_IN: usize = 0;
const MDEC_OUT: usize = 1;
const GPU: usize = 2;
const CDROM: usize = 3;
const SPU: usize = 4;
const PIO: usize = 5;
const OTC: usize = 6;

#[test]
fn mdec_in_custa_272_por_256_palavras() {
    assert_eq!(Dma::word_cost_per_256(MDEC_IN), 272);
}

#[test]
fn mdec_out_custa_272_por_256_palavras() {
    assert_eq!(Dma::word_cost_per_256(MDEC_OUT), 272);
}

#[test]
fn gpu_custa_272_por_256_palavras() {
    assert_eq!(Dma::word_cost_per_256(GPU), 272);
}

#[test]
fn cdrom_custa_6144_por_256_palavras_no_padrao_da_bios() {
    assert_eq!(Dma::word_cost_per_256(CDROM), 6144);
}

#[test]
fn spu_custa_1056_por_256_palavras() {
    assert_eq!(Dma::word_cost_per_256(SPU), 1056);
}

#[test]
fn pio_custa_5120_por_256_palavras() {
    assert_eq!(Dma::word_cost_per_256(PIO), 5120);
}

#[test]
fn otc_custa_272_por_256_palavras() {
    assert_eq!(Dma::word_cost_per_256(OTC), 272);
}

#[test]
fn canal_desconhecido_custa_zero() {
    assert_eq!(
        Dma::word_cost_per_256(7),
        0,
        "so os 7 canais 0-6 existem no hardware"
    );
}

#[test]
fn transfer_cost_de_256_palavras_bate_com_o_custo_por_256() {
    assert_eq!(Dma::transfer_cost(MDEC_IN, 256), 272);
    assert_eq!(Dma::transfer_cost(CDROM, 256), 6144);
    assert_eq!(Dma::transfer_cost(SPU, 256), 1056);
    assert_eq!(Dma::transfer_cost(PIO, 256), 5120);
}

#[test]
fn transfer_cost_de_16_palavras_do_mdec_bate_com_a_nota_da_dram_hyper_page_mode() {
    // § DRAM Hyper Page mode (04-dma.md L238-243): "effectively around 17 clks per 16
    // words" -- confere contra a fracao exata (272/256 = 17/16), nao contra o texto solto.
    assert_eq!(Dma::transfer_cost(MDEC_IN, 16), 17);
}

#[test]
fn transfer_cost_de_8_palavras_do_spu_bate_com_33_8() {
    assert_eq!(Dma::transfer_cost(SPU, 8), 33);
}

#[test]
fn transfer_cost_de_1_palavra_do_cdrom_e_pio_bate_com_a_razao_inteira() {
    assert_eq!(Dma::transfer_cost(CDROM, 1), 24);
    assert_eq!(Dma::transfer_cost(PIO, 1), 20);
}

#[test]
fn transfer_cost_de_zero_palavras_e_zero() {
    assert_eq!(Dma::transfer_cost(GPU, 0), 0);
}
