use std::path::{Path, PathBuf};

use psx_core::app::config::Config;
use psx_core::app::estados;
use psx_core::app::input_map::{Alvo, Eixos, Perfil, Teclado};
use psx_core::app::saves;
use psx_core::app::sessao;
use psx_core::app::troca::{PortaAberta, apertou_agora};
use psx_core::bus::{Bios, Bus, Ram};
use psx_core::cdrom_bin_cue::DiscLayout;
use psx_core::cpu::Cpu;
use psx_core::disc_image::DiscImage;
use psx_core::dualshock::Rumble;
use psx_core::snapshot::{self, DiscoGravado, Metadados};

use crate::audio::AudioOut;
use crate::gamepad::Leitura;

pub const SLOTS: u8 = estados::SLOTS;

type DiscoAberto = (PathBuf, DiscLayout, Box<dyn DiscImage>);

fn serial_do_cue(cue: &Path) -> String {
    crate::disco::identifica(cue)
        .ok()
        .and_then(|i| i.serial)
        .unwrap_or_default()
}

fn teclas_apertadas<'a>(ctx: &egui::Context, teclado: &'a Teclado) -> Vec<&'a str> {
    ctx.input(|i| {
        teclado
            .ligacoes()
            .iter()
            .map(|(_, nome)| nome.as_str())
            .filter(|nome| egui::Key::from_name(nome).is_some_and(|k| i.key_down(k)))
            .collect()
    })
}

const CPU_HZ: f64 = 33_868_800.0;
/// Drena o SPU antes de a fila interna dele (8192 quadros) encher: em 8x um único
/// `quadro()` de 50 ms produz 17.640 quadros.
const CICLOS_POR_FATIA_DE_AUDIO: u64 = 4096 * 768;

pub struct Emulador {
    cpu: Cpu,
    bus: Bus,
    audio: AudioOut,
    textura: Option<egui::ColorImage>,
    memcard: PathBuf,
    serial: String,
    pasta_de_saves: PathBuf,
    pub slot: u8,
    pub aviso: Option<String>,
    pub velocidade: u32,
    ultimo: std::time::Instant,
    jogado: f64,
    disco: PathBuf,
    serial_do_disco: String,
    porta: Option<PortaAberta>,
    modo_antes: bool,
}

impl Emulador {
    /// Um cartao por jogo: `cartoes/<serial>.mcd`, criado formatado na primeira vez. Cartao
    /// unico compartilhado enche com 15 blocos e obriga o usuario a apagar save alheio.
    pub fn novo(bios_bytes: Vec<u8>, serial: &str, config: &Config) -> Result<Self, String> {
        let bios = Bios::from_bytes(bios_bytes).map_err(|e| format!("BIOS invalida: {e:?}"))?;
        let mut bus = Bus::new(Ram::new(), bios);
        bus.sio_mut().connect_dualshock(true);

        let memcard = Path::new(&config.pasta_de_cartoes).join(saves::nome_do_cartao(serial));
        let bytes =
            std::fs::read(&memcard).unwrap_or_else(|_| psx_core::memcard::formatted_image());
        bus.sio_mut()
            .load_memory_card(&bytes)
            .map_err(|e| format!("memory card invalido: {e:?}"))?;

        Ok(Emulador {
            cpu: Cpu::new(),
            bus,
            audio: AudioOut::new(),
            textura: None,
            memcard,
            serial: serial.to_string(),
            pasta_de_saves: PathBuf::from(&config.pasta_de_saves),
            slot: config.slot_inicial,
            aviso: None,
            velocidade: 1,
            ultimo: std::time::Instant::now(),
            jogado: 0.0,
            disco: PathBuf::new(),
            serial_do_disco: String::new(),
            porta: None,
            modo_antes: false,
        })
    }

    pub fn serial(&self) -> &str {
        &self.serial
    }

    pub fn caminho_do_cartao(&self) -> &Path {
        &self.memcard
    }

    pub fn imagem_do_cartao(&self) -> Vec<u8> {
        self.bus.sio().memory_card_image()
    }

    /// Gerenciador de cartao com o jogo aberto: a imagem nova vai para o SIO (que volta a
    /// sinalizar "diretorio nao lido", como numa troca de cartao) e para o arquivo.
    pub fn troca_imagem_do_cartao(&mut self, imagem: &[u8]) -> Result<(), String> {
        self.bus
            .sio_mut()
            .load_memory_card(imagem)
            .map_err(|e| format!("memory card inválido: {e:?}"))?;
        grava_arquivo(&self.memcard, imagem)
    }

    pub fn insere_disco(&mut self, cue: &Path) -> Result<(), String> {
        let (layout, bin) = crate::disco::carrega(cue)?;
        self.bus.inject_disc_image(layout, bin);
        self.bus.cdrom_mut().insert_disc();
        self.passa_a_ter(cue);
        Ok(())
    }

