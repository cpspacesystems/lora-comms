use crate::sx1302::conf;
use crate::common::Bandwidth;
use crate::sx1302::types::Radios;
use crate::sx1302::conf::*;
use crate::common_config::{self, BASE_FREQ};

// default configuration
// the configuration here is derived from git:sx1302_hal@4b42025d/libloragw/packet_forwarder/global_conf.json.sx1250.US915
// most parameters set in this function will probably never need to be changed in real normal usuage
pub const SX1302_CONFIG: SX1302Configuration = SX1302Configuration {
    // device section
    device_com_path: "/dev/spidev0.0",
    device_com_type: DeviceCOMType::LGW_COM_SPI,
    device_clock_source_radio: Radios::Radio1RxOnly,
    device_comm_full_duplex: false, 
    device_lorawan_public: false,

    // demodulator section
    demodulator_lora_sf_config: DemodulatorLoraSFConfig::EnableAllLoraSpreadFactors,

    // fine timestamp section
    timestamp_config: FineTimestampConfig::NoFineTimestamps,

    // Radio 0 - Rx Tx - configurtion
    radio_0_rx_tx: RadioConfig { 
        enable: true, 
        center_freq_hz: BASE_FREQ as u32, // 907.3 base frequency to avoid the more often used lower channels
        rssi_offset: -215.4, 
        rssi_temp_comp: [0.0, 0.0, 20.41, 2162.56, 0.0], 
        radio_type: RadioType::SX1250, 
        input_mode: RadioInputMode::Differential 
    },

    // Radio 1 - Rx Only - configurtion
    radio_1_rx_only: RadioConfig { enable: true,
        center_freq_hz: BASE_FREQ as u32, 
        rssi_offset: -215.4, 
        rssi_temp_comp: [0.0, 0.0, 20.41, 2162.56, 0.0], 
        radio_type: RadioType::SX1250, 
        input_mode: RadioInputMode::Differential 
    },

    // Rx Channel configuration
    rx_0_lora: RxChannelConfigBuilder::default().enable(true).rf_radio(Radios::Radio1RxOnly).freq_offset_hz(common_config::LORA_125KHZ_CH0 as i32 - BASE_FREQ as i32).build(),
    rx_1_lora: RxChannelConfigBuilder::default().enable(true).rf_radio(Radios::Radio1RxOnly).freq_offset_hz(common_config::LORA_125KHZ_CH1 as i32 - BASE_FREQ as i32).build(),
    rx_2_lora: RxChannelConfigBuilder::default().enable(true).rf_radio(Radios::Radio1RxOnly).freq_offset_hz(common_config::LORA_125KHZ_CH2 as i32 - BASE_FREQ as i32).build(),
    rx_3_lora: RxChannelConfigBuilder::default().enable(true).rf_radio(Radios::Radio1RxOnly).freq_offset_hz(common_config::LORA_125KHZ_CH3 as i32 - BASE_FREQ as i32).build(),
    rx_4_lora: RxChannelConfigBuilder::default().enable(true).rf_radio(Radios::Radio1RxOnly).freq_offset_hz(common_config::LORA_125KHZ_CH4 as i32 - BASE_FREQ as i32).build(),
    rx_5_lora: RxChannelConfigBuilder::default().enable(true).rf_radio(Radios::Radio1RxOnly).freq_offset_hz(common_config::LORA_125KHZ_CH5 as i32 - BASE_FREQ as i32).build(),
    rx_6_lora: RxChannelConfigBuilder::default().enable(true).rf_radio(Radios::Radio1RxOnly).freq_offset_hz(common_config::LORA_125KHZ_CH6 as i32 - BASE_FREQ as i32).build(),
    rx_7_lora: RxChannelConfigBuilder::default().enable(false).rf_radio(Radios::Radio1RxOnly).build(), // channels 7 & 8 hard disabled as outside of the capability range of the radios if good sepertion were to be maintained
    rx_8_lora_any_bandwidth: RxChannelConfigBuilder::default_lora_any_bandwdith().enable(false).rf_radio(Radios::Radio1RxOnly).build(),
    rx_9_fsk: RxChannelConfigBuilder::default_fsk().enable(false).rf_radio(Radios::Radio1RxOnly).build(),

    // Tx Gains configuration for radio 0
    tx_gains: TxGainsTableBuilder::new()
        .add(TxGain { rf_power: 12, pa_gain: 0, pwr_idx: 15, dig_gain: 0, mix_gain: 5, dac_gain: 0, offset_i: 0, offset_q: 0})
        .add(TxGain { rf_power: 13, pa_gain: 0, pwr_idx: 16, dig_gain: 0, mix_gain: 5, dac_gain: 0, offset_i: 0, offset_q: 0})
        .add(TxGain { rf_power: 14, pa_gain: 0, pwr_idx: 17, dig_gain: 0, mix_gain: 5, dac_gain: 0, offset_i: 0, offset_q: 0})
        .add(TxGain { rf_power: 15, pa_gain: 0, pwr_idx: 19, dig_gain: 0, mix_gain: 5, dac_gain: 0, offset_i: 0, offset_q: 0})
        .add(TxGain { rf_power: 16, pa_gain: 0, pwr_idx: 20, dig_gain: 0, mix_gain: 5, dac_gain: 0, offset_i: 0, offset_q: 0})
        .add(TxGain { rf_power: 17, pa_gain: 0, pwr_idx: 22, dig_gain: 0, mix_gain: 5, dac_gain: 0, offset_i: 0, offset_q: 0})
        .add(TxGain { rf_power: 18, pa_gain: 1, pwr_idx:  1, dig_gain: 0, mix_gain: 5, dac_gain: 0, offset_i: 0, offset_q: 0})
        .add(TxGain { rf_power: 19, pa_gain: 1, pwr_idx:  2, dig_gain: 0, mix_gain: 5, dac_gain: 0, offset_i: 0, offset_q: 0})
        .add(TxGain { rf_power: 20, pa_gain: 1, pwr_idx:  3, dig_gain: 0, mix_gain: 5, dac_gain: 0, offset_i: 0, offset_q: 0})
        .add(TxGain { rf_power: 21, pa_gain: 1, pwr_idx:  4, dig_gain: 0, mix_gain: 5, dac_gain: 0, offset_i: 0, offset_q: 0})
        .add(TxGain { rf_power: 22, pa_gain: 1, pwr_idx:  5, dig_gain: 0, mix_gain: 5, dac_gain: 0, offset_i: 0, offset_q: 0})
        .add(TxGain { rf_power: 23, pa_gain: 1, pwr_idx:  6, dig_gain: 0, mix_gain: 5, dac_gain: 0, offset_i: 0, offset_q: 0})
        .add(TxGain { rf_power: 24, pa_gain: 1, pwr_idx:  7, dig_gain: 0, mix_gain: 5, dac_gain: 0, offset_i: 0, offset_q: 0})
        .add(TxGain { rf_power: 25, pa_gain: 1, pwr_idx:  9, dig_gain: 0, mix_gain: 5, dac_gain: 0, offset_i: 0, offset_q: 0})
        .add(TxGain { rf_power: 26, pa_gain: 1, pwr_idx: 11, dig_gain: 0, mix_gain: 5, dac_gain: 0, offset_i: 0, offset_q: 0})
        .add(TxGain { rf_power: 27, pa_gain: 1, pwr_idx: 14, dig_gain: 0, mix_gain: 5, dac_gain: 0, offset_i: 0, offset_q: 0})
        // docs say max rf power is 27, but we can prob push it if we want
        // also mix_gain is set to 5 in lora_pkt_fwd.c for some reason even though the sx1250 with our sx1302 doesn't need it 
        .build()
    ,

};