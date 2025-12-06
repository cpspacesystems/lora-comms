use std::{error::Error, ffi, fmt, mem::MaybeUninit};

use crate::sx1302::{conf::{self}, ffi_loragw_hal::{self, LGW_HAL_ERROR, lgw_board_setconf, lgw_com_type_t, lgw_conf_board_s, lgw_conf_chan_lbt_s, lgw_conf_demod_s, lgw_conf_ftime_s, lgw_conf_rxif_s, lgw_conf_rxrf_s, lgw_demod_setconf, lgw_ftime_mode_t, lgw_ftime_setconf, lgw_get_temperature, lgw_pkt_rx_s, lgw_radio_type_t, lgw_receive, lgw_rssi_tcomp_s, lgw_rxif_setconf, lgw_rxrf_setconf, lgw_send, lgw_start, lgw_status, lgw_stop, lgw_tx_gain_lut_s, lgw_tx_gain_s, lgw_txgain_setconf}}; 


/////////////
/// Types ///
/////////////

/// Common application error type
pub type ErrorType = Box<dyn Error + Send + Sync>; // this is not be best pratice, but I'm too lazy for now to make something good

/// rx status codes
pub enum StatusRX {
    Unexpected,
    Unknown, 
    /// RX modem is disabled, it will ignore commands
    Off,
    /// RX modem is receiving
    On,
    /// RX is suspended while a TX is ongoing
    Suspended,
}

/////////////////
/// Functions ///
/////////////////