    fn passa_a_ter(&mut self, cue: &Path) {
        self.disco = cue.to_path_buf();
        self.serial_do_disco = serial_do_cue(cue);
    }

    pub fn disco(&self) -> &Path {
        &self.disco
    }

    pub fn nome_do_disco(&self) -> String {
        self.disco
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default()
    }

    pub fn porta_aberta(&self) -> bool {
        self.bus.lid_open()
    }

    /// Abre a porta, poe o disco novo na bandeja e agenda o fechamento para ~1 s emulado
    /// depois: fechar no mesmo instante o jogo nem percebe que houve troca.
    pub fn troca_disco(&mut self, cue: &Path) {
        let (layout, bin) = match crate::disco::carrega(cue) {
            Ok(d) => d,
            Err(e) => {
                self.aviso = Some(e);
                return;
            }
        };
        self.bus.open_lid();
        self.bus.swap_disc_image(layout, bin);
        self.porta = Some(PortaAberta::desde(self.bus.total_cycles()));
        self.passa_a_ter(cue);
        self.aviso = Some(format!("trocando para {}", self.nome_do_disco()));
    }

    /// Estado salvo com a porta aberta volta com ela aberta e sem ninguem para fechar:
    /// sem agendamento, reagenda a partir de agora.
    fn cuida_da_porta(&mut self) {
        let agora = self.bus.total_cycles();
        match self.porta {
            Some(p) if p.deve_fechar(agora) => {
                if self.bus.lid_open() {
                    self.bus.close_lid();
                }
                self.porta = None;
            }
            Some(_) => {}
            None if self.bus.lid_open() => self.porta = Some(PortaAberta::desde(agora)),
            None => {}
        }
    }

    pub fn caminho_do_slot(&self, slot: u8) -> PathBuf {
        caminho_do_estado(&self.pasta_de_saves, &self.serial, slot)
    }

    pub fn pasta_de_saves(&self) -> &Path {
        &self.pasta_de_saves
    }

    pub fn salva_no_slot(&mut self, slot: u8) {
        self.slot = slot.min(SLOTS - 1);
        self.salva_estado();
    }

    pub fn carrega_do_slot(&mut self, slot: u8) {
        self.slot = slot.min(SLOTS - 1);
        self.carrega_estado();
    }

    /// F5. Um slot por arquivo: sobrescrever o anterior e o comportamento esperado, mas
    /// perder o save por falta da pasta nao e — por isso a pasta e criada aqui.
    pub fn salva_estado(&mut self) {
        let caminho = self.caminho_do_slot(self.slot);
        let bytes = match snapshot::salva_com(&self.cpu, &self.bus, &self.metadados()) {
            Ok(b) => b,
            Err(e) => {
                self.aviso = Some(format!("slot {}: {e}", self.slot));
                return;
            }
        };
        if let Err(e) = grava_arquivo(&caminho, &bytes) {
            self.aviso = Some(format!("não consegui gravar o slot {}: {e}", self.slot));
            return;
        }
        let miniatura = caminho_da_miniatura(&self.pasta_de_saves, &self.serial, self.slot);
        let _ = std::fs::remove_file(&miniatura);
        let extra = match self.grava_miniatura(&miniatura) {
            Ok(()) => String::new(),
            Err(e) => format!(" (sem miniatura: {e})"),
        };
        self.aviso = Some(format!(
            "slot {} salvo ({} KiB){extra}",
            self.slot,
            bytes.len() / 1024
        ));
    }

    fn grava_miniatura(&self, caminho: &Path) -> Result<(), String> {
        let fb = self
            .bus
            .gpu()
            .framebuffer_for_display()
            .ok_or("display desligado")?;
        let rgba = estados::miniatura(&fb.data, fb.width as usize, fb.height as usize)
            .ok_or("quadro vazio")?;
        grava_png(caminho, &rgba)
    }

    fn metadados(&self) -> Metadados {
        let disco = (!self.disco.as_os_str().is_empty()).then(|| DiscoGravado {
            serial: self.serial_do_disco.clone(),
            caminho: self.disco.to_string_lossy().to_string(),
        });
        Metadados {
            serial: self.serial.clone(),
            disco,
        }
    }

