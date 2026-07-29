use psx_core::cdrom_bin_cue::{TrackType, parse_cue};

#[test]
fn parse_cue_extrai_bin_path() {
    let cue = r#"FILE "game.bin" BINARY
  TRACK 01 MODE2/2352
    INDEX 01 00:00:00
"#;
    let layout = parse_cue(cue);
    assert_eq!(layout.bin_path, "game.bin");
}

#[test]
fn parse_cue_extrai_uma_track_mode2_2352() {
    let cue = r#"FILE "game.bin" BINARY
  TRACK 01 MODE2/2352
    INDEX 01 00:00:00
"#;
    let layout = parse_cue(cue);
    assert_eq!(layout.tracks.len(), 1);
    let t = &layout.tracks[0];
    assert_eq!(t.number, 1);
    assert!(matches!(t.track_type, TrackType::Mode2_2352));
    assert_eq!(t.index01_mm, 0);
    assert_eq!(t.index01_ss, 0);
    assert_eq!(t.index01_ff, 0);
}

#[test]
fn parse_cue_extrai_track_audio() {
    let cue = r#"FILE "game.bin" BINARY
  TRACK 01 MODE2/2352
    INDEX 01 00:00:00
  TRACK 02 AUDIO
    INDEX 00 02:10:30
    INDEX 01 02:12:30
"#;
    let layout = parse_cue(cue);
    assert_eq!(layout.tracks.len(), 2);
    let t = &layout.tracks[1];
    assert_eq!(t.number, 2);
    assert!(matches!(t.track_type, TrackType::Audio));
    assert_eq!(t.index01_mm, 2);
    assert_eq!(t.index01_ss, 12);
    assert_eq!(t.index01_ff, 30);
    assert!(t.index00_mm.is_some());
}

#[test]
fn parse_cue_extrai_track_mode1_2048() {
    let cue = r#"FILE "game.bin" BINARY
  TRACK 01 MODE1/2048
    INDEX 01 00:00:00
"#;
    let layout = parse_cue(cue);
    let t = &layout.tracks[0];
    assert!(matches!(t.track_type, TrackType::Mode1_2048));
}

#[test]
fn parse_cue_extrai_pregap() {
    let cue = r#"FILE "game.bin" BINARY
  TRACK 01 MODE2/2352
    INDEX 01 00:00:00
  TRACK 02 AUDIO
    PREGAP 00:02:00
    INDEX 01 01:00:00
"#;
    let layout = parse_cue(cue);
    let t = &layout.tracks[1];
    assert_eq!(t.pregap_mm, Some(0));
    assert_eq!(t.pregap_ss, Some(2));
    assert_eq!(t.pregap_ff, Some(0));
}

#[test]
fn parse_cue_offset_bin_corresponde_endereco_logico() {
    let cue = r#"FILE "game.bin" BINARY
  TRACK 01 MODE2/2352
    INDEX 01 00:00:00
  TRACK 02 AUDIO
    INDEX 01 02:00:00
"#;
    let layout = parse_cue(cue);
    let t = &layout.tracks[1];
    let offset = t.bin_offset();
    assert_eq!(offset, 2 * 60 * 75 * 2352);
}

#[test]
fn read_data_sector_extrai_2048_bytes_do_bin() {
    let cue = r#"FILE "game.bin" BINARY
  TRACK 01 MODE2/2352
    INDEX 01 00:00:00
"#;
    let layout = parse_cue(cue);
    let mut raw = vec![0u8; 2352];
    for (i, byte) in raw.iter_mut().enumerate() {
        *byte = (i & 0xFF) as u8;
    }
    let sector = layout.read_data_sector(&raw, 0);
    assert_eq!(sector.len(), 2048);
    assert_eq!(sector[0], (0x10 & 0xFF) as u8);
    assert_eq!(sector[1], (0x11 & 0xFF) as u8);
    assert_eq!(sector[2047], (0x80F & 0xFF) as u8);
}

#[test]
fn read_data_sector_no_segundo_setor() {
    let raw = vec![0xAAu8; 2352 * 5];
    let cue = r#"FILE "game.bin" BINARY
  TRACK 01 MODE2/2352
    INDEX 01 00:00:00
"#;
    let layout = parse_cue(cue);
    let sector = layout.read_data_sector(&raw, 1);
    assert_eq!(sector.len(), 2048);
    assert_eq!(sector[0], 0xAA);
}

#[test]
fn parse_cue_track_com_index00_explicito() {
    let cue = r#"FILE "game.bin" BINARY
  TRACK 01 MODE2/2352
    INDEX 01 00:00:00
  TRACK 02 AUDIO
    INDEX 00 05:00:00
    INDEX 01 05:02:00
"#;
    let layout = parse_cue(cue);
    let t = &layout.tracks[1];
    assert!(t.index00_mm.is_some());
    assert_eq!(t.index00_mm, Some(5));
    assert_eq!(t.index00_ss, Some(0));
    assert_eq!(t.index00_ff, Some(0));
}

#[test]
fn parse_cue_bin_path_com_aspas() {
    let cue = r#"FILE "path/to/game.bin" BINARY
  TRACK 01 MODE2/2352
    INDEX 01 00:00:00
"#;
    let layout = parse_cue(cue);
    assert_eq!(layout.bin_path, "path/to/game.bin");
}

#[test]
fn parse_cue_ignora_rem() {
    let cue = r#"REM Comment about the game
FILE "game.bin" BINARY
REM Some remark
  TRACK 01 MODE2/2352
    REM inline comment
    INDEX 01 00:00:00
"#;
    let layout = parse_cue(cue);
    assert_eq!(layout.tracks.len(), 1);
}