const RX_TX_RADIO_RF_CHAIN: u8 = 0;
const RX_RADIO_RF_CHAIN: u8 = 1;
// the configuration here is derived from git:sx1302_hal@4b42025d/libloragw/packet_forwarder/global_conf.json.sx1250.US915
// parameters set in this function will probably never need to be changed in real normal usuage
/// configures the SX1302 radio
pub fn configure(config: conf::SX1302Configuration) -> Result<(), ErrorType> {        
    // board configuration
    unsafe {
        let mut com_path: [ffi::c_char; 64] = [0; 64]; 
        let cstr = ffi::CString::new(config.device_spi_path)?;
        std::ptr::copy_nonoverlapping(cstr.as_ptr(), com_path.as_mut_ptr(), cstr.count_bytes());
        
        let mut conf = lgw_conf_board_s {
            lorawan_public: false,
            clksrc: RX_TX_RADIO_RF_CHAIN,
            full_duplex: config.comm_full_duplex,
            com_type: lgw_com_type_t::LGW_COM_SPI,
            com_path: com_path,
        };

        check(lgw_board_setconf(&mut conf as *mut lgw_conf_board_s))?; 
    }

    // demodulator configuration
    unsafe {
        let mut conf = lgw_conf_demod_s {
            multisf_datarate: ffi_loragw_hal::LGW_MULTI_SF_EN,
        };

        check(lgw_demod_setconf(&mut conf as *mut lgw_conf_demod_s))?;
    }

    // packet timestamping
    unsafe {
        let mut conf = lgw_conf_ftime_s {
            enable: config.packet_fine_timestamps,
            mode: lgw_ftime_mode_t::LGW_FTIME_MODE_ALL_SF,
        };

        check(lgw_ftime_setconf(&mut conf as *mut lgw_conf_ftime_s))?;
    }

    // RX - IF Chain + Modem configuration -- aka frequencies and bandwidths for RX Radio 1
    unsafe {
        // Max LGW_IF_CHAIN_NB chains, channels 0-7 are for Multi-SF 125khz channels, 
        // channel 8 is for set SF 125/250/500khz channels, channel 9 is for FSK trafic  

        // Multi-SF 125khz channels
        check(lgw_rxif_setconf(0, RXIFConfigBuilder::new().enable(true).rf_chain(RX_RADIO_RF_CHAIN).freq_hz(0*400000).build()))?;
        check(lgw_rxif_setconf(1, RXIFConfigBuilder::new().enable(true).rf_chain(RX_RADIO_RF_CHAIN).freq_hz(1*400000).build()))?;
        check(lgw_rxif_setconf(2, RXIFConfigBuilder::new().enable(true).rf_chain(RX_RADIO_RF_CHAIN).freq_hz(2*400000).build()))?;
        check(lgw_rxif_setconf(3, RXIFConfigBuilder::new().enable(true).rf_chain(RX_RADIO_RF_CHAIN).freq_hz(3*400000).build()))?;
        check(lgw_rxif_setconf(4, RXIFConfigBuilder::new().enable(true).rf_chain(RX_RADIO_RF_CHAIN).freq_hz(4*400000).build()))?;
        check(lgw_rxif_setconf(5, RXIFConfigBuilder::new().enable(true).rf_chain(RX_RADIO_RF_CHAIN).freq_hz(5*400000).build()))?;
        check(lgw_rxif_setconf(6, RXIFConfigBuilder::new().enable(true).rf_chain(RX_RADIO_RF_CHAIN).freq_hz(6*400000).build()))?;
        check(lgw_rxif_setconf(7, RXIFConfigBuilder::new().enable(true).rf_chain(RX_RADIO_RF_CHAIN).freq_hz(7*400000).build()))?;
        
        // Set SF Any Bandwidth channels
        check(lgw_rxif_setconf(8, RXIFConfigBuilder::new().enable(true).rf_chain(RX_RADIO_RF_CHAIN).freq_hz(10*400000).bandwidth(ffi_loragw_hal::BW_500KHZ).datarate(ffi_loragw_hal::DR_LORA_SF7).build()))?;

        // (G)FSK channel
        check(lgw_rxif_setconf(9, RXIFConfigBuilder::new().enable(false).build()))?;
    }

    // radio 0 (RX TX) configuration
    unsafe {
        let mut conf = lgw_conf_rxrf_s {
            enable: true,
            freq_hz: config.comm_base_frequency_hz,
            rssi_offset: -215.4,
            rssi_tcomp: lgw_rssi_tcomp_s { coeff_a: 0.0, coeff_b: 0.0, coeff_c: 20.41, coeff_d: 2162.56, coeff_e: 0.0 },
            r#type: lgw_radio_type_t::LGW_RADIO_TYPE_SX1250,
            tx_enable: true,
            single_input_mode: false,
        }; 
        check(lgw_rxrf_setconf(RX_TX_RADIO_RF_CHAIN, &mut conf as *mut lgw_conf_rxrf_s))?;
    }

    // radio 1 (RX) configuration
    unsafe {
        let mut conf = lgw_conf_rxrf_s {
            enable: true,
            freq_hz: config.comm_base_frequency_hz,
            rssi_offset: -215.4,
            rssi_tcomp: lgw_rssi_tcomp_s { coeff_a: 0.0, coeff_b: 0.0, coeff_c: 20.41, coeff_d: 2162.56, coeff_e: 0.0 },
            r#type: lgw_radio_type_t::LGW_RADIO_TYPE_SX1250,
            tx_enable: false,
            single_input_mode: false,
        }; 
        check(lgw_rxrf_setconf(RX_RADIO_RF_CHAIN, &mut conf as *mut lgw_conf_rxrf_s))?;
    }

    // radio 0 TX gains configuration
    unsafe {
        let mut conf = TXGainLutBuilder::new()
            .add_tx_gain(lgw_tx_gain_s { rf_power: 12, pa_gain: 0, pwr_idx: 15, dig_gain: 0, mix_gain: 5, dac_gain: 0, offset_i: 0, offset_q: 0})
            .add_tx_gain(lgw_tx_gain_s { rf_power: 13, pa_gain: 0, pwr_idx: 16, dig_gain: 0, mix_gain: 5, dac_gain: 0, offset_i: 0, offset_q: 0})
            .add_tx_gain(lgw_tx_gain_s { rf_power: 14, pa_gain: 0, pwr_idx: 17, dig_gain: 0, mix_gain: 5, dac_gain: 0, offset_i: 0, offset_q: 0})
            .add_tx_gain(lgw_tx_gain_s { rf_power: 15, pa_gain: 0, pwr_idx: 19, dig_gain: 0, mix_gain: 5, dac_gain: 0, offset_i: 0, offset_q: 0})
            .add_tx_gain(lgw_tx_gain_s { rf_power: 16, pa_gain: 0, pwr_idx: 20, dig_gain: 0, mix_gain: 5, dac_gain: 0, offset_i: 0, offset_q: 0})
            .add_tx_gain(lgw_tx_gain_s { rf_power: 17, pa_gain: 0, pwr_idx: 22, dig_gain: 0, mix_gain: 5, dac_gain: 0, offset_i: 0, offset_q: 0})
            .add_tx_gain(lgw_tx_gain_s { rf_power: 18, pa_gain: 1, pwr_idx: 1, dig_gain: 0, mix_gain: 5, dac_gain: 0, offset_i: 0, offset_q: 0})
            .add_tx_gain(lgw_tx_gain_s { rf_power: 19, pa_gain: 1, pwr_idx: 2, dig_gain: 0, mix_gain: 5, dac_gain: 0, offset_i: 0, offset_q: 0})
            .add_tx_gain(lgw_tx_gain_s { rf_power: 20, pa_gain: 1, pwr_idx: 3, dig_gain: 0, mix_gain: 5, dac_gain: 0, offset_i: 0, offset_q: 0})
            .add_tx_gain(lgw_tx_gain_s { rf_power: 21, pa_gain: 1, pwr_idx: 4, dig_gain: 0, mix_gain: 5, dac_gain: 0, offset_i: 0, offset_q: 0})
            .add_tx_gain(lgw_tx_gain_s { rf_power: 22, pa_gain: 1, pwr_idx: 5, dig_gain: 0, mix_gain: 5, dac_gain: 0, offset_i: 0, offset_q: 0})
            .add_tx_gain(lgw_tx_gain_s { rf_power: 23, pa_gain: 1, pwr_idx: 6, dig_gain: 0, mix_gain: 5, dac_gain: 0, offset_i: 0, offset_q: 0})
            .add_tx_gain(lgw_tx_gain_s { rf_power: 24, pa_gain: 1, pwr_idx: 7, dig_gain: 0, mix_gain: 5, dac_gain: 0, offset_i: 0, offset_q: 0})
            .add_tx_gain(lgw_tx_gain_s { rf_power: 25, pa_gain: 1, pwr_idx: 9, dig_gain: 0, mix_gain: 5, dac_gain: 0, offset_i: 0, offset_q: 0})
            .add_tx_gain(lgw_tx_gain_s { rf_power: 26, pa_gain: 1, pwr_idx: 11, dig_gain: 0, mix_gain: 5, dac_gain: 0, offset_i: 0, offset_q: 0})
            .add_tx_gain(lgw_tx_gain_s { rf_power: 27, pa_gain: 1, pwr_idx: 14, dig_gain: 0, mix_gain: 5, dac_gain: 0, offset_i: 0, offset_q: 0})
            .build()
        ;
        check(lgw_txgain_setconf(RX_TX_RADIO_RF_CHAIN, &mut conf as *mut lgw_tx_gain_lut_s))?;
    }

    println!("INFO SX1302: radio configuration finished");
    Ok(())
}