    /// F8. Estado recusado nao mexe na maquina: o `carrega` decodifica tudo antes de
    /// escrever qualquer campo, e o disco gravado e aberto antes dele.
    pub fn carrega_estado(&mut self) {
        let caminho = self.caminho_do_slot(self.slot);
        let Ok(bytes) = std::fs::read(&caminho) else {
            self.aviso = Some(format!("slot {} vazio", self.slot));
            return;
        };
        let troca = match snapshot::metadados_de(&bytes)
            .map_err(|e| e.to_string())
            .and_then(|m| self.disco_do_estado(&m))
        {
            Ok(t) => t,
            Err(e) => {
                self.aviso = Some(format!("slot {}: {e}", self.slot));
                return;
            }
        };
        if let Err(e) = snapshot::carrega(&mut self.cpu, &mut self.bus, &bytes, &self.serial) {
            self.aviso = Some(format!("slot {}: {e}", self.slot));
            return;
        }
        self.bus.sio_mut().connect_dualshock(true);
        self.porta = None;
        self.aviso = Some(match troca {
            Some((cue, layout, bin)) => {
                self.bus.inject_disc_image(layout, bin);
                self.passa_a_ter(&cue);
                format!("slot {} carregado com {}", self.slot, self.nome_do_disco())
            }
            None => format!("slot {} carregado", self.slot),
        });
    }

    /// Estado salvo com outro disco na bandeja (antes ou depois de uma troca): esse disco
    /// volta junto, e se ele sumiu ou mudou o estado e recusado.
    fn disco_do_estado(&self, metadados: &Metadados) -> Result<Option<DiscoAberto>, String> {
        let atual = self.disco.to_string_lossy();
        let Some(gravado) = metadados.disco_diferente_de(&atual) else {
            return Ok(None);
        };
        let cue = PathBuf::from(&gravado.caminho);
        let rotulo = format!("'{}' ({})", gravado.caminho, gravado.serial);
        if !cue.exists() {
            return Err(format!(
                "o estado usa o disco {rotulo}, que não foi encontrado"
            ));
        }
        let achado = serial_do_cue(&cue);
        if achado != gravado.serial {
            return Err(format!(
                "o estado usa o disco {rotulo}, mas o arquivo agora é do {achado}"
            ));
        }
        let (layout, bin) = crate::disco::carrega(&cue)?;
        Ok(Some((cue, layout, bin)))
    }

    pub fn slot_existe(&self, slot: u8) -> bool {
        self.caminho_do_slot(slot).exists()
    }

    /// Teclado e controle valem ao mesmo tempo: o pad do PS1 recebe a UNIAO dos dois,
    /// que e o que um jogador que larga o controle e pega o teclado espera.
    pub fn entrada(&mut self, ctx: &egui::Context, perfil: &Perfil, controle: &Leitura) {
        let teclado = perfil.teclado();
        let modo_agora = perfil
            .analog()
            .is_some_and(|e| controle.entradas.contains(&e));
        let tecla_analog = teclado
            .tecla_de(Alvo::Analog)
            .and_then(egui::Key::from_name)
            .is_some_and(|k| ctx.input(|i| i.key_pressed(k)));
        if tecla_analog || apertou_agora(self.modo_antes, modo_agora) {
            self.aperta_analog();
        }
        self.modo_antes = modo_agora;

        let apertadas = teclas_apertadas(ctx, teclado);
        let analogico = self.bus.sio().analog_mode();
        let botoes =
            teclado.palavra(&apertadas) & perfil.palavra_no_modo(&controle.entradas, analogico);
        self.bus.sio_mut().set_buttons(botoes);
        let eixos = controle.eixos.une(&teclado.eixos(&apertadas));
        self.bus.sio_mut().set_sticks(eixos.sticks());
    }

    fn aperta_analog(&mut self) {
        let trocou = self.bus.sio_mut().press_analog_button();
        self.aviso = Some(match (trocou, self.bus.sio().analog_mode()) {
            (false, _) => "modo analogico travado pelo jogo".to_string(),
            (true, true) => "modo analogico (LED aceso)".to_string(),
            (true, false) => "modo digital".to_string(),
        });
    }

    pub fn modo_analogico(&self) -> bool {
        self.bus.sio().analog_mode()
    }

    pub fn vibracao(&self) -> Rumble {
        self.bus.sio().rumble()
    }

    pub fn quadro(&mut self, ganho: f32) {
        let agora = std::time::Instant::now();
        let dt = (agora - self.ultimo).as_secs_f64().min(0.05);
        self.ultimo = agora;
        self.jogado += dt;
        let alvo =
            self.bus.total_cycles() + (dt * CPU_HZ * f64::from(self.velocidade.max(1))) as u64;
        while self.bus.total_cycles() < alvo {
            let fim_da_fatia = (self.bus.total_cycles() + CICLOS_POR_FATIA_DE_AUDIO).min(alvo);
            while self.bus.total_cycles() < fim_da_fatia {
                self.cpu.step(&mut self.bus);
            }
            let quadros = self.bus.drain_audio();
            self.audio.push(&quadros, ganho);
        }
        self.cuida_da_porta();
        self.salva_memcard();
        self.atualiza_textura();
    }

    fn salva_memcard(&mut self) {
        if !self.bus.sio().memory_card_dirty() {
            return;
        }
        if let Some(pai) = self.memcard.parent() {
            let _ = std::fs::create_dir_all(pai);
        }
        let _ = std::fs::write(&self.memcard, self.bus.sio().memory_card_image());
    }

