mod disasm;

use psx_core::app::library;
use psx_core::bus::{Bios, Bus, BusRead, Ram};
use psx_core::cdrom_bin_cue::{DiscLayout, parse_cue};
use psx_core::cpu::Cpu;
use psx_core::pad_script::{PadScript, RELEASED};
use std::collections::HashSet;
use std::io::Read;
use std::io::Write;

const RUNNER_MAX_STEPS: usize = 50_000_000;
const KERNEL_ENTRYPOINT_PC: u32 = 0x8003_0000;
const BIOS_BOOT_TO_KERNEL_MAX_STEPS: usize = 20_000_000;

fn boot_bios_to_kernel(cpu: &mut Cpu, bus: &mut Bus) -> Result<usize, String> {
    let mut steps = 0usize;
    while cpu.pc != KERNEL_ENTRYPOINT_PC {
        if steps >= BIOS_BOOT_TO_KERNEL_MAX_STEPS {
            return Err(format!(
                "BIOS nao alcancou o ponto de kernel montado (pc=0x{:08X}) em {} passos; \
                 pc atual=0x{:08X}",
                KERNEL_ENTRYPOINT_PC, BIOS_BOOT_TO_KERNEL_MAX_STEPS, cpu.pc
            ));
        }
        cpu.step(bus);
        steps += 1;
    }
    Ok(steps)
}

fn resolve_btable_entry(bus: &Bus, index: u32) -> Option<u32> {
    let w_b0 = bus.read32::<BusRead>(0x0000_00B0);
    let w_b4 = bus.read32::<BusRead>(0x0000_00B4);
    let imm_lui = w_b0 & 0xFFFF;
    let imm_addiu = w_b4 & 0xFFFF;
    let b_dispatch = (imm_lui << 16).wrapping_add(imm_addiu);
    if b_dispatch == 0 {
        return None;
    }

    let w_lui_disp = bus.read32::<BusRead>(b_dispatch);
    let w_addiu_disp = bus.read32::<BusRead>(b_dispatch + 4);
    let imm_lui_disp = w_lui_disp & 0xFFFF;
    let imm_addiu_disp = w_addiu_disp & 0xFFFF;
    let b_table = (imm_lui_disp << 16).wrapping_add(imm_addiu_disp);
    if b_table == 0 {
        return None;
    }

    let addr = bus.read32::<BusRead>(b_table + index * 4);
    if addr == 0 { None } else { Some(addr) }
}

/// Sondas que so existem para medir uma rodada; nenhuma delas muda o que o
/// emulador executa.
struct Sondas<'a> {
    trace_pcs: &'a HashSet<u32>,
    sample_pcs: Option<(usize, usize, usize)>,
    watch_mem: &'a [u32],
    vram_timeline: Option<(usize, &'a str)>,
    audio_dump: Option<&'a str>,
}

