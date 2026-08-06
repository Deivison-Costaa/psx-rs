mod support;

use support::asm;

#[test]
fn hblank_alterna_sozinho_ao_avancar_o_scheduler_por_uma_linha() {
    let mut bus = asm::bus_with_bios_empty();
    let mut viu_ativo = false;
    let mut viu_blanking = false;

    // Avanca ~2.5 scanlines (566187/263 ~= 2153 ciclos de CPU cada) em passos finos,
    // usando SO tick_timers (o caminho real do scheduler) — nunca gpu.set_hblank_active,
    // que e so o setter manual usado pelos testes de Timers::tick isolados.
    for _ in 0..900 {
        bus.tick_timers(6);
        if bus.gpu().hblank_active() {
            viu_blanking = true;
        } else {
            viu_ativo = true;
        }
    }

    assert!(
        viu_ativo,
        "tem que existir periodo de display ativo (hblank=false) dentro da linha"
    );
    assert!(
        viu_blanking,
        "spec § Horizontal Timings / GP1(06h) (03-gpu.md L1469, L826): o sinal de hblank e \
         gerado uma vez por scanline, fora da janela [X1,X2) do Horizontal Display Range — \
         hoje isso NUNCA acontece porque nada agenda um evento de hblank no scheduler \
         (achado 10.117), so o VBLANK e agendado"
    );
}
