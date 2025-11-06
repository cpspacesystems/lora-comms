use crc::{self, CRC_8_LTE, CRC_16_CMS};
use crate::byte_ops::types::BufferType;

// adds section boundary
pub const DATA_BOUNDARY: u8 = 0b11011011_u8.to_le();  
pub fn add_boundary(buffer: &mut BufferType) {
    buffer.push(DATA_BOUNDARY);
}

// computes 8 bit CRC on bytes
pub fn compute_crc8(bytes: &[u8]) -> u8 {
    let crc = crc::Crc::<u8>::new(&CRC_8_LTE);
    crc.checksum(bytes)
}

// computes 16 bit CRC on bytes
pub fn compute_crc16(bytes: &[u8]) -> u16 {
    let crc = crc::Crc::<u16>::new(&CRC_16_CMS);
    crc.checksum(bytes)
}

