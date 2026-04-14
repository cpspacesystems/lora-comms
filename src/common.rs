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
#[repr(u8)]
pub enum LoraCodeRate {
    /// 4 data bits and 1 parity bits for 5 total bits
    CR1 = 1,
    /// 4 data bits and 2 parity bits for 6 total bits
    CR2 = 2,
    /// 4 data bits and 3 parity bits for 7 total bits
    CR3 = 3,
    /// 4 data bits and 4 parity bits for 8 total bits
    CR4 = 4,
}
impl Default for LoraCodeRate {
    // default CR1
    fn default() -> Self {
        Self::CR1
    }
}
impl LoraCodeRate {
    pub const MIN: Self = LoraCodeRate::CR1;
    pub const MAX: Self = LoraCodeRate::CR4;
    pub fn increment(&self) -> Self {
        match *self {
            Self::MAX => Self::MAX,
            // SAFETY: p + 1 is guranteed to be a valid code rate, we can never overflow valid ranges
            p => unsafe { std::mem::transmute(p as u8 + 1) }
        }
    }
    pub fn decrement(&self) -> Self {
        match *self {
            Self::MIN => Self::MIN,
            // SAFETY: p - 1 is guranteed to be a valid code rate, we can never nuderflow valid ranges
            p => unsafe { std::mem::transmute(p as u8 - 1) }
        }
    }
}

/// bandwidth for radio channels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bandwidth {
    Low125khz,
    Mid250khz,
    High500khz,
}

// lora spread factor, valid only between [5,12]
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum SpreadFactor {
    SF5 = 5, SF6 = 6, SF7 = 7, SF8 = 8, SF9 = 9, SF10 = 10, SF11 = 11, SF12 = 12
}
impl Default for SpreadFactor {
    // Default SF7
    fn default() -> Self {
        Self::SF7
    }
}
impl SpreadFactor {
    pub const MIN: Self = SpreadFactor::SF5;
    pub const MAX: Self = SpreadFactor::SF12;
    pub fn increment(&self) -> Self {
        match *self {
            Self::MAX => Self::MAX,
            // SAFETY: p + 1 is guranteed to be a valid spread factor, we can never overflow valid ranges
            p => unsafe { std::mem::transmute(p as u8 + 1) }
        }
    }
    pub fn decrement(&self) -> Self {
        match *self {
            Self::MIN => Self::MIN,
            // SAFETY: p - 1 is guranteed to be a valid spread factor, we can never nuderflow valid ranges
            p => unsafe { std::mem::transmute(p as u8 - 1) }
        }
    }
}
macro_rules! __SFCVTImpl {
    ($($T:ty $(,)?)+) => {$(
        impl From<SpreadFactor> for $T {
            fn from(value: SpreadFactor) -> Self {
                (value as u8).into()
            }
        }
        impl TryFrom<$T> for SpreadFactor {
            type Error = errors::InvalidData;

            fn try_from(value: $T) -> Result<Self, Self::Error> {
                if 5 <= value && value <= 12 {
                    // SAFETY: all values are within valid ranges for SF
                    Ok(unsafe { std::mem::transmute(value as u8) })
                } else {
                    Err(errors::InvalidData(format!("Invalid data: {value} is not a valid lora spread factor!")))                    
                }
            }
        }
    )*};
}
__SFCVTImpl!(u32, u8, i32);


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

