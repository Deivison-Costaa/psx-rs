use psx_core::bus::BusRead;
use psx_core::cpu::Cpu;

mod support;
use support::asm::{bus_with_bios_empty, nop};

// oraculo/gte/test-all (JaCzekanski/ps1-tests), test 51 "GTE 0x01 (lm=1, ...)":
// FLAG.22 e o IR3 armazenado divergiam do gabarito porque nosso RTPS usava a MESMA
// faixa (a do lm real) para decidir a flag e para recortar o valor guardado. A spec
// separa as duas coisas (§ COP2 0180001h - RTPS (L507-511) de docs/reference/07-gte.md).

fn ctc2(rt: u32, rd: u32) -> u32 {
    (0x12 << 26) | (0x06 << 21) | (rt << 16) | (rd << 11)
}

fn mfc2(rt: u32, rd: u32) -> u32 {
    (0x12 << 26) | (rt << 16) | (rd << 11)
}

fn cfc2(rt: u32, rd: u32) -> u32 {
    (0x12 << 26) | (0x02 << 21) | (rt << 16) | (rd << 11)
}

fn cop2_cmd(real_cmd: u32, sf: bool, lm: bool) -> u32 {
    (0x12 << 26) | (1 << 25) | (real_cmd & 0x3F) | ((sf as u32) << 19) | ((lm as u32) << 10)
}

fn escreve_e_executa(cpu: &mut Cpu, bus: &mut psx_core::bus::Bus, addr: u32, instr: u32) {
    bus.write32::<BusRead>(addr, instr);
    cpu.pc = addr;
    cpu.step(bus);
}

fn ctc2_r8(cpu: &mut Cpu, bus: &mut psx_core::bus::Bus, a: u32, rd: u32, val: u32) {
    cpu.regs[8] = val;
    escreve_e_executa(cpu, bus, a, ctc2(8, rd));
}

fn le_mfc2(cpu: &mut Cpu, bus: &mut psx_core::bus::Bus, a: u32, dst: usize, rd: u32) {
    escreve_e_executa(cpu, bus, a, mfc2(dst as u32, rd));
    escreve_e_executa(cpu, bus, a, nop());
}

fn le_cfc2(cpu: &mut Cpu, bus: &mut psx_core::bus::Bus, a: u32, dst: usize, rd: u32) {
    escreve_e_executa(cpu, bus, a, cfc2(dst as u32, rd));
    escreve_e_executa(cpu, bus, a, nop());
}

// RT31..33 e TRX/TRY ficam em zero (default); com sf=1 e TRZ=trz, MAC3 = TRZ*1000h
// deslocado 12 bits = TRZ exato, o que isola o teste da parte de rotacao/perspectiva.
fn roda_rtps_so_com_trz(cpu: &mut Cpu, bus: &mut psx_core::bus::Bus, trz: i32, lm: bool) {
    let a = 0x0000u32;
    ctc2_r8(cpu, bus, a, 7, trz as u32); // cop2r39 (cnt7) = TRZ
    escreve_e_executa(cpu, bus, a, cop2_cmd(0x01, true, lm));
    escreve_e_executa(cpu, bus, a, nop());
}

#[test]
fn ir3_dentro_da_faixa_lm0_mas_fora_da_lm1_nao_liga_flag22() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.set_sr(1 << 30);

    // -100 cabe em -8000h..+7FFFh (lm=0) mas nao em 0..+7FFFh (lm=1).
    roda_rtps_so_com_trz(&mut cpu, &mut bus, -100, true);

    le_mfc2(&mut cpu, &mut bus, 0x0000, 10, 11);
    le_cfc2(&mut cpu, &mut bus, 0x0000, 11, 31);

    assert_eq!(
        cpu.regs[10] as i32, 0,
        "IR3 armazenado usa a faixa do lm real (lm=1): -100 satura em 0"
    );
    assert_eq!(
        cpu.regs[11] & (1 << 22),
        0,
        "FLAG.22 usa SEMPRE a faixa de lm=0: -100 cabe nela, entao a flag fica limpa"
    );
}

#[test]
fn ir3_fora_da_faixa_lm0_liga_flag22_mesmo_com_lm1() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.set_sr(1 << 30);

    // -40000 nao cabe nem em -8000h..+7FFFh (lm=0) nem em 0..+7FFFh (lm=1).
    roda_rtps_so_com_trz(&mut cpu, &mut bus, -40000, true);

    le_mfc2(&mut cpu, &mut bus, 0x0000, 10, 11);
    le_cfc2(&mut cpu, &mut bus, 0x0000, 11, 31);

    assert_eq!(
        cpu.regs[10] as i32, 0,
        "IR3 armazenado usa a faixa do lm real (lm=1): -40000 satura em 0"
    );
    assert_ne!(
        cpu.regs[11] & (1 << 22),
        0,
        "FLAG.22 usa a faixa de lm=0: -40000 excede -8000h, entao a flag liga"
    );
}

// § COP2 0180001h - RTPS (L509-511): "Quando sf=0, FLAG.22 so liga se 'MAC3 SAR 12'
// exceder a faixa" — ou seja, a decisao usa SEMPRE o deslocamento de 12 bits, mesmo
// que o MAC3 armazenado (sf=0, sem deslocamento) tenha uma escala bem maior. Este
// teste e o unico que separa de fato "flag_check" (sempre SAR 12) de "mac3" bruto:
// com sf=1 (como nos dois testes acima) os dois sempre coincidem.
#[test]
fn flag22_usa_sar12_mesmo_com_sf_zero_apesar_do_mac3_bruto_estourar() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.set_sr(1 << 30);
    let a = 0x0000u32;

    // sf=0: MAC3 armazenado = TRZ*1000h = 100*1000h = 409600 (estoura -8000h..+7FFFh
    // por si so), mas "MAC3 SAR 12" = TRZ = 100, que cabe folgado na faixa de lm=0.
    ctc2_r8(&mut cpu, &mut bus, a, 7, 100u32); // cop2r39 (cnt7) = TRZ
    escreve_e_executa(&mut cpu, &mut bus, a, cop2_cmd(0x01, false, true));
    escreve_e_executa(&mut cpu, &mut bus, a, nop());

    le_cfc2(&mut cpu, &mut bus, 0x0000, 11, 31);

    assert_eq!(
        cpu.regs[11] & (1 << 22),
        0,
        "FLAG.22 usa 'MAC3 SAR 12'=100 (dentro da faixa), nao o MAC3 bruto=409600"
    );
}

#[test]
fn ir3_fora_da_faixa_lm0_com_lm0_satura_em_8000h() {
    let mut bus = bus_with_bios_empty();
    let mut cpu = Cpu::new();
    cpu.set_sr(1 << 30);

    // Mesmo MAC3 do teste anterior, mas agora com lm=0: a faixa armazenada tambem
    // e -8000h..+7FFFh, entao o valor guardado muda (satura em -8000h, nao em 0).
    roda_rtps_so_com_trz(&mut cpu, &mut bus, -40000, false);

    le_mfc2(&mut cpu, &mut bus, 0x0000, 10, 11);

    assert_eq!(
        cpu.regs[10] as i32, -0x8000,
        "IR3 armazenado com lm=0: -40000 satura em -8000h"
    );
}
