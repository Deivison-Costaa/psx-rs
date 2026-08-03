pub const RAW_SECTOR_BYTES: usize = 2352;
pub const CDDA_FRAMES: usize = RAW_SECTOR_BYTES / 4;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct XaState {
    pub old_left: i32,
    pub older_left: i32,
    pub old_right: i32,
    pub older_right: i32,
}

pub fn decode_28_nibbles(
    _src: &[u8],
    _blk: usize,
    _nibble: usize,
    _old: i32,
    _older: i32,
) -> ([i16; 28], i32, i32) {
    ([0i16; 28], 0, 0)
}

pub fn decode_sector(_src: &[u8], _stereo: bool, _state: &mut XaState) -> Vec<(i16, i16)> {
    Vec::new()
}

pub fn cdda_frames(_raw: &[u8]) -> Vec<(i16, i16)> {
    Vec::new()
}