/// Start the SX1302 radio
pub fn start() -> Result<(), ErrorType> {
    unsafe {
        check(lgw_start())?;   
    }
    println!("INFO SX1302: Gateway susscessfully started operation.");
    Ok(())
}

/// Stop the SX1302 radio
pub fn stop() -> Result<(), ErrorType> {
    unsafe  {
        check(lgw_stop())?;
    }
    println!("INFO SX1302: Gateway susscessfully stopped operation.");
    Ok(())
}

/// try receiving packets from sx1302, only valid packets are returned
pub fn try_receive() -> Result<Vec<Vec<u8>>, ErrorType> {
    let mut holder = unsafe { MaybeUninit::<RawPacketHolder>::zeroed().assume_init() }; 
    let count = unsafe { LGWHalStatus::check_and_get(lgw_receive(RAW_PACKET_HOLDER_SIZE, &mut holder.packets as *mut lgw_pkt_rx_s))? };
    
    let mut raw_data: Vec<Vec<u8>> = Vec::with_capacity(count as usize); 
    for i in 0..count as usize {
        let packet = &holder.packets[i];
        println!("INFO SX1302: Got new packet: {:#?}", packet);
        
        if RxPacketStatus::check_status(packet.status, RxPacketStatus::OkCRC).is_err() {
            println!("WARN SX1302: last packet was bad due to CRC mismatch, skipping this packet!");
            continue;
        };

        let mut vec = Vec::with_capacity(packet.size as usize); 
        vec.copy_from_slice(&packet.payload[0..packet.size as usize]);
        raw_data.push(vec);
    }
    Ok(raw_data)
}    

