use std::fmt::format;

use crate::sx1302::{bindings_loragw_hal::{self}, types::{Bandwidth, Radios}};

type DeviceCOMType = bindings_loragw_hal::lgw_com_type_t;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DemodulatorLoraSFConfig {
    /// Enables all spreading factors from SF5 to SF12
    EnableAllLoraSpreadFactors,
    /// Enables only some spreading factors. Use bitwise or to create mutiple:`a | b | c`. 
    /// See DemodulatorValidLoraSF for valid lora spread factor enums.
    CustomLoraSpreadFactors(u8)
}
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DemodulatorValidLoraSF {
    SF5 = 0x01,
    SF6 = 0x02,
    SF7 = 0x04,
    SF8 = 0x08,
    SF9 = 0x10,
    SF10 = 0x20,
    SF11 = 0x40,
    SF12 = 0x80,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FineTimestampConfig {
    /// disable fine time stamps
    NoFineTimestamps,
    /// fine time stamps on all packets with spreading factors (SF5 -> 12)
    EnableForAll,
    /// fine time stamps for packets with spreadings factors of SF5 -> 10
    HighCapacityOnly,
}

pub type RxChannelConfig = bindings_loragw_hal::lgw_conf_rxif_s; 
#[derive(Debug, Clone, Copy, Default)]
/// builder to make allow for partial custom configuration of lgw_conf_rxif_s, 
/// all non configured values default to 0. This builder is designed to be used at compile time or be program static
pub struct RxChannelConfigBuilder {
    config: RxChannelConfig,
    fsk_operation: bool,
    lora_125khz_locked: bool,
    implicit_crc_set: bool,
    freq_set: bool,
}
impl RxChannelConfigBuilder {
    /// creates a new default initialized RXChannelConfigBuilder for Lora
    pub const fn default() -> Self {
        // SAFETY: All values in lgw_conf_rxis_s has a safe default of zero. 
        // MaybeUninit::zeroed also gurantees proper initialization. 
        let mut this = Self { 
            config: unsafe { std::mem::MaybeUninit::zeroed().assume_init() }, 
            implicit_crc_set: false, fsk_operation: false, 
            lora_125khz_locked: true, freq_set: false
        };
        
        // set defaults that need to be explicitly set
        this
        .rf_radio(Radios::Radio0RxTx)
        .bandwidth(Bandwidth::Low125khz)
        .lora_sf(7)
        ;

        this
    }
    /// creates a new default initialized RXChannelConfigBuilder for Lora allowing any bandwidth
    pub const fn default_lora_any_bandwdith() -> Self {
        let mut this = Self::default();
        this.lora_125khz_locked = false;
        this
    }
    /// creates a new default initialized RXChannelConfigBuilder for FSK
    pub const fn default_fsk() -> Self {
        let mut this = Self::default();
        this.fsk_operation = true;
        this.fsk_datarate(0); // datarate param is shared with lora. Unset datarate so user have to set it explicitly
        this
    }

    /// enables or disables this channel, default false --> disabled
    pub const fn enable(&mut self, v: bool) -> &mut Self { self.config.enable = v; self } 
    /// sets the channel center frequency, in amount hz offset from selected rf_radio's center frequency 
    /// MANDATORY if channel enabled   
    pub const fn freq_offset_hz(&mut self, v: i32) -> &mut Self { self.freq_set = true; self.config.freq_hz = v; self }
    /// selects which radio to use for this RX channel, default Radios::Radio0RxTx
    pub const fn rf_radio(&mut self, v: Radios) -> &mut Self { self.config.rf_chain = v as u8; self }
    /// selects the bandwidth of the channel, see Bandwidth, default Bandwidth:Low125khz 
    pub const fn bandwidth(&mut self, v: Bandwidth) -> &mut Self { self.config.bandwidth = v as u8; self }
    /// selects the lora spreading factor, valid value [5,12], default 7
    pub const fn lora_sf(&mut self, v: u32) -> &mut Self { self.config.datarate = v; self }
    
    /// lora implicit header, default false
    pub const fn lora_implicit_hdr(&mut self, v: bool) -> &mut Self { self.config.implicit_hdr = v; self }
    /// lora implicit header with implicit CRC, MANDATORY if implicit header enabled. No effect if implicit header is not enabled.
    pub const fn lora_implicit_crc_en(&mut self, v: bool) -> &mut Self { self.config.implicit_crc_en = v; self.implicit_crc_set = true; self }
    /// lora implicit header's specified payload length, MANDATORY if implicit header enabled. No effect if implicit header is not enabled.
    pub const fn implicit_payload_length(&mut self, v: u8) -> &mut Self { self.config.implicit_payload_length = v; self }
    /// lora implicit header's specificed codeing rate, MANDATORY if implicit header enabled. No effect if implicit header is not enabled.
    pub const fn implicit_coderate(&mut self, v: u8) -> &mut Self { self.config.implicit_coderate = v; self }

    /// sets the FSK datarate, MANDATORY if channel is enabled and used for FSK. 
    pub const fn fsk_datarate(&mut self, v: u32) -> &mut Self { self.config.datarate = v; self }
    /// sets the FSK sync word, Alight Right, default random
    pub const fn fsk_sync_word(&mut self, v: u64) -> &mut Self { self.config.sync_word = v; self }
    /// sets the FSK sync word size
    pub const fn fsk_sync_word_size(&mut self, v: u8) -> &mut Self { self.config.sync_word_size = v; self }

    /// builds this RxChannelBuilder into a lgw_conf_rxif_s.
    /// 
    /// !panicable -- if parameters not set correctly 
    pub const fn build(&mut self) -> RxChannelConfig {
        if self.config.enable == false {
            return self.config as RxChannelConfig;
        }
        assert!(self.freq_set == true, "Rx Channel enabled, but channel frequency offset not set!");
        
        if self.lora_125khz_locked == true {
            assert!(self.config.bandwidth == Bandwidth::Low125khz as u8, "Rx Channel not created as Lora any bandwidth, but bandwidth is set!");
        }

        // ensure fsk params are set correctly if under fsk ops
        if self.fsk_operation {
            assert!(self.config.datarate != 0, "Rx Channel for FSK enabled, but channel datarate not set!");
            assert!(self.config.sync_word == 0 && self.config.sync_word_size == 0, "Rx Channel for FSK enabled and FSK sync word set, but FSK sync word size not set!");            
        } else { // check lora SF if not fsk
            assert!(5 <= self.config.datarate && self.config.datarate <= 12, "Rx Channel's Lora spreading factor not with in the supported range of [5,12]");
        }

        // ensure implicit header parameters are set if implicit header is enabled
        if self.config.implicit_hdr == true {
            assert!(self.config.implicit_payload_length != 0, "Implicit header enabled, but implicit payload length not set (or set to invalid 0)!");
            assert!(self.implicit_crc_set, "Implicit header enabled, but implicit crc enable not set!");
            assert!(self.config.implicit_coderate != 0, "Implicit header enabled, but implicit coderate not set (or set to invalid 0)!");
        }

        self.config as RxChannelConfig
    }
} 

pub type TxGain = bindings_loragw_hal::lgw_tx_gain_s;
pub type TxGainsTable = bindings_loragw_hal::lgw_tx_gain_lut_s;
/// builder to make adding TxGain easier
pub struct TxGainsTableBuilder {
    gains: [TxGain; bindings_loragw_hal::TX_GAIN_LUT_SIZE_MAX],
    num_gains: usize
}
impl TxGainsTableBuilder {
    /// create a new TxGainsBuilder
    pub const fn new() -> Self {
        // SAFETY: All values in lgw_tx_gain_s has a safe default of zero. 
        // MaybeUninit::zeroed also gurantees proper initialization. 
        Self { gains: unsafe { std::mem::MaybeUninit::zeroed().assume_init() }, num_gains: 0 }
    }
    /// adds a TxGain into this Builder
    /// 
    /// !panicable -- if number of added TxGains exceed TX_GAIN_LUT_SIZE_MAX
    pub const fn add(&mut self, v: TxGain) -> &mut Self {
        assert!(self.num_gains <= self.gains.len(), "Can not add more TxGain. The max number of TxGain has already been added.");
        self.gains[self.num_gains] = v;
        self.num_gains += 1;
        self
    }
    /// buils the added TxGain into a lgw_tx_gain_lut_s
    pub const fn build(&mut self) -> TxGainsTable {
        TxGainsTable { 
            lut: self.gains, 
            size: self.num_gains as u8 
        }
    }
}

/// supported radio types that could be on board with SX1302
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RadioType {
    SX1255,
    SX1257,
    SX1272,
    SX1276,
    SX1250
}
impl Into<bindings_loragw_hal::lgw_radio_type_t> for RadioType {
    fn into(self) -> bindings_loragw_hal::lgw_radio_type_t {
        match self {
            RadioType::SX1255 => bindings_loragw_hal::lgw_radio_type_t::LGW_RADIO_TYPE_SX1255,
            RadioType::SX1257 => bindings_loragw_hal::lgw_radio_type_t::LGW_RADIO_TYPE_SX1257,
            RadioType::SX1272 => bindings_loragw_hal::lgw_radio_type_t::LGW_RADIO_TYPE_SX1272,
            RadioType::SX1276 => bindings_loragw_hal::lgw_radio_type_t::LGW_RADIO_TYPE_SX1276,
            RadioType::SX1250 => bindings_loragw_hal::lgw_radio_type_t::LGW_RADIO_TYPE_SX1250,
        }
    }
}
/// radio input modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RadioInputMode {
    Single,
    Differential
}
/// configuration of a radio on sx1302
#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone, Copy)]
pub struct RadioConfig {
    /// is this radio enabled
    pub enable: bool,
    // this is not allowed to be set as we already wefined which radio is rxtx and which is tx only
    /// is this radio allowed to transmit
    // pub tx_enable: bool,
    /// the center frequency for this radio in hz
    pub center_freq_hz: u32,
    /// board specific RSSI correction factor for this radio,
    pub rssi_offset: f32,
    /// board specific RSSI temperature compensation coefficients
    /// 
    /// coefficients in form of: [f32;5] = [ a, b, c, d, e ]
    pub rssi_temp_comp: [f32;5],
    /// radio type for this radio
    pub radio_type: RadioType,
    /// configure the radio in single or differential input mode, only valid for sx1302
    pub input_mode: RadioInputMode
}


