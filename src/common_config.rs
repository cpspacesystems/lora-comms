use std::time;

use crate::{common::{LoraCodeRate}, network, network_ids, packet::OutgoingFrameBuilder};


// must match the payload arr size in loragw_hal, or smaller
// must also fit into a u8 (aka <= 255)
pub const MAX_PAYLOAD_SIZE: usize = 255;
pub const PACKER_MAX_SIZE: usize = 
    MAX_PAYLOAD_SIZE - OutgoingFrameBuilder::PACKET_FIXED_SIZE - size_of::<network_ids::TypeID>() - 3; // minus 3 is for potential capnpack worst case overhead 
// pub const PACKER_MAX_SIZE: usize = 100;

pub const BASE_FREQ: u32 = 907300000;
pub const FREQ_OFFSET: u32 = 200_000; 

pub const LORA_125KHZ_CH0: u32 = BASE_FREQ + 0 * FREQ_OFFSET; // DOWNLINK
pub const LORA_125KHZ_CH1: u32 = BASE_FREQ - 1 * FREQ_OFFSET; // UPLINK
pub const LORA_125KHZ_CH2: u32 = BASE_FREQ + 1 * FREQ_OFFSET; // DOWNLINK
pub const LORA_125KHZ_CH3: u32 = BASE_FREQ - 2 * FREQ_OFFSET; // UPLINK
pub const LORA_125KHZ_CH4: u32 = BASE_FREQ + 2 * FREQ_OFFSET; // DOWNLINK
pub const LORA_125KHZ_CH5: u32 = BASE_FREQ - 3 * FREQ_OFFSET; // UPLINK
pub const LORA_125KHZ_CH6: u32 = BASE_FREQ + 3 * FREQ_OFFSET; // DOWNLINK

pub const DOWNLINK_SELECTED_CH: u32 = LORA_125KHZ_CH0;
pub const UPLINK_SELECTED_CH: u32 = LORA_125KHZ_CH3;
pub const ALLOW_CH_CHANGE: bool = false;

pub const LORA_PREAMBLE_LENGTH: u16 = 8;

pub const INITIAL_CODE_RATE: LoraCodeRate = LoraCodeRate::CR1;

pub const PRNG_SET_SEED_ENABLED: bool = true;
pub const PRNG_SET_SEED: u64 = 133746970;

pub const LORA_REGONATION_CODE: u8 = 0b1101001;

pub const RADIO_ENABLE_UPLINK: bool = true;

pub const MAX_MAIN_LOOP_REFRESH_RATE: time::Duration = time::Duration::from_millis(360/1000);

pub const TISM_MAX_STALENESS: time::Duration = time::Duration::from_secs(5); // seconds

pub const PACKET_LOST_CALC_INTERVAL: time::Duration = time::Duration::from_secs(60); // seconds
pub const CONNECTION_LOST_AFTER_PERIOD: time::Duration = time::Duration::from_secs(10); // seconds
pub const UPLINK_TRANSMIT_BEGIN_PERIOD: time::Duration = time::Duration::from_millis(10); // ms
pub const UPLINK_TRANSMIT_TIMEOUT_PERIOD: time::Duration = time::Duration::from_millis(100); // 100 ms 

pub const SIMULATION_GROUND_ADDR: &'static str = "127.0.0.1:13374";
pub const SIMULATION_ROCKET_ADDR: &'static str = "127.0.0.1:13375";
pub const HWAS_PUBLISH_RATE: time::Duration = time::Duration::from_millis(100);