pub const HEADER_LENGTH: usize = 14;
pub const QOI_MAGIC_BYTES: [u8; 4] = [0x71, 0x6F, 0x69, 0x66]; // "qoif"
pub const ENDING_QOI_MAGIC_BYTES: [u8; 8] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01];

pub const RGB_BYTE: u8 = 0b11111110;
pub const RGBA_BYTE: u8 = 0b11111111;

pub const COMPRESSION_TAG_MASK: u8 = 0b11000000;
pub const REMAINING_DATA_MASK: u8 = 0b00111111;
pub const INDEX_TAG: u8 = 0b00000000;
pub const DIFF_TAG: u8 = 0b01000000;
pub const LUMA_TAG: u8 = 0b10000000;
pub const RUN_TAG: u8 = 0b11000000;
