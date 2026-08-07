use psx_core::gte::Gte;

// § Cada comando GTE (docs/reference/07-gte.md, cabecalhos "#### COP2 <imm25>h - N Cycles -
// <MNEMONICO>") documenta o proprio custo no titulo da secao. Os valores de `func` abaixo
// sao o imm25 COMPLETO de cada cabecalho, nao so os 6 bits baixos que `Gte::execute_command`
// usa pra despachar — isso prova, de quebra, que `command_cycles` decodifica o opcode do
// mesmo jeito que o dispatch real (`func & 0x3F`).

#[test]
fn rtps_custa_15() {
    assert_eq!(Gte::command_cycles(0x0180001), 15, "07-gte.md L481: RTPS");
}

#[test]
fn rtpt_custa_23() {
    assert_eq!(Gte::command_cycles(0x0280030), 23, "07-gte.md L482: RTPT");
}

#[test]
fn nclip_custa_8() {
    assert_eq!(Gte::command_cycles(0x1400006), 8, "07-gte.md L513: NCLIP");
}

#[test]
fn avsz3_custa_5() {
    assert_eq!(Gte::command_cycles(0x158002D), 5, "07-gte.md L523: AVSZ3");
}

#[test]
fn avsz4_custa_6() {
    assert_eq!(Gte::command_cycles(0x168002E), 6, "07-gte.md L524: AVSZ4");
}

#[test]
fn mvmva_custa_8() {
    assert_eq!(Gte::command_cycles(0x0400012), 8, "07-gte.md L541: MVMVA");
}

#[test]
fn sqr_custa_5() {
    assert_eq!(Gte::command_cycles(0x0A00428), 5, "07-gte.md L566: SQR");
}

#[test]
fn op_custa_6() {
    assert_eq!(Gte::command_cycles(0x170000C), 6, "07-gte.md L574: OP");
}

#[test]
fn ncs_custa_14() {
    assert_eq!(Gte::command_cycles(0x0C8041E), 14, "07-gte.md L592: NCS");
}

#[test]
fn nct_custa_30() {
    assert_eq!(Gte::command_cycles(0x0D80420), 30, "07-gte.md L593: NCT");
}

#[test]
fn nccs_custa_17() {
    assert_eq!(Gte::command_cycles(0x108041B), 17, "07-gte.md L594: NCCS");
}

#[test]
fn ncct_custa_39() {
    assert_eq!(Gte::command_cycles(0x118043F), 39, "07-gte.md L595: NCCT");
}

#[test]
fn ncds_custa_19() {
    assert_eq!(Gte::command_cycles(0x0E80413), 19, "07-gte.md L596: NCDS");
}

#[test]
fn ncdt_custa_44() {
    assert_eq!(Gte::command_cycles(0x0F80416), 44, "07-gte.md L597: NCDT");
}

#[test]
fn cc_custa_11() {
    assert_eq!(Gte::command_cycles(0x138041C), 11, "07-gte.md L610: CC");
}

#[test]
fn cdp_custa_13() {
    // A trap deste comando: CC e CDP caem no mesmo braco Rust (`color_color`), so
    // diferenciados pelo FarColor -- copiar o custo do braco de dispatch em vez do
    // cabecalho da spec dava os dois iguais.
    assert_eq!(Gte::command_cycles(0x1280414), 13, "07-gte.md L611: CDP");
}

#[test]
fn dcpl_custa_8() {
    assert_eq!(Gte::command_cycles(0x0680029), 8, "07-gte.md L622: DCPL");
}

#[test]
fn dpcs_custa_8() {
    assert_eq!(Gte::command_cycles(0x0780010), 8, "07-gte.md L623: DPCS");
}

#[test]
fn dpct_custa_17() {
    assert_eq!(Gte::command_cycles(0x0F8002A), 17, "07-gte.md L624: DPCT");
}

#[test]
fn intpl_custa_8() {
    assert_eq!(Gte::command_cycles(0x0980011), 8, "07-gte.md L625: INTPL");
}

#[test]
fn gpf_custa_5() {
    assert_eq!(Gte::command_cycles(0x190003D), 5, "07-gte.md L641: GPF");
}

#[test]
fn gpl_custa_5() {
    assert_eq!(Gte::command_cycles(0x1A0003E), 5, "07-gte.md L642: GPL");
}

#[test]
fn sf_e_lm_nao_mudam_o_custo() {
    // sf (bit 19) e lm (bit 10) sao flags de comportamento numerico, nao fazem parte do
    // opcode (bits 0-5) -- variar esses bits nao pode mudar o custo tabelado.
    assert_eq!(
        Gte::command_cycles(0x0A00428),
        Gte::command_cycles(0x0A80428),
        "SQR com sf=0 e sf=1 custam o mesmo (5 ciclos)"
    );
}

#[test]
fn comando_nao_implementado_custa_zero() {
    // A spec nao documenta custo pra nenhum opcode fora dos 22 que Gte::execute_command
    // despacha (o dispatch real cai no `_ => {}`, sem efeito) -- nao ha numero pra citar,
    // entao o custo fica em zero em vez de inventado.
    assert_eq!(
        Gte::command_cycles(0x0000_0000),
        0,
        "opcode 0 nao e nenhum dos 22 comandos documentados"
    );
}
