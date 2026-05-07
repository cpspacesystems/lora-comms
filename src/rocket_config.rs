use crate::{common::LoraChannel, common_config::{LORA_125KHZ_CH0, LORA_125KHZ_CH3, LORA_PREAMBLE_LENGTH, UPLINK_CH}, lr1121::LR1121Config};



pub const LR1121_CONFIG: LR1121Config = LR1121Config {
    spi_channel: 0, spi_speed: 16_000_000, spi_device: 1, 
    gpio_device: 4, cs: 0xFFFFFFFF, 
    rst: 5, busy: 6, irq: 7,
    freq: (LORA_125KHZ_CH3 as f32)/1_000_000.0, bw: 125.0, sf: 7, 
    cr: 5, sync_word: 0x12, power: 22,
    dio8: 0xFFFFFFFF,
    preamble_length: LORA_PREAMBLE_LENGTH, tcxo_voltage: 3.3
};