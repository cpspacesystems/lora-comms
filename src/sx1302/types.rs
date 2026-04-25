use core::num;
use std::array;

use crate::{common::{Bandwidth, LoraCodeRate, SpreadFactor}, errors, sx1302::{self, bindings_loragw_hal}};

//////////////////////////////////////////////////
/// All common public data types and functions ///
//////////////////////////////////////////////////


/// Radios present on the SX1302
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Radios {
    /// The radio capable of both receive and transmit
    Radio0RxTx = 0,
    /// The radio capable of only receiving
    Radio1RxOnly = 1,
}

/// Status of a radio
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RadioStatus {
    /// Radio is off, both Rx and Tx modems are stopped
    Off,
    /// Radio is on, currently listening for packets.
    /// Tx operation/packet transmit is avaliable and ready.
    Avaliable,
    /// Radio is currently trasmitting a packet. 
    /// Unable to engage in Rx operation and any additional Tx operation
    /// untill the current packet has finished transmission.   
    Busy,
    /// Radio is on, currently listening for packets,
    /// but Tx modem is off/disabled. No Tx operations avaliable!
    RxOnly,
    /// Unable to get status of the radio or radio in unknown state.
    /// 
    /// This can be due to radio disconnect, or radio not yet configured and started.
    /// Or something has gone terriably wrong and we are all gonaa die.  
    Unknown,
}

pub fn sx1302_from_bandwidth(value: Bandwidth) -> u8 {
    match value {
        Bandwidth::Low125khz => bindings_loragw_hal::BW_125KHZ,
        Bandwidth::Mid250khz => bindings_loragw_hal::BW_250KHZ,
        Bandwidth::High500khz => bindings_loragw_hal::BW_500KHZ,
        // no BW_UNDEFINED here, the default is explicitly defined here as Low125khz
    }
}

pub fn sx1302_from_coderate(value: LoraCodeRate) -> u8 {
    match value {
        LoraCodeRate::CR1 => bindings_loragw_hal::CR_LORA_4_5,
        LoraCodeRate::CR2 => bindings_loragw_hal::CR_LORA_4_6,
        LoraCodeRate::CR3 => bindings_loragw_hal::CR_LORA_4_7,
        LoraCodeRate::CR4 => bindings_loragw_hal::CR_LORA_4_8,
    }
}

pub fn sx1302_to_coderate(value: u8) -> Result<LoraCodeRate, errors::InvalidData> {
    match value {
        bindings_loragw_hal::CR_LORA_4_5 => Ok(LoraCodeRate::CR1),
        bindings_loragw_hal::CR_LORA_4_6 => Ok(LoraCodeRate::CR2),
        bindings_loragw_hal::CR_LORA_4_7 => Ok(LoraCodeRate::CR3),
        bindings_loragw_hal::CR_LORA_4_8 => Ok(LoraCodeRate::CR4),
        _ => Err(errors::InvalidData(format!("Invalid data: {value} is not a valid SX1302 lora code rate")))
    }
}
