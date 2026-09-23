use bincode::config::Config;
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
pub const VERSAO: u32 = 4;
/// A versao 3 e a 4 menos o disco: o corpo dela e o serial seguido da mesma `Maquina`.
pub const VERSAO_SEM_DISCO: u32 = 3;

const CABECALHO: usize = 12;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotError {
    Magico,
    Versao(u32),
    Corrompido,
    Codificacao,
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
            SnapshotError::Codificacao => write!(f, "falha ao codificar o estado da maquina"),
            SnapshotError::Serial { esperado, achado } => {
                write!(f, "save state e do jogo {achado}, nao do {esperado}")
            }
        }
    }
}

impl std::error::Error for SnapshotError {}

/// Qual disco estava na bandeja: depois de uma troca, o estado so faz sentido com ele.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoGravado {
    pub serial: String,
    pub caminho: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metadados {
    pub serial: String,
    pub disco: Option<DiscoGravado>,
}

impl Metadados {
    pub fn sem_disco(serial: &str) -> Self {
        Metadados {
            serial: serial.to_string(),
            disco: None,
        }
    }

    /// O disco que precisa voltar para a bandeja, se nao for o que ja esta nela.
    pub fn disco_diferente_de(&self, caminho_atual: &str) -> Option<&DiscoGravado> {
        self.disco.as_ref().filter(|d| d.caminho != caminho_atual)
    }
}

/// A imagem do disco e a BIOS ficam DE FORA de proposito: sao centenas de MB que o
/// frontend ja tem em maos, e um save state de 700 MB por slot nao serve para nada.
#[derive(Serialize, Deserialize)]
struct Maquina {
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

// Inteiro de tamanho fixo, nao varint: o save state fica do mesmo tamanho toda vez
// (a VRAM ocupa 1 MiB, nao "1 a 3 MiB conforme o que o jogo desenhou") e codifica mais rapido.
fn config() -> impl Config {
    bincode::config::standard().with_fixed_int_encoding()
}

fn cabecalho() -> Vec<u8> {
    let mut fora = Vec::with_capacity(CABECALHO);
    fora.extend_from_slice(MAGICO);
    fora.extend_from_slice(&VERSAO.to_le_bytes());
    fora
}

pub fn salva(cpu: &Cpu, bus: &Bus, serial: &str) -> Result<Vec<u8>, SnapshotError> {
    salva_com(cpu, bus, &Metadados::sem_disco(serial))
}

pub fn salva_com(cpu: &Cpu, bus: &Bus, metadados: &Metadados) -> Result<Vec<u8>, SnapshotError> {
    let maquina = Maquina {
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
    for parte in [
        bincode::serde::encode_to_vec(metadados, config()),
        bincode::serde::encode_to_vec(&maquina, config()),
    ] {
        fora.extend_from_slice(&parte.map_err(|_| SnapshotError::Codificacao)?);
    }
    Ok(fora)
}

fn corpo(bytes: &[u8]) -> Result<(u32, &[u8]), SnapshotError> {
    if bytes.len() < CABECALHO {
        return Err(SnapshotError::Corrompido);
    }
    if &bytes[0..8] != MAGICO {
        return Err(SnapshotError::Magico);
    }
    let versao = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    if versao != VERSAO && versao != VERSAO_SEM_DISCO {
        return Err(SnapshotError::Versao(versao));
    }
    Ok((versao, &bytes[CABECALHO..]))
}

fn decodifica_parte<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
) -> Result<(T, &[u8]), SnapshotError> {
    bincode::serde::decode_from_slice::<T, _>(bytes, config())
        .map(|(valor, lidos)| (valor, &bytes[lidos..]))
        .map_err(|_| SnapshotError::Corrompido)
}

fn decodifica_metadados(bytes: &[u8]) -> Result<(Metadados, &[u8]), SnapshotError> {
    match corpo(bytes)? {
        (VERSAO_SEM_DISCO, resto) => decodifica_parte::<String>(resto)
            .map(|(serial, resto)| (Metadados::sem_disco(&serial), resto)),
        (_, resto) => decodifica_parte::<Metadados>(resto),
    }
}

fn decodifica(bytes: &[u8]) -> Result<(Metadados, Maquina), SnapshotError> {
    let (metadados, resto) = decodifica_metadados(bytes)?;
    let (maquina, _) = decodifica_parte::<Maquina>(resto)?;
    Ok((metadados, maquina))
}

/// So o comeco do arquivo: o frontend decide que disco colocar antes de decodificar a maquina.
pub fn metadados_de(bytes: &[u8]) -> Result<Metadados, SnapshotError> {
    decodifica_metadados(bytes).map(|(metadados, _)| metadados)
}

pub fn serial_de(bytes: &[u8]) -> Option<String> {
    decodifica(bytes)
        .ok()
        .map(|(metadados, _)| metadados.serial)
}

/// Nada e escrito na maquina antes de o estado inteiro ter sido decodificado: um arquivo
/// truncado nao pode deixar o emulador meio restaurado.
pub fn carrega(
    cpu: &mut Cpu,
    bus: &mut Bus,
    bytes: &[u8],
    serial: &str,
) -> Result<(), SnapshotError> {
    let (metadados, estado) = decodifica(bytes)?;
    if metadados.serial != serial {
        return Err(SnapshotError::Serial {
            esperado: serial.to_string(),
            achado: metadados.serial,
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