fn run(cpu: &mut Cpu, bus: &mut Bus, max_steps: usize, pad: &PadScript, sondas: &Sondas) -> usize {
    let Sondas {
        trace_pcs,
        sample_pcs,
        watch_mem,
        vram_timeline,
        audio_dump,
    } = *sondas;
    let mut steps = 0;
    // Comparar antes/depois de cada passo atribui a escrita ao PC exato que a fez. Foi assim
    // que a 0182 descobriu quem apagava o IRQ0 do Rayman em minutos, depois de horas de
    // desmontagem a mao.
    let mut watch_valores: Vec<u32> = watch_mem
        .iter()
        .map(|&a| bus.read32::<BusRead>(a))
        .collect();
    let mut pad_state = RELEASED;
    if !pad.is_empty() {
        bus.sio_mut().set_buttons(pad_state);
    }
    let mut deliver_event_pc: Option<u32> = None;
    // Quadros PCM crus (i16 L/R little-endian, 44100 Hz) para provar que o jogo soa.
    let mut audio_pcm: Vec<u8> = Vec::new();
    while steps < max_steps {
        let pc_antes = cpu.pc;
        cpu.step(bus);
        steps += 1;

        for (idx, &addr) in watch_mem.iter().enumerate() {
            let agora = bus.read32::<BusRead>(addr);
            if agora != watch_valores[idx] {
                eprintln!(
                    "watch {:08X}: passo={} pc=0x{:08X} de=0x{:08X} para=0x{:08X}",
                    addr, steps, pc_antes, watch_valores[idx], agora
                );
                watch_valores[idx] = agora;
            }
        }

        if !pad.is_empty() {
            let desejado = pad.buttons_at(steps as u64);
            if desejado != pad_state {
                pad_state = desejado;
                bus.sio_mut().set_buttons(pad_state);
            }
        }

        if let Some((cada, prefixo)) = vram_timeline {
            if steps % cada == 0 {
                write_vram_dump(&format!("{prefixo}-{}.vram", steps / cada), bus);
            }
        }

        if audio_dump.is_some() && steps % 4096 == 0 {
            for (e, d) in bus.drain_audio() {
                audio_pcm.extend_from_slice(&e.to_le_bytes());
                audio_pcm.extend_from_slice(&d.to_le_bytes());
            }
        }

        if let Some((start, end, stride)) = sample_pcs {
            if steps >= start && steps <= end && (steps - start) % stride == 0 {
                eprintln!("sample pc=0x{:08X} step={}", cpu.pc, steps);
            }
        }

        if deliver_event_pc.is_none() && steps % 100_000 == 0 {
            if let Some(addr) = resolve_btable_entry(bus, 7) {
                deliver_event_pc = Some(addr);
                eprintln!(
                    "# DeliverEvent B-table[07h]=0x{:08X} (step={})",
                    addr, steps
                );
            }
        }

        if let Some(depc) = deliver_event_pc {
            if cpu.pc == depc {
                eprintln!(
                    "# DeliverEvent pc=0x{:08X} step={} class(a0)=0x{:08X} spec(a1)=0x{:08X}",
                    cpu.pc, steps, cpu.regs[4], cpu.regs[5]
                );
            }
        }

        if !trace_pcs.is_empty() && trace_pcs.contains(&cpu.pc) {
            let instr = bus.read32::<BusRead>(cpu.pc);
            let _rs = ((instr >> 21) & 0x1F) as usize;
            let _rt = ((instr >> 16) & 0x1F) as usize;
            eprintln!(
                "trace pc=0x{:08X} step={} instr=0x{:08X} \
                 regs: a0($4)=0x{:08X} a1($5)=0x{:08X} t1($9)=0x{:08X} s1($17)=0x{:08X} \
                 v0($2)=0x{:08X} t4($12)=0x{:08X} t5($13)=0x{:08X} ra($31)=0x{:08X}",
                cpu.pc,
                steps,
                instr,
                cpu.regs[4],
                cpu.regs[5],
                cpu.regs[9],
                cpu.regs[17],
                cpu.regs[2],
                cpu.regs[12],
                cpu.regs[13],
                cpu.regs[31],
            );
            eprintln!(
                "     mem[t1*4]=0x{:08X} mem[s1*4]=0x{:08X}",
                bus.read32::<BusRead>(cpu.regs[9].wrapping_mul(4)),
                bus.read32::<BusRead>(cpu.regs[17].wrapping_mul(4)),
            );
            let _ = std::io::stderr().flush();
        }
    }
    if let Some(caminho) = audio_dump {
        let nao_silencio = audio_pcm.chunks_exact(2).filter(|q| q != &[0, 0]).count();
        eprintln!(
            "# AUDIO quadros={} amostras-nao-zero={}",
            audio_pcm.len() / 4,
            nao_silencio
        );
        if let Err(e) = std::fs::write(caminho, &audio_pcm) {
            eprintln!("Erro: nao consegui gravar o audio '{caminho}': {e}");
        }
    }
    steps
}

fn write_vram_dump(path: &str, bus: &Bus) {
    let mut data = Vec::with_capacity(1024 * 512 * 2);
    for y in 0..512u16 {
        for x in 0..1024u16 {
            data.extend_from_slice(&bus.gpu().vram_pixel(x, y).to_le_bytes());
        }
    }
    if let Err(e) = std::fs::write(path, &data) {
        eprintln!(
            "Erro: nao foi possivel escrever dump de VRAM '{}': {}",
            path, e
        );
        std::process::exit(1);
    }
    eprintln!("dump-vram: {} ({} bytes)", path, data.len());
    escreve_tela(&format!("{path}.tela.png"), bus);
}

