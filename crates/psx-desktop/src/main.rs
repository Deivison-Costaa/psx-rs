use psx_core::gpu::Gpu;

struct PsxDesktop {
    gpu: Gpu,
    texture: Option<egui::ColorImage>,
}

impl PsxDesktop {
    fn new() -> Self {
        PsxDesktop {
            gpu: Gpu::new(),
            texture: None,
        }
    }

    fn update_texture(&mut self) {
        let fb = match self.gpu.framebuffer_for_display() {
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
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "psx-rs",
        options,
        Box::new(|_cc| Ok(Box::new(PsxDesktop::new()))),
    )
}
