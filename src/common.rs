use std::{cell::RefCell, hash::{BuildHasher, Hasher, RandomState}, rc::Rc};

use crc::{self, CRC_8_LTE, CRC_16_CMS};

use crate::{common_config::{self, PRNG_SET_SEED, PRNG_SET_SEED_ENABLED}, errors};

pub type BufferType = Vec<u8>;
pub type GPSTime = u64; 

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


pub trait AsRc {
    fn as_rc(self) -> Rc<RefCell<Self>> where Self: Sized {
        Rc::new(RefCell::new(self))
    }
}
impl<T> AsRc for T {}
