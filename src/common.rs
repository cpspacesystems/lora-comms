use crc::{self, CRC_8_LTE, CRC_16_CMS};

pub type BufferType = Vec<u8>; 
pub type GPSTime = u64; 

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