// Get the RX radio status
pub fn get_status_rx() -> Result<StatusRX, ErrorType> {
    let mut code: u8 = 0;
    unsafe {
        check(lgw_status(RX_RADIO_RF_CHAIN, ffi_loragw_hal::RX_STATUS, &mut code as *mut u8))?;
    }
    match code {
        ffi_loragw_hal::RX_ON => Ok(StatusRX::On),
        ffi_loragw_hal::RX_SUSPENDED => Ok(StatusRX::Suspended),
        ffi_loragw_hal::RX_OFF => Ok(StatusRX::Off),
        ffi_loragw_hal::RX_STATUS_UNKNOWN => Ok(StatusRX::Unknown),
        _ => Ok(StatusRX::Unexpected)
    }
}

// Get the SX1302 temperature in degrees celcius
pub fn get_temperature_celcius () -> Result<f32, ErrorType> {
    let mut temp: f32 = 0.0;
    unsafe {
        check(lgw_get_temperature(&mut temp as *mut f32))?;
    }
    Ok(temp)
}


///////////////
/// Helpers ///
///////////////

#[derive(Debug, PartialEq)]
/// lgw hal c ffi return status codes
enum LGWHalStatus {
    SUCCESS,
    ERROR,
    LBTNotAllowed,
    Unknown
}
impl LGWHalStatus {
    // check if the status is Ok
    fn check(status_code: ffi::c_int) -> Result<Self, Self> {
        match status_code {
            ffi_loragw_hal::LGW_HAL_SUCCESS => Ok(LGWHalStatus::SUCCESS),
            ffi_loragw_hal::LGW_HAL_ERROR => Err(LGWHalStatus::ERROR), 
            ffi_loragw_hal::LGW_LBT_NOT_ALLOWED => Err(LGWHalStatus::LBTNotAllowed),
            _ => Err(LGWHalStatus::Unknown)
        }
    }

    // check if the status is Ok, then return what ever number the status code is
    fn check_and_get(status_code: ffi::c_int) -> Result<ffi::c_int, Self> {
        match status_code {
            ffi_loragw_hal::LGW_HAL_ERROR => Err(LGWHalStatus::ERROR),
            // assume sussccess if not error
            _ => Ok(status_code)
        }
    }
    
}
impl fmt::Display for LGWHalStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::SUCCESS => write!(f, "INFO SX1302: Gateway Success."),
            Self::ERROR | Self::LBTNotAllowed => write!(f, "ERROR SX1302: Gateway error: {}", self),
            Self::Unknown => write!(f, "ERROR SX1302: Gateway unknown error happened, application is in an undefined state!")
        }
    }
}
impl Error for LGWHalStatus {} 
#[inline(always)]
/// helper function to quickly check the status of c ffi return status codes
fn check(status_code: ffi::c_int) -> Result<LGWHalStatus,LGWHalStatus> {LGWHalStatus::check(status_code)}

/// builder to make it more ergonomic to create lgw_tx_gain_lut_s
struct TXGainLutBuilder {
    tx_gains: Vec<lgw_tx_gain_s>
}
impl TXGainLutBuilder {
    fn new() -> Self {
        Self { tx_gains: Vec::with_capacity(ffi_loragw_hal::TX_GAIN_LUT_SIZE_MAX) }
    }

    fn add_tx_gain(&mut self, gain: lgw_tx_gain_s) -> &mut Self {
        self.tx_gains.push(gain);
        self
    }

