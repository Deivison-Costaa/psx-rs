use psx_core::bus::{Bios, Bus, Ram};
use psx_core::spu;

// Bug do proprio scheduler (nao spec de hardware, R2 do CLAUDE.md): eventos periodicos
// (VBLANK/HBLANK/SPU_TICK) reagendavam a partir de `total_cycles` no instante em que
// disparam, nao do prazo que venceu. Invisivel enquanto o maior "tick" possivel era 27
// ciclos (custo de ROM); um tick grande (GTE ate 44, DMA de um setor = 12288) faz o
// reagendamento passar direto por cima de varios periodos que deveriam ter dado catch-up.

fn bus_ntsc() -> Bus {
    let bios = Bios::from_bytes(vec![0; 0x80000]).expect("BIOS sintetica valida");
    Bus::new(Ram::new(), bios)
}

// `Spu::tick()` empilha uma amostra em `output` a cada chamada (spu.rs:488-489) -- e' o
// unico jeito, sem expor o scheduler interno (pub(crate)), de contar de fora quantas vezes
// o evento periodico SPU_TICK disparou de verdade.
#[test]
fn tick_gigante_dispara_todos_os_periodos_de_spu_cobertos_um_a_um() {
    let mut bus = bus_ntsc();
    // primeiro SPU_TICK agendado em CPU_CYCLES_PER_SAMPLE (bus.rs:196-199); um unico tick
    // cobrindo 10 periodos inteiros + sobra deveria disparar o evento 10 vezes, nao 1.
    let dez_periodos_e_sobra = (spu::CPU_CYCLES_PER_SAMPLE as u32) * 10 + 100;
    bus.tick_timers(dez_periodos_e_sobra);
    assert_eq!(
        bus.spu().output_len(),
        10,
        "10 periodos de SPU_TICK cabem inteiros num tick de 10*periodo+100; reagendar a \
         partir de total_cycles (bug) faz o novo prazo saltar por cima do resto e so 1 \
         dispara"
    );
}

#[test]
fn dois_ticks_gigantes_seguidos_nao_perdem_nem_duplicam_periodos() {
    let mut bus = bus_ntsc();
    let periodo = spu::CPU_CYCLES_PER_SAMPLE as u32;
    // 4 periodos e meio, depois mais 4 periodos e meio: fecha 9 periodos completos ao
    // final, nao 8 (perda) nem 10 (duplicacao) -- prova que o reagendamento nao dá deriva
    // entre chamadas de tick_timers.
    bus.tick_timers(periodo * 4 + periodo / 2);
    bus.tick_timers(periodo * 4 + periodo / 2);
    assert_eq!(
        bus.spu().output_len(),
        9,
        "9 periodos completos de SPU_TICK cobertos por dois ticks de 4,5 periodos cada"
    );
}

#[test]
fn tick_pequeno_continua_disparando_no_maximo_um_periodo_por_vez() {
    // controle: nenhum tick isolado menor que um periodo deveria disparar mais de um
    // evento -- prova que o catch-up so age quando o tick realmente cobre >1 periodo,
    // igual ao comportamento ja coberto por irq0_periodo_ntsc.rs pra ticks de 1 ciclo.
    let mut bus = bus_ntsc();
    let periodo = spu::CPU_CYCLES_PER_SAMPLE as u32;
    bus.tick_timers(periodo - 1);
    assert_eq!(
        bus.spu().output_len(),
        0,
        "ainda nao venceu o primeiro periodo"
    );
    bus.tick_timers(1);
    assert_eq!(
        bus.spu().output_len(),
        1,
        "venceu exatamente um periodo, dispara um"
    );
}
