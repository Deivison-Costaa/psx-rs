mod audio;

use audio::AudioOut;
use psx_core::bus::{Bios, Bus, Ram};
use psx_core::cdrom_bin_cue::{DiscLayout, parse_cue};
use psx_core::cpu::Cpu;

const CPU_HZ: f64 = 33_868_800.0;

struct PsxDesktop {
    cpu: Cpu,
    bus: Bus,
    texture: Option<egui::ColorImage>,
    ultimo: std::time::Instant,
    velocidade: f64,
    audio: AudioOut,
    memcard: Option<String>,
}

fn load_disc(disc_path: &str) -> Result<(DiscLayout, Vec<u8>), String> {
    let cue_text = std::fs::read_to_string(disc_path)
        .map_err(|e| format!("Erro lendo CUE '{}': {}", disc_path, e))?;
    let mut layout = parse_cue(&cue_text);
    let cue_dir = std::path::Path::new(disc_path)
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    let mut setores: Vec<u32> = Vec::new();
    let mut bin_data: Vec<u8> = Vec::new();
    for arquivo in layout.arquivos_em_ordem() {
        let bin_path = cue_dir.join(&arquivo);
        let d = std::fs::read(&bin_path)
            .map_err(|e| format!("Erro lendo BIN '{}': {}", bin_path.display(), e))?;
        setores.push((d.len() / 2352) as u32);
        bin_data.extend_from_slice(&d);
    }
    layout.atribui_lbas_absolutos(&setores);
    Ok((layout, bin_data))
}

impl PsxDesktop {
    fn new(bios_path: &str, disc_path: Option<&str>, memcard: Option<String>) -> Result<Self, String> {
        let bios_data = std::fs::read(bios_path)
            .map_err(|e| format!("Erro lendo BIOS '{}': {}", bios_path, e))?;
        let bios = Bios::from_bytes(bios_data).map_err(|e| format!("BIOS invalida: {:?}", e))?;
        let ram = Ram::new();
        let mut bus = Bus::new(ram, bios);
        let cpu = Cpu::new();

        bus.sio_mut().connect_digital_pad(true);
        if let Some(caminho) = memcard.as_deref() {
            let bytes =
                std::fs::read(caminho).unwrap_or_else(|_| vec![0u8; psx_core::memcard::CARD_BYTES]);
            if let Err(e) = bus.sio_mut().load_memory_card(&bytes) {
                return Err(format!("memory card invalido: {e:?}"));
            }
        }
        if let Some(cue) = disc_path {
            let (layout, bin_data) = load_disc(cue)?;
            bus.inject_disc(layout, bin_data);
            bus.cdrom_mut().insert_disc();
        }

        Ok(PsxDesktop {
            cpu,
            bus,
            texture: None,
            ultimo: std::time::Instant::now(),
            velocidade: 1.0,
            audio: AudioOut::new(),
            memcard,
        })
    }

    fn poll_input(&mut self, ctx: &egui::Context) {
        let mut buttons: u16 = 0xFFFF;
        for (key, bit) in [
            (egui::Key::ArrowUp, 4u32),
            (egui::Key::ArrowDown, 6),
            (egui::Key::ArrowLeft, 7),
            (egui::Key::ArrowRight, 5),
            (egui::Key::Z, 14),
            (egui::Key::Space, 13),
            (egui::Key::A, 15),
            (egui::Key::S, 12),
            (egui::Key::Enter, 3),
            (egui::Key::Tab, 0),
            (egui::Key::D, 10),
            (egui::Key::F, 11),
            (egui::Key::E, 8),
            (egui::Key::R, 9),
        ] {
            if ctx.input(|i| i.key_down(key)) {
                buttons &= !(1u16 << bit);
            }
        }
        self.bus.sio_mut().set_buttons(buttons);
    }

    fn salva_memcard(&mut self) {
        let Some(caminho) = self.memcard.clone() else {
            return;
        };
        if self.bus.sio().memory_card_dirty() {
            let _ = std::fs::write(caminho, self.bus.sio().memory_card_image());
        }
    }

    fn update_texture(&mut self) {
        let fb = match self.bus.gpu().framebuffer_for_display() {
            Some(fb) => fb,
            None => {
                self.texture = None;
                return;
            }
        };
        let size = [fb.width as usize, fb.height as usize];
        self.texture = Some(egui::ColorImage::from_rgba_unmultiplied(size, &fb.data));
    }
}

impl eframe::App for PsxDesktop {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_input(ctx);
        let agora = std::time::Instant::now();
        let dt = (agora - self.ultimo).as_secs_f64().min(0.05);
        self.ultimo = agora;
        let alvo = self.bus.total_cycles() + (dt * CPU_HZ * self.velocidade) as u64;
        while self.bus.total_cycles() < alvo {
            self.cpu.step(&mut self.bus);
        }
        let quadros = self.bus.drain_audio();
        self.audio.push(&quadros);
        self.salva_memcard();
        self.update_texture();
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if self.audio.ativo() {
                    ui.label(format!("Audio: {} Hz", self.audio.device_hz()));
                } else {
                    ui.label("Audio desligado (sem dispositivo de saida)");
                }
                if let Some(ref texture) = self.texture {
                    let [w, h] = texture.size;
                    ui.label(format!("Video: {}x{}", w, h));
                }
                ui.add(
                    egui::Slider::new(&mut self.velocidade, 0.25..=2.0)
                        .text("Velocidade")
                        .fixed_decimals(2),
                );
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(ref texture) = self.texture {
                let handle = ctx.load_texture(
                    "framebuffer",
                    texture.clone(),
                    egui::TextureOptions::NEAREST,
                );
                let livre = ui.available_size();
                let lado = (livre.x / 4.0).min(livre.y / 3.0).max(1.0);
                ui.centered_and_justified(|ui| {
                    ui.add(
                        egui::Image::new(&handle)
                            .fit_to_exact_size(egui::vec2(lado * 4.0, lado * 3.0)),
                    );
                });
            } else {
                ui.label("Display desligado");
            }
        });
        ctx.request_repaint();
    }
}

fn main() -> Result<(), eframe::Error> {
    let mut bios_path: Option<String> = None;
    let mut disc: Option<String> = None;
    let mut memcard: Option<String> = None;
    for arg in std::env::args().skip(1) {
        if arg.to_lowercase().ends_with(".cue") {
            disc = Some(arg);
        } else if bios_path.is_none() {
            bios_path = Some(arg);
        } else {
            memcard = Some(arg);
        }
    }
    let bios_path = bios_path.unwrap_or_else(|| {
        eprintln!("Uso: psx-desktop <BIOS.bin> [jogo.cue] [cartao.mcd]");
        std::process::exit(1);
    });

    let app = match PsxDesktop::new(&bios_path, disc.as_deref(), memcard) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native("psx-rs", options, Box::new(move |_cc| Ok(Box::new(app))))
}
