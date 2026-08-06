mod support;

use support::asm;

#[test]
fn hblank_entra_uma_vez_por_scanline_na_proporcao_do_horizontal_display_range() {
    let mut bus = asm::bus_with_bios_empty();
    let cpu_per_sl = bus.gpu().cpu_cycles_per_scanline();

    let mut ciclos_ativos = 0u64;
    let mut ciclos_blanking = 0u64;
    let mut entradas_em_blanking = 0u32;
    let mut estado_anterior = bus.gpu().hblank_active();

    // 3 scanlines completas, ciclo a ciclo, usando SO tick_timers (o caminho real do
    // scheduler) — nunca gpu.set_hblank_active, que e so o setter manual usado pelos
    // testes de Timers::tick isolados.
    let total_ciclos = cpu_per_sl * 3;
    let mut avancado = 0u64;
    while avancado < total_ciclos {
        bus.tick_timers(1);
        avancado += 1;
        let estado = bus.gpu().hblank_active();
        if estado {
            ciclos_blanking += 1;
        } else {
            ciclos_ativos += 1;
        }
        if estado && !estado_anterior {
            entradas_em_blanking += 1;
        }
        estado_anterior = estado;
    }

    assert_eq!(
        entradas_em_blanking, 3,
        "3 scanlines completas tem que produzir exatamente 3 entradas em blanking (uma por \
         linha, achado 10.117) — se o periodo de reagendamento estiver errado (ex.: 1x por \
         frame em vez de 1x por linha), esse numero fica bem menor. Obtido {entradas_em_blanking}"
    );

    let fracao_ativa = ciclos_ativos as f64 / (ciclos_ativos + ciclos_blanking) as f64;
    assert!(
        (0.60..0.85).contains(&fracao_ativa),
        "spec § GP1(06h) - Horizontal Display range (03-gpu.md L826): com X1=0x200,X2=0xC00 \
         (janela ativa de ~75% dos 3413 video cycles/linha em NTSC), o display ativo tem que \
         dominar a linha. Fracao ativa medida: {fracao_ativa:.3} — se X1/X2 estiverem trocados \
         na conversao pra ciclos de CPU, essa fracao cai pra ~25% em vez de ~75%"
    );
}