/// A VRAM crua nao diz em que profundidade a area de display esta sendo lida; um dump
/// visto sempre como 15bpp faz FMV (24bpp) parecer ruido. Este PNG e o que o console
/// mostraria, com a profundidade que a GPU esta usando.
fn escreve_tela(saida: &str, bus: &Bus) {
    let fb = bus.gpu().framebuffer();
    if fb.width == 0 || fb.height == 0 {
        return;
    }
    let mut rgb = Vec::with_capacity(fb.data.len() / 4 * 3);
    for px in fb.data.chunks_exact(4) {
        rgb.extend_from_slice(&px[..3]);
    }
    let arquivo = match std::fs::File::create(saida) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Erro: nao criei '{saida}': {e}");
            return;
        }
    };
    let mut cod = png::Encoder::new(
        std::io::BufWriter::new(arquivo),
        fb.width as u32,
        fb.height as u32,
    );
    cod.set_color(png::ColorType::Rgb);
    cod.set_depth(png::BitDepth::Eight);
    let escrita = cod
        .write_header()
        .and_then(|mut w| w.write_image_data(&rgb).map(|_| ()));
    match escrita {
        Ok(()) => eprintln!(
            "dump-tela: {} ({}x{}, {}bpp)",
            saida,
            fb.width,
            fb.height,
            if bus.gpu().stat() & (1 << 21) != 0 {
                24
            } else {
                15
            }
        ),
        Err(e) => eprintln!("Erro: png '{saida}': {e}"),
    }
}

fn load_disc(disc_path: &str) -> (DiscLayout, Vec<u8>) {
    let cue_text = match std::fs::read_to_string(disc_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Erro: nao foi possivel ler CUE '{}': {}", disc_path, e);
            std::process::exit(1);
        }
    };

    let layout = parse_cue(&cue_text);
    let cue_dir = std::path::Path::new(disc_path)
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));

    let mut layout = layout;
    let mut setores: Vec<u32> = Vec::new();
    let mut bin_data: Vec<u8> = Vec::new();
    for arquivo in layout.arquivos_em_ordem() {
        let bin_path = cue_dir.join(&arquivo);
        match std::fs::read(&bin_path) {
            Ok(d) => {
                setores.push((d.len() / 2352) as u32);
                bin_data.extend_from_slice(&d);
            }
            Err(e) => {
                eprintln!(
                    "Erro: nao foi possivel ler BIN '{}': {}",
                    bin_path.display(),
                    e
                );
                std::process::exit(1);
            }
        }
    }

    layout.atribui_lbas_absolutos(&setores);

    (layout, bin_data)
}

/// Carrega a imagem `.mcd` (criando uma zerada de 128 KiB se nao existir) e liga o
/// cartao no slot 1.
fn monta_memory_card(bus: &mut Bus, caminho: Option<&str>) {
    let Some(caminho) = caminho else {
        return;
    };
    let bytes = std::fs::read(caminho).unwrap_or_else(|_| vec![0u8; psx_core::memcard::CARD_BYTES]);
    match bus.sio_mut().load_memory_card(&bytes) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Erro: memory card '{caminho}' invalido: {e:?}");
            std::process::exit(1);
        }
    }
}

/// Regrava o `.mcd` se o jogo escreveu nele durante a execucao.
fn salva_memory_card(bus: &Bus, caminho: Option<&str>) {
    let Some(caminho) = caminho else {
        return;
    };
    if !bus.sio().memory_card_dirty() {
        return;
    }
    if let Err(e) = std::fs::write(caminho, bus.sio().memory_card_image()) {
        eprintln!("Erro: nao consegui gravar o memory card '{caminho}': {e}");
    }
}