/// Configuration structure for all SX1302 parameters
#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone, Copy)]
pub struct SX1302Configuration {

    /// The COMmunication interface (SPI/USB) to connect to the SX1302
    pub device_com_type: DeviceCOMType,
    /// Path to the corrosponding SPI/USB device for this SX1302
    pub device_com_path: &'static str,
    /// Which Radio to use for the clock on the SX1302
    pub device_clock_source_radio: Radios,
    /// Default disabled, Enable ONLY for public networks using the LoRa MAC protocol
    pub device_lorawan_public: bool,
    /// Weather or Full Duplex communications are enabled
    pub device_comm_full_duplex: bool,    

    /// demodulator configuration
    pub demodulator_lora_sf_config: DemodulatorLoraSFConfig,

    /// fine timestamp configuration
    pub timestamp_config: FineTimestampConfig,

    /// Radio 0 (Rx Tx) configuration
    pub radio_0_rx_tx: RadioConfig,
    /// Radio 1 (Rx only) configuration
    pub radio_1_rx_only: RadioConfig,

    /// RX Lora 125khz channel 0 configuration
    pub rx_0_lora: RxChannelConfig,
    /// RX Lora 125khz channel 1 configuration
    pub rx_1_lora: RxChannelConfig,
    /// RX Lora 125khz channel 2 configuration
    pub rx_2_lora: RxChannelConfig,
    /// RX Lora 125khz channel 3 configuration
    pub rx_3_lora: RxChannelConfig,
    /// RX Lora 125khz channel 4 configuration
    pub rx_4_lora: RxChannelConfig,
    /// RX Lora 125khz channel 5 configuration
    pub rx_5_lora: RxChannelConfig,
    /// RX Lora 125khz channel 6 configuration
    pub rx_6_lora: RxChannelConfig,
    /// RX Lora 125khz channel 7 configuration
    pub rx_7_lora: RxChannelConfig,
    /// RX Lora any bandwidth channel configuration
    pub rx_8_lora_any_bandwidth: RxChannelConfig,
    /// RX (G)FSK channel configuration
    pub rx_9_fsk: RxChannelConfig,

