use crate::{common_config::{LORA_125KHZ_CH0, LORA_PREAMBLE_LENGTH}, lr1121::LR1121Config};



pub const LR1121_CONFIG: LR1121Config = LR1121Config {
    spi_channel: 1, spi_speed: 16_000_000, spi_device: 0, 
    gpio_device: 4, cs: 18, 
    rst: 0, busy: 0, 
    freq: (LORA_125KHZ_CH0) as f32, bw: 125.0, sf: 0, 
    cr: 0x1, sync_word: 0, power: 22,
    dio8: 0, irq: 0,
    preamble_length: LORA_PREAMBLE_LENGTH, tcxo_voltage: 3.3, timeout: 0
}; 