/// Identifica um disco sem bootar nada: le so os setores do ISO 9660 (licenca, PVD,
/// diretorio raiz, SYSTEM.CNF) por seek. E a mesma rotina que a biblioteca do app usa.
fn imprime_identidade(disc_path: &str) {
    let caminho = std::path::Path::new(disc_path);
    let layout = match std::fs::read_to_string(caminho) {
        Ok(t) => parse_cue(&t),
        Err(e) => {
            eprintln!("Erro: nao foi possivel ler CUE '{disc_path}': {e}");
            std::process::exit(1);
        }
    };
    let pasta = caminho
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    let Some(primeiro) = layout.arquivos_em_ordem().first().map(|a| pasta.join(a)) else {
        eprintln!("Erro: CUE sem FILE: '{disc_path}'");
        std::process::exit(1);
    };
    let passo: u64 = match layout.tracks.first().map(|t| &t.track_type) {
        Some(psx_core::cdrom_bin_cue::TrackType::Mode1_2048) => 2048,
        _ => 2352,
    };
    let mut arquivo = match std::fs::File::open(&primeiro) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Erro: nao foi possivel abrir '{}': {e}", primeiro.display());
            std::process::exit(1);
        }
    };
    let id = library::identifica(|lba| {
        use std::io::{Read, Seek, SeekFrom};
        let mut bruto = vec![0u8; passo as usize];
        arquivo.seek(SeekFrom::Start(u64::from(lba) * passo)).ok()?;
        arquivo.read_exact(&mut bruto).ok()?;
        library::dados_do_setor(&bruto).map(<[u8]>::to_vec)
    });
    println!("arquivo: {disc_path}");
    println!("serial: {}", id.serial.as_deref().unwrap_or("?"));
    println!("regiao: {}", id.regiao.nome());
    println!("rotulo: {}", id.rotulo.as_deref().unwrap_or("?"));
    println!("boot: {}", id.boot.as_deref().unwrap_or("?"));
    println!("trilhas: {}", layout.tracks.len());
}