    /// Tx Gains configuration, must be less than 
    pub tx_gains: TxGainsTable,
}

// default configuration
// the configuration here is derived from git:sx1302_hal@4b42025d/libloragw/packet_forwarder/global_conf.json.sx1250.US915
// most parameters set in this function will probably never need to be changed in real normal usuage
pub const DEFAULT_SX1302_CONFIG: SX1302Configuration = SX1302Configuration {
    // device section
    device_com_path: "/dev/spidev0.0",
    device_com_type: DeviceCOMType::LGW_COM_SPI,
    device_clock_source_radio: Radios::Radio1RxOnly,
    device_comm_full_duplex: false, 
    device_lorawan_public: false,

    // demodulator section
    demodulator_lora_sf_config: DemodulatorLoraSFConfig::EnableAllLoraSpreadFactors,

    // fine timestamp section
    timestamp_config: FineTimestampConfig::EnableForAll,

    // Radio 0 - Rx Tx - configurtion
    radio_0_rx_tx: RadioConfig { 
        enable: true, 
        center_freq_hz: 907300000, // 907.3 base frequency to avoid the more often used lower channels
        rssi_offset: -215.4, 
        rssi_temp_comp: [0.0, 0.0, 20.41, 2162.56, 0.0], 
        radio_type: RadioType::SX1250, 
        input_mode: RadioInputMode::Differential 
    },

    // Radio 1 - Rx Only - configurtion
    radio_1_rx_only: RadioConfig { enable: true,
        center_freq_hz: 907300000, 
        rssi_offset: -215.4, 
        rssi_temp_comp: [0.0, 0.0, 20.41, 2162.56, 0.0], 
        radio_type: RadioType::SX1250, 
        input_mode: RadioInputMode::Differential 
    },

    // Rx Channel configuration
    rx_0_lora: RxChannelConfigBuilder::default().enable(true).rf_radio(Radios::Radio1RxOnly).freq_offset_hz(0*400_000).build(),
    rx_1_lora: RxChannelConfigBuilder::default().enable(true).rf_radio(Radios::Radio1RxOnly).freq_offset_hz(1*400_000).build(),
    rx_2_lora: RxChannelConfigBuilder::default().enable(true).rf_radio(Radios::Radio1RxOnly).freq_offset_hz(2*400_000).build(),
    rx_3_lora: RxChannelConfigBuilder::default().enable(true).rf_radio(Radios::Radio1RxOnly).freq_offset_hz(3*400_000).build(),
    rx_4_lora: RxChannelConfigBuilder::default().enable(true).rf_radio(Radios::Radio1RxOnly).freq_offset_hz(4*400_000).build(),
    rx_5_lora: RxChannelConfigBuilder::default().enable(true).rf_radio(Radios::Radio1RxOnly).freq_offset_hz(5*400_000).build(),
    rx_6_lora: RxChannelConfigBuilder::default().enable(true).rf_radio(Radios::Radio1RxOnly).freq_offset_hz(6*400_000).build(),
    rx_7_lora: RxChannelConfigBuilder::default().enable(true).rf_radio(Radios::Radio1RxOnly).freq_offset_hz(7*400_000).build(),
    rx_8_lora_any_bandwidth: RxChannelConfigBuilder::default_lora_any_bandwdith().enable(true).
        rf_radio(Radios::Radio1RxOnly).freq_offset_hz(10*400_400).bandwidth(Bandwidth::High500khz).lora_sf(7).build(),
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
        .build()
    ,

};

