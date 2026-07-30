use psx_core::bus::{Bios, Bus, Ram};
use psx_core::cpu::Cpu;

struct PsxDesktop {
    cpu: Cpu,
    bus: Bus,
    texture: Option<egui::ColorImage>,
    steps_per_frame: usize,
}

impl PsxDesktop {
    fn new(bios_path: &str) -> Result<Self, String> {
        let bios_data = std::fs::read(bios_path)
            .map_err(|e| format!("Erro lendo BIOS '{}': {}", bios_path, e))?;
        let bios = Bios::from_bytes(bios_data).map_err(|e| format!("BIOS invalida: {:?}", e))?;
        let ram = Ram::new();
        let mut bus = Bus::new(ram, bios);
        let cpu = Cpu::new();
        let steps_per_frame = bus.gpu().frame_cycles() as usize;

        bus.sio_mut().connect_digital_pad(true);

        Ok(PsxDesktop {
            cpu,
            bus,
            texture: None,
            steps_per_frame,
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
        for _step in 0..self.steps_per_frame {
            self.cpu.step(&mut self.bus);
        }
        self.update_texture();
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(ref texture) = self.texture {
                let handle = ctx.load_texture(
                    "framebuffer",
                    texture.clone(),
                    egui::TextureOptions::NEAREST,
                );
                ui.image(&handle);
            } else {
                ui.label("Display desligado");
            }
        });
        ctx.request_repaint();
    }
}

fn main() -> Result<(), eframe::Error> {
    let bios_path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Uso: psx-desktop <BIOS.bin>");
        std::process::exit(1);
    });

    let app = match PsxDesktop::new(&bios_path) {
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