fn vram_para_png(entrada: &str, saida: &str) -> Result<(), String> {
    const ESPERADO: usize = 1024 * 512 * 2;
    let crua = std::fs::read(entrada).map_err(|e| format!("nao li '{entrada}': {e}"))?;
    if crua.len() != ESPERADO {
        return Err(format!(
            "'{entrada}' tem {} bytes; uma VRAM crua tem {ESPERADO} bytes (1024x512 halfwords)",
            crua.len()
        ));
    }
    let mut rgb = Vec::with_capacity(1024 * 512 * 3);
    for par in crua.chunks_exact(2) {
        let px = u16::from_le_bytes([par[0], par[1]]);
        rgb.push(((px & 0x1F) << 3) as u8);
        rgb.push((((px >> 5) & 0x1F) << 3) as u8);
        rgb.push((((px >> 10) & 0x1F) << 3) as u8);
    }
    let arquivo = std::fs::File::create(saida).map_err(|e| format!("nao criei '{saida}': {e}"))?;
    let mut codificador = png::Encoder::new(std::io::BufWriter::new(arquivo), 1024, 512);
    codificador.set_color(png::ColorType::Rgb);
    codificador.set_depth(png::BitDepth::Eight);
    let mut escritor = codificador
        .write_header()
        .map_err(|e| format!("cabecalho png: {e}"))?;
    escritor
        .write_image_data(&rgb)
        .map_err(|e| format!("dados png: {e}"))?;
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 || (args.len() == 2 && args[1] == "--version") {
        println!("psx-cli {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    if args.len() == 3 && args[1] == "--disc-info" {
        imprime_identidade(&args[2]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--vram-to-png") {
        if args.len() != 4 {
            eprintln!("Uso: psx-cli --vram-to-png <entrada.vram> <saida.png>");
            std::process::exit(1);
        }
        if let Err(e) = vram_para_png(&args[2], &args[3]) {
            eprintln!("Erro: {e}");
            std::process::exit(1);
        }
        return;
    }

    let mut bios_arg: Option<String> = None;
    let mut exe_arg: Option<String> = None;
    let mut disc_arg: Option<String> = None;
    let mut max_steps: Option<usize> = None;
    let mut trace_pcs: HashSet<u32> = HashSet::new();
    let mut watch_mem: Vec<u32> = Vec::new();
    let mut dump_mem: Vec<(u32, usize)> = Vec::new();
    let mut disasm_mem: Vec<(u32, usize)> = Vec::new();
    let mut dump_vram: Option<String> = None;
    let mut vram_timeline: Option<(usize, String)> = None;
    let mut audio_dump: Option<String> = None;
    let mut sample_pcs: Option<(usize, usize, usize)> = None;
    let mut pad_connected = false;
    let mut memcard_arg: Option<String> = None;
    let mut press_specs: Vec<String> = Vec::new();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--bios" if i + 1 < args.len() => {
                bios_arg = Some(args[i + 1].clone());
                i += 2;
            }
            "--exe" if i + 1 < args.len() => {
                exe_arg = Some(args[i + 1].clone());
                i += 2;
            }
            "--pad" => {
                pad_connected = true;
                i += 1;
            }
            "--dump-audio" if i + 1 < args.len() => {
                audio_dump = Some(args[i + 1].clone());
                i += 2;
            }
            "--memcard" if i + 1 < args.len() => {
                memcard_arg = Some(args[i + 1].clone());
                i += 2;
            }
            "--press" if i + 1 < args.len() => {
                press_specs.push(args[i + 1].clone());
                pad_connected = true;
                i += 2;
            }
            "--disc" if i + 1 < args.len() => {
                disc_arg = Some(args[i + 1].clone());
                i += 2;
            }
            "--max-steps" if i + 1 < args.len() => match args[i + 1].parse::<usize>() {
                Ok(n) => {
                    max_steps = Some(n);
                    i += 2;
                }
                Err(e) => {
                    eprintln!(
                        "Erro: '--max-steps' espera um numero, '{}': {}",
                        args[i + 1],
                        e
                    );
                    std::process::exit(1);
                }
            },
            "--watch-mem" if i + 1 < args.len() => {
                for piece in args[i + 1].split(',') {
                    let piece = piece.trim();
                    match u32::from_str_radix(piece.trim_start_matches("0x"), 16) {
                        Ok(addr) => watch_mem.push(addr & !3),
                        Err(e) => {
                            eprintln!(
                                "Erro: '--watch-mem' espera enderecos hex, '{}': {}",
                                piece, e
                            );
                            std::process::exit(1);
                        }
                    }
                }
                i += 2;
            }
            "--trace-pcs" if i + 1 < args.len() => {
                for piece in args[i + 1].split(',') {
                    let piece = piece.trim();
                    match u32::from_str_radix(piece.trim_start_matches("0x"), 16) {
                        Ok(addr) => {
                            trace_pcs.insert(addr);
                        }
                        Err(e) => {
                            eprintln!(
                                "Erro: '--trace-pcs' espera enderecos hex, '{}': {}",
                                piece, e
                            );
                            std::process::exit(1);
                        }
                    }
                }
                i += 2;
            }
            "--bios" | "--exe" | "--disc" => {
                eprintln!("Erro: '{}' requer um caminho", args[i]);
                std::process::exit(1);
            }
            "--dump-mem" if i + 2 < args.len() => {
                let addr = match u32::from_str_radix(args[i + 1].trim_start_matches("0x"), 16) {
                    Ok(a) => a,
                    Err(e) => {
                        eprintln!(
                            "Erro: '--dump-mem' espera endereco hex, '{}': {}",
                            args[i + 1],
                            e
                        );
                        std::process::exit(1);
                    }
                };
                let len = match usize::from_str_radix(args[i + 2].trim_start_matches("0x"), 16) {
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!(
                            "Erro: '--dump-mem' espera um comprimento hex, '{}': {}",
                            args[i + 2],
                            e
                        );
                        std::process::exit(1);
                    }
                };
                dump_mem.push((addr, len));
                i += 3;
            }
            "--disasm" if i + 2 < args.len() => {
                let addr = match u32::from_str_radix(args[i + 1].trim_start_matches("0x"), 16) {
                    Ok(a) => a,
                    Err(e) => {
                        eprintln!(
                            "Erro: '--disasm' espera endereco hex, '{}': {}",
                            args[i + 1],
                            e
                        );
                        std::process::exit(1);
                    }
                };
                let n_instr = match args[i + 2].parse::<usize>() {
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!(
                            "Erro: '--disasm' espera numero de instrucoes, '{}': {}",
                            args[i + 2],
                            e
                        );
                        std::process::exit(1);
                    }
                };
                disasm_mem.push((addr, n_instr));
                i += 3;
            }
            "--dump-vram-every" if i + 2 < args.len() => {
                match args[i + 1].parse::<usize>() {
                    Ok(n) if n > 0 => vram_timeline = Some((n, args[i + 2].clone())),
                    _ => {
                        eprintln!("Erro: --dump-vram-every espera N PREFIXO com N > 0");
                        std::process::exit(1);
                    }
                }
                i += 3;
            }
            "--dump-vram" if i + 1 < args.len() => {
                dump_vram = Some(args[i + 1].clone());
                i += 2;
            }
            "--sample-pcs" if i + 1 < args.len() => {
                let parts: Vec<_> = args[i + 1].split(':').collect();
                let parsed = if parts.len() == 3 {
                    match (
                        parts[0].parse::<usize>(),
                        parts[1].parse::<usize>(),
                        parts[2].parse::<usize>(),
                    ) {
                        (Ok(s), Ok(e), Ok(st)) if st > 0 && s <= e => Some((s, e, st)),
                        _ => None,
                    }
                } else {
                    None
                };
                match parsed {
                    Some(v) => {
                        sample_pcs = Some(v);
                        i += 2;
                    }
                    None => {
                        eprintln!(
                            "Erro: '--sample-pcs' espera inicio:fim:passo decimais, '{}'",
                            args[i + 1]
                        );
                        std::process::exit(1);
                    }
                }
            }
            "--press" => {
                eprintln!("Erro: '--press' requer BOTAO@PASSO[:DURACAO]");
                std::process::exit(1);
            }
            "--max-steps" | "--trace-pcs" | "--dump-vram" | "--sample-pcs" | "--watch-mem"
            | "--dump-vram-every" => {
                eprintln!("Erro: '{}' requer um valor", args[i]);
                std::process::exit(1);
            }
            arg => {
                eprintln!("Erro: argumento desconhecido: '{}'", arg);
                std::process::exit(1);
            }
        }
    }
    let max_steps = max_steps.unwrap_or(RUNNER_MAX_STEPS);
    let pad_script = match PadScript::parse(&press_specs) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Erro: --press {}", e);
            std::process::exit(1);
        }
    };

    if disc_arg.is_some() && bios_arg.is_none() {
        eprintln!("Erro: --disc requer --bios <caminho_da_BIOS>");
        std::process::exit(1);
    }

    if exe_arg.is_some() && bios_arg.is_none() {
        eprintln!(
            "Erro: --exe requer --bios <caminho_da_BIOS>. Use: psx-cli --bios <BIOS> --exe <PS-EXE>"
        );
        std::process::exit(1);
    }

    match (bios_arg.take(), exe_arg.take(), disc_arg.take()) {
        (Some(bios_path), Some(exe_path), disc_path) => {
            let bios_data = match std::fs::read(&bios_path) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Erro: nao foi possivel ler BIOS '{}': {}", bios_path, e);
                    std::process::exit(1);
                }
            };
            let bios = match Bios::from_bytes(bios_data) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("Erro: BIOS invalida: {}", e);
                    std::process::exit(1);
                }
            };

            let exe_data = match std::fs::read(&exe_path) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Erro: nao foi possivel ler EXE '{}': {}", exe_path, e);
                    std::process::exit(1);
                }
            };

            let ram = Ram::new();
            let mut bus = Bus::new(ram, bios);
            let mut cpu = Cpu::new();

            if let Some(disc_path) = disc_path {
                let (layout, bin_data) = load_disc(&disc_path);
                bus.inject_disc(layout, bin_data);
            }

            if let Err(e) = boot_bios_to_kernel(&mut cpu, &mut bus) {
                eprintln!("Erro: {}", e);
                std::process::exit(1);
            }
            if let Err(e) = psx_core::psexe::load_psexe(&exe_data, &mut bus, &mut cpu) {
                eprintln!("Erro: falha ao carregar PS-EXE '{}': {}", exe_path, e);
                std::process::exit(1);
            }

            if pad_connected {
                bus.sio_mut().connect_digital_pad(true);
            }
            monta_memory_card(&mut bus, memcard_arg.as_deref());
            let steps = run(
                &mut cpu,
                &mut bus,
                max_steps,
                &pad_script,
                &Sondas {
                    trace_pcs: &trace_pcs,
                    sample_pcs,
                    watch_mem: &watch_mem,
                    vram_timeline: vram_timeline.as_ref().map(|(n, p)| (*n, p.as_str())),
                    audio_dump: audio_dump.as_deref(),
                },
            );

            salva_memory_card(&bus, memcard_arg.as_deref());
            let tty = bus.take_tty();
            if !tty.is_empty() {
                let _ = std::io::stdout().write_all(&tty);
                std::io::stdout().flush().ok();
            }

            eprintln!("Runner: {} passos, TTY: {} bytes", steps, tty.len());

            for &(addr, len) in &dump_mem {
                eprintln!("dump {:08X}:", addr);
                for off in (0..len).step_by(4) {
                    let word = bus.read32::<BusRead>(addr.wrapping_add(off as u32));
                    eprintln!("  {:08X}: {:08X}", addr.wrapping_add(off as u32), word);
                }
            }

            for &(addr, n_instr) in &disasm_mem {
                eprintln!("disasm {:08X}:", addr);
                for i in 0..n_instr {
                    let a = addr.wrapping_add((i * 4) as u32);
                    let instr = bus.read32::<BusRead>(a);
                    eprintln!("  {:08X}: {:08X}  {}", a, instr, disasm::desmonta(instr));
                }
            }

            if let Some(path) = &dump_vram {
                write_vram_dump(path, &bus);
            }

            return;
        }
        (Some(bios_path), None, disc_path) => {
            let bios = {
                let mut file = match std::fs::File::open(&bios_path) {
                    Ok(f) => f,
                    Err(e) => {
                        eprintln!(
                            "Erro: nao foi possivel ler o arquivo BIOS '{}': {}",
                            bios_path, e
                        );
                        std::process::exit(1);
                    }
                };
                let mut data = Vec::new();
                if let Err(e) = file.read_to_end(&mut data) {
                    eprintln!("Erro: falha ao ler '{}': {}", bios_path, e);
                    std::process::exit(1);
                }
                match Bios::from_bytes(data) {
                    Ok(b) => b,
                    Err(e) => {
                        eprintln!("Erro: BIOS invalida: {}", e);
                        std::process::exit(1);
                    }
                }
            };

            let ram = Ram::new();
            let mut bus = Bus::new(ram, bios);
            let mut cpu = Cpu::new();

            if let Some(disc_path) = disc_path {
                let (layout, bin_data) = load_disc(&disc_path);
                bus.inject_disc(layout, bin_data);
                bus.cdrom_mut().insert_disc();
            }

            if pad_connected {
                bus.sio_mut().connect_digital_pad(true);
            }
            monta_memory_card(&mut bus, memcard_arg.as_deref());
            let steps = run(
                &mut cpu,
                &mut bus,
                max_steps,
                &pad_script,
                &Sondas {
                    trace_pcs: &trace_pcs,
                    sample_pcs,
                    watch_mem: &watch_mem,
                    vram_timeline: vram_timeline.as_ref().map(|(n, p)| (*n, p.as_str())),
                    audio_dump: audio_dump.as_deref(),
                },
            );

            salva_memory_card(&bus, memcard_arg.as_deref());
            let tty = bus.take_tty();
            if !tty.is_empty() {
                let _ = std::io::stdout().write_all(&tty);
                std::io::stdout().flush().ok();
            }

            eprintln!("Runner: {} passos, TTY: {} bytes", steps, tty.len());

            for &(addr, len) in &dump_mem {
                eprintln!("dump {:08X}:", addr);
                for off in (0..len).step_by(4) {
                    let word = bus.read32::<BusRead>(addr.wrapping_add(off as u32));
                    eprintln!("  {:08X}: {:08X}", addr.wrapping_add(off as u32), word);
                }
            }

            for &(addr, n_instr) in &disasm_mem {
                eprintln!("disasm {:08X}:", addr);
                for i in 0..n_instr {
                    let a = addr.wrapping_add((i * 4) as u32);
                    let instr = bus.read32::<BusRead>(a);
                    eprintln!("  {:08X}: {:08X}  {}", a, instr, disasm::desmonta(instr));
                }
            }

            if let Some(path) = &dump_vram {
                write_vram_dump(path, &bus);
            }

            return;
        }
        (bios_restored, _exe_restored, _disc_restored) => {
            bios_arg = bios_restored;
        }
    }

    if let Some(path) = bios_arg {
        let mut file = match std::fs::File::open(&path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!(
                    "Erro: nao foi possivel ler o arquivo BIOS '{}': {}",
                    path, e
                );
                std::process::exit(1);
            }
        };
        let mut data = Vec::new();
        if let Err(e) = file.read_to_end(&mut data) {
            eprintln!("Erro: falha ao ler '{}': {}", path, e);
            std::process::exit(1);
        }
        match Bios::from_bytes(data) {
            Ok(bios) => {
                use sha2::Digest;
                let hash = sha2::Sha256::digest(bios.raw());
                println!("BIOS: {} bytes, SHA-256: {:x}", bios.size(), hash);
            }
            Err(e) => {
                eprintln!("Erro: BIOS invalida: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    eprintln!("Uso: psx-cli [--version | --bios <caminho> [--exe <caminho>] [--disc <caminho>]]");
    eprintln!("     [--pad] [--press BOTAO@PASSO[:DURACAO]]");
    std::process::exit(1);
}