    fn build(&self) -> lgw_tx_gain_lut_s {
        let mut arr: [MaybeUninit<lgw_tx_gain_s>; ffi_loragw_hal::TX_GAIN_LUT_SIZE_MAX] = [MaybeUninit::uninit(); ffi_loragw_hal::TX_GAIN_LUT_SIZE_MAX];
        for i in 0..self.tx_gains.len() {
            arr[i].write(self.tx_gains[i]);
        } 
        if self.tx_gains.len() > ffi_loragw_hal::TX_GAIN_LUT_SIZE_MAX {
            panic!("The amount of tx gains provided of {} is grater than TX_GAIN_LUT_SIZE_MAX of {}", self.tx_gains.len(), ffi_loragw_hal::TX_GAIN_LUT_SIZE_MAX);
        }

        unsafe {
            lgw_tx_gain_lut_s { lut: std::mem::transmute(arr), size: self.tx_gains.len() as u8 }
        }
    }
} 

/// builder to make allow for partial custom configuration of lgw_conf_rxif_s, all non configured values default to 0 
struct RXIFConfigBuilder {
    config: lgw_conf_rxif_s
}
impl RXIFConfigBuilder {
    fn new() -> Self {
        Self { config: lgw_conf_rxif_s::default() }
    }

    fn enable(&mut self, v: bool) -> &mut Self { self.config.enable = v; self }
    fn implicit_hdr(&mut self, v: bool) -> &mut Self { self.config.implicit_hdr = v; self }
    fn implicit_crc_en(&mut self, v: bool) -> &mut Self { self.config.implicit_crc_en = v; self }
    fn rf_chain(&mut self, v: u8) -> &mut Self { self.config.rf_chain = v; self }
    fn bandwidth(&mut self, v: u8) -> &mut Self { self.config.bandwidth = v; self }
    fn sync_word_size(&mut self, v: u8) -> &mut Self { self.config.sync_word_size = v; self }
    fn implicit_payload_length(&mut self, v: u8) -> &mut Self { self.config.implicit_payload_length = v; self }
    fn implicit_coderate(&mut self, v: u8) -> &mut Self { self.config.implicit_coderate = v; self }
    fn freq_hz(&mut self, v: i32) -> &mut Self { self.config.freq_hz = v; self }
    fn datarate(&mut self, v: u32) -> &mut Self { self.config.datarate = v; self }
    fn sync_word(&mut self, v: u64) -> &mut Self { self.config.sync_word = v; self }
    
    
    fn build(&mut self) -> *mut lgw_conf_rxif_s {
        &mut self.config as *mut lgw_conf_rxif_s
    }
} 


/// Packet status codes
#[derive(PartialEq)]
enum RxPacketStatus {
    Unexpected,
    Undefined,
    NoCRC,
    BadCRC,
    OkCRC,
}
impl RxPacketStatus {
    /// check if the return c status code matches an expected status enum
    fn check_status(status: u8, expected: RxPacketStatus) -> Result<RxPacketStatus, RxPacketStatus>{
        match status {
            ffi_loragw_hal::STAT_UNDEFINED => if expected == RxPacketStatus::Undefined {Ok(RxPacketStatus::Undefined)} else {Err(RxPacketStatus::Undefined)}
            ffi_loragw_hal::STAT_NO_CRC => if expected == RxPacketStatus::NoCRC {Ok(RxPacketStatus::NoCRC)} else {Err(RxPacketStatus::NoCRC)}
            ffi_loragw_hal::STAT_CRC_BAD => if expected == RxPacketStatus::BadCRC {Ok(RxPacketStatus::BadCRC)} else {Err(RxPacketStatus::BadCRC)}
            ffi_loragw_hal::STAT_CRC_OK => if expected == RxPacketStatus::OkCRC {Ok(RxPacketStatus::OkCRC)} else {Err(RxPacketStatus::OkCRC)}
            _ => if expected == RxPacketStatus::Unexpected {Ok(RxPacketStatus::Unexpected)} else {Err(RxPacketStatus::Unexpected)}
        }
    }
}

const RAW_PACKET_HOLDER_SIZE: u8 = 10;
// holds up to 10 raw lora packets, type def to force C layout on rust array
#[repr(C)]
struct RawPacketHolder {
    packets: [lgw_pkt_rx_s; RAW_PACKET_HOLDER_SIZE as usize]
}

