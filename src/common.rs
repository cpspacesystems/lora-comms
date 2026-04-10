use std::hash::{BuildHasher, Hasher, RandomState};

use crc::{self, CRC_8_LTE, CRC_16_CMS};

use crate::{common_config::{self, PRNG_SET_SEED, PRNG_SET_SEED_ENABLED}, errors};

pub type BufferType = Vec<u8>;
pub type GPSTime = u64; 

/// lora radio channels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoraChannel {
    CH0, CH1, CH2, CH3, CH4, CH5, CH6, CH7, CH8HBW
}
// lora radio channels mapping
impl From<LoraChannel> for u32 {    
    fn from(value: LoraChannel) -> u32 {
        match value {
            LoraChannel::CH0 => common_config::LORA_125KHZ_CH0,
            LoraChannel::CH1 => common_config::LORA_125KHZ_CH1,
            LoraChannel::CH2 => common_config::LORA_125KHZ_CH2,
            LoraChannel::CH3 => common_config::LORA_125KHZ_CH3,
            LoraChannel::CH4 => common_config::LORA_125KHZ_CH4,
            LoraChannel::CH5 => common_config::LORA_125KHZ_CH5,
            LoraChannel::CH6 => common_config::LORA_125KHZ_CH6,
            LoraChannel::CH7 => common_config::LORA_125KHZ_CH7,
            LoraChannel::CH8HBW => common_config::LORA_500KHZ_CH8,
        }
    }    
}
impl TryFrom<u32> for LoraChannel {
    type Error = errors::InvalidData;
    
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            common_config::LORA_125KHZ_CH0 => Ok(LoraChannel::CH0),
            common_config::LORA_125KHZ_CH1 => Ok(LoraChannel::CH1),
            common_config::LORA_125KHZ_CH2 => Ok(LoraChannel::CH2),
            common_config::LORA_125KHZ_CH3 => Ok(LoraChannel::CH3),
            common_config::LORA_125KHZ_CH4 => Ok(LoraChannel::CH4),
            common_config::LORA_125KHZ_CH5 => Ok(LoraChannel::CH5),
            common_config::LORA_125KHZ_CH6 => Ok(LoraChannel::CH6),
            common_config::LORA_125KHZ_CH7 => Ok(LoraChannel::CH7),
            common_config::LORA_500KHZ_CH8 => Ok(LoraChannel::CH8HBW),
            n => Err(errors::InvalidData(format!("{n} is not a frequency corrosponding to any Lora Channel!")))
        }
    }    
}

/// an assert macro similar to assert!, except an Err(AssertFailure) is returned as an result instead of panic.
macro_rules! assert_no_panic {
    ($cond:expr $(,)?) => {{
        if !($cond) {
            return Err(crate::errors::AssertFailure(format!("Assertion of {} failed at {}:{}:{}", stringify!($cond), file!(), line!(), column!())).into());
        }
    }};
    ($cond:expr, $($arg:tt)+) => {{ 
        if !($cond) {
            return Err(crate::errors::AssertFailure(format!("Assertion of {} failed at {}:{}:{} with message: {}", stringify!($cond), file!(), line!(), column!(), format!($($arg)*))).into());
        }
    }};
}
pub(super) use assert_no_panic as assert_np;

/// error correction level for LoRa packets
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoraCodeRate {
    /// 4 data bits and 1 parity bits for 5 total bits
    CR1,
    /// 4 data bits and 2 parity bits for 6 total bits
    CR2,
    /// 4 data bits and 3 parity bits for 7 total bits
    CR3,
    /// 4 data bits and 4 parity bits for 8 total bits
    CR4,
}

/// bandwidth for radio channels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bandwidth {
    Low125khz,
    Mid250khz,
    High500khz,
}

/// gets a PRNG generator, using a global set seed if specificed
pub fn get_prng() -> oorandom::Rand32 {
    if PRNG_SET_SEED_ENABLED { oorandom::Rand32::new(PRNG_SET_SEED) }
    // uses os random pool to get a seed
    else { oorandom::Rand32::new(RandomState::new().build_hasher().finish()) }
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

