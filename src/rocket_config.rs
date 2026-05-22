use crate::{common::{Bandwidth, LoraCodeRate, SpreadFactor}, common_config::{LORA_125KHZ_CH0, LORA_125KHZ_CH3, LORA_PREAMBLE_LENGTH, UPLINK_SELECTED_CH}, lr1121::{DEFAULT_LR1121_CONFIG, LR112FreqMhz, LR1121_NO_CONNECT, LR1121Config}};



pub const LR1121_CONFIG: LR1121Config = LR1121Config {
    spi_channel: 0, spi_speed_hz: 16_000_000, spi_device: 1, 
    gpio_device: 4,
    rst: 5, busy: 6, irq: 7,
    receive_freq: LR112FreqMhz::from_hz(UPLINK_SELECTED_CH), 
    receive_bw: Bandwidth::Low125khz, 
    receive_sf: SpreadFactor::SF7, 
    receive_cr: LoraCodeRate::CR1, 
    sync_word: 0x12, power: 22,
    preamble_length: LORA_PREAMBLE_LENGTH,
    ..DEFAULT_LR1121_CONFIG
};