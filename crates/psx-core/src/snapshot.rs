use bincode::config::Configuration;
use serde::{Deserialize, Serialize};

use crate::bus::{Bcc, Bus, MemCtrl, Ram, Scratchpad};
use crate::cdrom::Cdrom;
use crate::cpu::Cpu;
use crate::dma::Dma;
use crate::gpu::Gpu;
use crate::gte::Gte;
use crate::irq::Irq;
use crate::mdec::Mdec;
use crate::scheduler::Scheduler;
use crate::sio::Sio;
use crate::spu::Spu;
use crate::timers::Timers;

pub const MAGICO: &[u8; 8] = b"PSXRS-ST";
pub const VERSAO: u32 = 1;

const CABECALHO: usize = 12;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotError {
    Magico,
    Versao(u32),
    Corrompido,
    Serial { esperado: String, achado: String },
}

impl std::fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SnapshotError::Magico => write!(f, "nao e um save state do psx-rs"),
            SnapshotError::Versao(v) => {
                write!(
                    f,
                    "save state versao {v}; este emulador le a versao {VERSAO}"
                )
            }
            SnapshotError::Corrompido => write!(f, "save state truncado ou corrompido"),
            SnapshotError::Serial { esperado, achado } => {
                write!(f, "save state e do jogo {achado}, nao do {esperado}")
            }
        }
    }
}

impl std::error::Error for SnapshotError {}

/// A imagem do disco e a BIOS ficam DE FORA de proposito: sao centenas de MB que o
/// frontend ja tem em maos, e um save state de 700 MB por slot nao serve para nada.
#[derive(Serialize, Deserialize)]
struct Estado {
    serial: String,
    cpu: Cpu,
    ram: Ram,
    scratchpad: Scratchpad,
    gpu: Gpu,
    gte: Gte,
    irq: Irq,
    dma: Dma,
    cdrom: Cdrom,
    timers: Timers,
    sio: Sio,
    mdec: Mdec,
    spu: Spu,
    mem_ctrl: MemCtrl,
    bcc: Bcc,
    tty_buffer: Vec<u8>,
    scheduler: Scheduler,
    total_cycles: u64,
}

fn config() -> Configuration {
    bincode::config::standard()
}

fn cabecalho() -> Vec<u8> {
    let mut fora = Vec::with_capacity(CABECALHO);
    fora.extend_from_slice(MAGICO);
    fora.extend_from_slice(&VERSAO.to_le_bytes());
    fora
}

pub fn salva(cpu: &Cpu, bus: &Bus, serial: &str) -> Vec<u8> {
    let estado = Estado {
        serial: serial.to_string(),
        cpu: cpu.clone(),
        ram: bus.ram.clone(),
        scratchpad: bus.scratchpad.clone(),
        gpu: bus.gpu.clone(),
        gte: bus.gte.clone(),
        irq: bus.irq.clone(),
        dma: bus.dma.clone(),
        cdrom: bus.cdrom.clone(),
        timers: bus.timers.clone(),
        sio: bus.sio.clone(),
        mdec: bus.mdec.clone(),
        spu: bus.spu.clone(),
        mem_ctrl: bus.mem_ctrl.clone(),
        bcc: bus.bcc.clone(),
        tty_buffer: bus.tty_buffer.clone(),
        scheduler: bus.scheduler.clone(),
        total_cycles: bus.total_cycles,
    };
    let mut fora = cabecalho();
    let corpo = bincode::serde::encode_to_vec(&estado, config()).unwrap_or_default();
    fora.extend_from_slice(&corpo);
    fora
}

fn corpo(bytes: &[u8]) -> Result<&[u8], SnapshotError> {
    if bytes.len() < CABECALHO {
        return Err(SnapshotError::Corrompido);
    }
    if &bytes[0..8] != MAGICO {
        return Err(SnapshotError::Magico);
    }
    let versao = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    if versao != VERSAO {
        return Err(SnapshotError::Versao(versao));
    }
    Ok(&bytes[CABECALHO..])
}

fn decodifica(bytes: &[u8]) -> Result<Estado, SnapshotError> {
    let corpo = corpo(bytes)?;
    bincode::serde::decode_from_slice::<Estado, _>(corpo, config())
        .map(|(estado, _)| estado)
        .map_err(|_| SnapshotError::Corrompido)
}

pub fn serial_de(bytes: &[u8]) -> Option<String> {
    decodifica(bytes).ok().map(|e| e.serial)
}

/// Nada e escrito na maquina antes de o estado inteiro ter sido decodificado: um arquivo
/// truncado nao pode deixar o emulador meio restaurado.
pub fn carrega(
    cpu: &mut Cpu,
    bus: &mut Bus,
    bytes: &[u8],
    serial: &str,
) -> Result<(), SnapshotError> {
    let estado = decodifica(bytes)?;
    if estado.serial != serial {
        return Err(SnapshotError::Serial {
            esperado: serial.to_string(),
            achado: estado.serial,
        });
    }

    *cpu = estado.cpu;
    bus.ram = estado.ram;
    bus.scratchpad = estado.scratchpad;
    bus.gpu = estado.gpu;
    bus.gte = estado.gte;
    bus.irq = estado.irq;
    bus.dma = estado.dma;
    bus.cdrom = estado.cdrom;
    bus.timers = estado.timers;
    bus.sio = estado.sio;
    bus.mdec = estado.mdec;
    bus.spu = estado.spu;
    bus.mem_ctrl = estado.mem_ctrl;
    bus.bcc = estado.bcc;
    bus.tty_buffer = estado.tty_buffer;
    bus.scheduler = estado.scheduler;
    bus.total_cycles = estado.total_cycles;
    Ok(())
}