    fn atualiza_textura(&mut self) {
        let Some(fb) = self
            .bus
            .gpu()
            .framebuffer_for_display()
            .filter(|fb| fb.width > 0 && fb.height > 0)
        else {
            self.textura = None;
            return;
        };
        let tamanho = [fb.width as usize, fb.height as usize];
        self.textura = Some(egui::ColorImage::from_rgba_unmultiplied(tamanho, &fb.data));
    }

    pub fn textura(&self) -> Option<&egui::ColorImage> {
        self.textura.as_ref()
    }

    pub fn audio_ativo(&self) -> bool {
        self.audio.ativo()
    }

    pub fn audio_hz(&self) -> u32 {
        self.audio.device_hz()
    }

    /// Tempo REAL com o jogo aberto; fast-forward 8x por uma hora conta uma hora.
    pub fn segundos_jogados(&self) -> u64 {
        self.jogado as u64
    }

    /// Parado, o anel de audio esvazia sozinho com fade: nao ha o que cortar aqui.
    pub fn pausa(&mut self) {}

    /// Sem isto o primeiro quadro depois da pausa contaria o tempo parado como jogado.
    pub fn retoma(&mut self) {
        self.ultimo = std::time::Instant::now();
    }

    pub fn solta_tudo(&mut self) {
        self.bus.sio_mut().set_buttons(0xFFFF);
        self.bus.sio_mut().set_sticks(Eixos::default().sticks());
    }

    pub fn ciclos(&self) -> u64 {
        self.bus.total_cycles()
    }

    pub fn ciclos_por_quadro(&self) -> u64 {
        self.bus.gpu().frame_cycles()
    }

    pub fn troca_velocidade(&mut self) {
        self.velocidade = sessao::proxima_velocidade(self.velocidade);
    }
}

pub fn caminho_do_estado(pasta: &Path, serial: &str, slot: u8) -> PathBuf {
    pasta.join(estados::nome_do_estado(serial, slot))
}

pub fn caminho_da_miniatura(pasta: &Path, serial: &str, slot: u8) -> PathBuf {
    pasta.join(estados::nome_da_miniatura(serial, slot))
}

pub fn apaga_estado(pasta: &Path, serial: &str, slot: u8) -> Result<(), String> {
    let _ = std::fs::remove_file(caminho_da_miniatura(pasta, serial, slot));
    let estado = caminho_do_estado(pasta, serial, slot);
    std::fs::remove_file(&estado).map_err(|e| format!("apagando '{}': {e}", estado.display()))
}

pub fn grava_arquivo(caminho: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(pai) = caminho.parent() {
        std::fs::create_dir_all(pai).map_err(|e| format!("criando '{}': {e}", pai.display()))?;
    }
    std::fs::write(caminho, bytes).map_err(|e| format!("gravando '{}': {e}", caminho.display()))
}

fn grava_png(caminho: &Path, rgba: &[u8]) -> Result<(), String> {
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(
        &mut bytes,
        estados::MINIATURA_LARGURA as u32,
        estados::MINIATURA_ALTURA as u32,
    );
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .and_then(|mut w| w.write_image_data(rgba))
        .map_err(|e| e.to_string())?;
    grava_arquivo(caminho, &bytes)
}

pub fn le_png(caminho: &Path) -> Option<egui::ColorImage> {
    let bytes = std::fs::read(caminho).ok()?;
    let mut leitor = png::Decoder::new(std::io::Cursor::new(bytes))
        .read_info()
        .ok()?;
    let mut buffer = vec![0u8; leitor.output_buffer_size()?];
    let info = leitor.next_frame(&mut buffer).ok()?;
    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        return None;
    }
    let tamanho = [info.width as usize, info.height as usize];
    let dados = buffer.get(..info.buffer_size())?;
    Some(egui::ColorImage::from_rgba_unmultiplied(tamanho, dados))
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn miniatura_png_vai_e_volta() {
        let dir = std::env::temp_dir().join(format!("psx-rs-png-{}", std::process::id()));
        let caminho = caminho_da_miniatura(&dir, "SCUS-94900", 2);
        let rgba: Vec<u8> = (0..estados::MINIATURA_LARGURA * estados::MINIATURA_ALTURA)
            .flat_map(|i| [(i % 256) as u8, 7, 9, 255])
            .collect();
        grava_png(&caminho, &rgba).expect("grava png");
        let volta = le_png(&caminho).expect("le png");
        assert_eq!(
            volta.size,
            [estados::MINIATURA_LARGURA, estados::MINIATURA_ALTURA]
        );
        assert_eq!(volta.pixels[3], egui::Color32::from_rgb(3, 7, 9));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
