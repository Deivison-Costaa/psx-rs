#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrackType {
    Audio,
    Mode1_2048,
    Mode1_2352,
    Mode2_2336,
    Mode2_2352,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackInfo {
    pub number: u8,
    pub file: String,
    pub start_lba: u32,
    pub track_type: TrackType,
    pub index01_mm: u8,
    pub index01_ss: u8,
    pub index01_ff: u8,
    pub index00_mm: Option<u8>,
    pub index00_ss: Option<u8>,
    pub index00_ff: Option<u8>,
    pub pregap_mm: Option<u8>,
    pub pregap_ss: Option<u8>,
    pub pregap_ff: Option<u8>,
}

impl TrackInfo {
    pub fn index01_em_quadros(&self) -> u32 {
        self.index01_mm as u32 * 60 * 75 + self.index01_ss as u32 * 75 + self.index01_ff as u32
    }

    pub fn bin_offset(&self) -> u32 {
        let total_frames =
            self.index01_mm as u32 * 60 * 75 + self.index01_ss as u32 * 75 + self.index01_ff as u32;
        total_frames * 2352
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscLayout {
    pub bin_path: String,
    pub tracks: Vec<TrackInfo>,
}

impl DiscLayout {
    // Imagem rasgada trilha a trilha tem um arquivo por trilha, cada um comecando no proprio
    // LBA 0. Concatenar na ordem das trilhas reconstroi a imagem indexavel por LBA absoluto,
    // que e o que `read_sector_from_disc` espera.
    // Num `.cue` de arquivo unico o INDEX 01 ja e o LBA. Num rip por trilha ele e relativo ao
    // proprio arquivo, e o absoluto so aparece somando os tamanhos dos anteriores — que o
    // parser nao pode medir sem I/O (R3). Quem le os arquivos passa os tamanhos aqui.
    pub fn atribui_lbas_absolutos(&mut self, setores_por_arquivo: &[u32]) {
        let arquivos = self.arquivos_em_ordem();
        let mut base: Vec<(String, u32)> = Vec::new();
        let mut acumulado = 0u32;
        for (i, nome) in arquivos.iter().enumerate() {
            base.push((nome.clone(), acumulado));
            acumulado = acumulado.saturating_add(setores_por_arquivo.get(i).copied().unwrap_or(0));
        }
        for t in &mut self.tracks {
            let deslocamento = base
                .iter()
                .find(|(nome, _)| *nome == t.file)
                .map(|(_, o)| *o)
                .unwrap_or(0);
            t.start_lba = deslocamento + t.index01_em_quadros();
        }
    }

    pub fn arquivos_em_ordem(&self) -> Vec<String> {
        let mut fora = Vec::new();
        for t in &self.tracks {
            if !t.file.is_empty() && !fora.contains(&t.file) {
                fora.push(t.file.clone());
            }
        }
        if fora.is_empty() && !self.bin_path.is_empty() {
            fora.push(self.bin_path.clone());
        }
        fora
    }

    pub fn read_data_sector(&self, bin_data: &[u8], sector_index: u32) -> Vec<u8> {
        let offset = sector_index as usize * 2352;
        let start = offset + 0x10;
        let end = start + 2048;
        bin_data[start..end].to_vec()
    }
}

fn parse_mm_ss_ff(s: &str) -> Option<(u8, u8, u8)> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 3 {
        return None;
    }
    let mm = parts[0].parse::<u8>().ok()?;
    let ss = parts[1].parse::<u8>().ok()?;
    let ff = parts[2].parse::<u8>().ok()?;
    Some((mm, ss, ff))
}

fn parse_track_type(s: &str) -> Option<TrackType> {
    match s {
        "AUDIO" => Some(TrackType::Audio),
        "MODE1/2048" => Some(TrackType::Mode1_2048),
        "MODE1/2352" => Some(TrackType::Mode1_2352),
        "MODE2/2336" => Some(TrackType::Mode2_2336),
        "MODE2/2352" => Some(TrackType::Mode2_2352),
        _ => None,
    }
}

pub fn parse_cue(cue_text: &str) -> DiscLayout {
    let mut bin_path = String::new();
    let mut arquivo_atual = String::new();
    let mut tracks: Vec<TrackInfo> = Vec::new();
    let mut current_track: Option<TrackInfo> = None;

    for line in cue_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("REM") {
            continue;
        }

        let upper = trimmed.to_uppercase();

        if upper.starts_with("FILE ") {
            let rest = &trimmed[5..].trim();
            if let Some(quote_end) = rest.rfind('"') {
                if rest.starts_with('"') && quote_end > 0 {
                    arquivo_atual = rest[1..quote_end].to_string();
                    if bin_path.is_empty() {
                        bin_path = arquivo_atual.clone();
                    }
                }
            }
            continue;
        }

        if upper.starts_with("TRACK ") {
            if let Some(current) = current_track.take() {
                tracks.push(current);
            }
            let rest = &trimmed[6..].trim();
            let parts: Vec<&str> = rest.splitn(2, ' ').collect();
            if parts.len() >= 2 {
                if let Ok(num) = parts[0].parse::<u8>() {
                    if let Some(tt) = parse_track_type(parts[1]) {
                        current_track = Some(TrackInfo {
                            number: num,
                            file: arquivo_atual.clone(),
                            start_lba: 0,
                            track_type: tt,
                            index01_mm: 0,
                            index01_ss: 0,
                            index01_ff: 0,
                            index00_mm: None,
                            index00_ss: None,
                            index00_ff: None,
                            pregap_mm: None,
                            pregap_ss: None,
                            pregap_ff: None,
                        });
                    }
                }
            }
            continue;
        }

        if let Some(ref mut track) = current_track {
            if upper.starts_with("INDEX ") {
                let rest = &trimmed[6..].trim();
                let parts: Vec<&str> = rest.splitn(2, ' ').collect();
                if parts.len() >= 2 {
                    if let Ok(idx) = parts[0].parse::<u8>() {
                        if let Some((mm, ss, ff)) = parse_mm_ss_ff(parts[1]) {
                            if idx == 0 {
                                track.index00_mm = Some(mm);
                                track.index00_ss = Some(ss);
                                track.index00_ff = Some(ff);
                            } else if idx == 1 {
                                track.index01_mm = mm;
                                track.index01_ss = ss;
                                track.index01_ff = ff;
                            }
                        }
                    }
                }
            } else if upper.starts_with("PREGAP ") {
                let rest = &trimmed[7..].trim();
                if let Some((mm, ss, ff)) = parse_mm_ss_ff(rest) {
                    track.pregap_mm = Some(mm);
                    track.pregap_ss = Some(ss);
                    track.pregap_ff = Some(ff);
                }
            }
        }
    }

    if let Some(current) = current_track {
        tracks.push(current);
    }

    DiscLayout { bin_path, tracks }
}
