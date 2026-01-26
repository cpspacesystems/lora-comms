use std::{error::Error, ffi, fmt, mem::MaybeUninit};

use crate::sx1302::{SX1302, bindings_loragw_hal::{self, LGW_HAL_ERROR, LGW_HAL_SUCCESS, lgw_board_setconf, lgw_com_type_t, lgw_conf_board_s, lgw_conf_chan_lbt_s, lgw_conf_demod_s, lgw_conf_ftime_s, lgw_conf_rxif_s, lgw_conf_rxrf_s, lgw_demod_setconf, lgw_ftime_mode_t, lgw_ftime_setconf, lgw_get_temperature, lgw_pkt_rx_s, lgw_radio_type_t, lgw_receive, lgw_rssi_tcomp_s, lgw_rxif_setconf, lgw_rxrf_setconf, lgw_send, lgw_start, lgw_status, lgw_stop, lgw_tx_gain_lut_s, lgw_tx_gain_s, lgw_txgain_setconf}, conf::{self, FineTimestampConfig}}; 
use crate::sx1302::SX1302Error;

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

pub struct ImplPhysicalSX1302 {}
impl SX1302 for ImplPhysicalSX1302 {
    /// configures the SX1302 radio
    fn configure(config: conf::SX1302Configuration) -> Result<(), SX1302Error> {
        // board configuration
        let mut com_path: [ffi::c_char; 64] = [0; 64]; 
        let cstr = if let Ok(v) = ffi::CString::new(config.device_com_path) { v } else {
            return Err(SX1302Error::ConfigUnparsableCOMPath(config.device_com_path.to_string()))
        };
        if cstr.count_bytes() > std::mem::size_of_val(&com_path) {
            return Err(SX1302Error::ConfigCOMPathTooLong(config.device_com_path.to_string(), cstr.count_bytes(), std::mem::size_of_val(&com_path)));
        }
        unsafe {
            // SAFETY: cstr is guranteed to be initialized at this point, and guranteed to fit into com_path)
            std::ptr::copy_nonoverlapping(cstr.as_ptr(), com_path.as_mut_ptr(), cstr.count_bytes());
            
            let mut conf = lgw_conf_board_s {
                lorawan_public: config.device_lorawan_public,
                clksrc: config.device_clock_source_radio as u8,
                full_duplex: config.device_comm_full_duplex,
                com_type: config.device_com_type,
                com_path: com_path,
            };

            if LGW_HAL_SUCCESS != lgw_board_setconf(&mut conf as *mut lgw_conf_board_s) {
                return Err(SX1302Error::ConfigBoardSetConfError);
            } 
        }

        // demodulator configuration
        unsafe {
            let mut conf = lgw_conf_demod_s {
                multisf_datarate: match config.demodulator_lora_sf_config {
                    conf::DemodulatorLoraSFConfig::EnableAllLoraSpreadFactors => bindings_loragw_hal::LGW_MULTI_SF_EN,
                    conf::DemodulatorLoraSFConfig::CustomLoraSpreadFactors(v) => v,
                },
            };

            if LGW_HAL_SUCCESS != lgw_demod_setconf(&mut conf as *mut lgw_conf_demod_s) {
                return Err(SX1302Error::ConfigDemodSetConfError);
            };
        }

        // packet timestamping
        unsafe {
            let mut conf = lgw_conf_ftime_s {
                enable: config.timestamp_config != FineTimestampConfig::NoFineTimestamps,
                mode: match config.timestamp_config {
                    FineTimestampConfig::NoFineTimestamps => lgw_ftime_mode_t::LGW_FTIME_MODE_ALL_SF, // this doesn matter any ways if it's disabled
                    FineTimestampConfig::EnableForAll => lgw_ftime_mode_t::LGW_FTIME_MODE_ALL_SF,
                    FineTimestampConfig::HighCapacityOnly => lgw_ftime_mode_t::LGW_FTIME_MODE_HIGH_CAPACITY,
                },
            };

            if LGW_HAL_SUCCESS != lgw_ftime_setconf(&mut conf as *mut lgw_conf_ftime_s) {
                return Err(SX1302Error::ConfigFineTimestampSetConfError)
            };
        }

        // RX - IF Chain + Modem configuration -- aka frequencies and bandwidths for RX Radio 1
        unsafe {
            // Max LGW_IF_CHAIN_NB chains, channels 0-7 are for Multi-SF 125khz channels, 
            // channel 8 is for set SF 125/250/500khz channels, channel 9 is for FSK trafic  

            // Multi-SF 125khz channels
            if LGW_HAL_SUCCESS != lgw_rxif_setconf(0, &config.rx_0_lora as *const lgw_conf_rxif_s as *mut lgw_conf_rxif_s) { return Err(SX1302Error::ConfigRxIFSetConfError(0))};
            if LGW_HAL_SUCCESS != lgw_rxif_setconf(1, &config.rx_1_lora as *const lgw_conf_rxif_s as *mut lgw_conf_rxif_s) { return Err(SX1302Error::ConfigRxIFSetConfError(1))};
            if LGW_HAL_SUCCESS != lgw_rxif_setconf(2, &config.rx_2_lora as *const lgw_conf_rxif_s as *mut lgw_conf_rxif_s) { return Err(SX1302Error::ConfigRxIFSetConfError(2))};
            if LGW_HAL_SUCCESS != lgw_rxif_setconf(3, &config.rx_3_lora as *const lgw_conf_rxif_s as *mut lgw_conf_rxif_s) { return Err(SX1302Error::ConfigRxIFSetConfError(3))};
            if LGW_HAL_SUCCESS != lgw_rxif_setconf(4, &config.rx_4_lora as *const lgw_conf_rxif_s as *mut lgw_conf_rxif_s) { return Err(SX1302Error::ConfigRxIFSetConfError(4))};
            if LGW_HAL_SUCCESS != lgw_rxif_setconf(5, &config.rx_5_lora as *const lgw_conf_rxif_s as *mut lgw_conf_rxif_s) { return Err(SX1302Error::ConfigRxIFSetConfError(5))};
            if LGW_HAL_SUCCESS != lgw_rxif_setconf(6, &config.rx_6_lora as *const lgw_conf_rxif_s as *mut lgw_conf_rxif_s) { return Err(SX1302Error::ConfigRxIFSetConfError(6))};
            if LGW_HAL_SUCCESS != lgw_rxif_setconf(7, &config.rx_7_lora as *const lgw_conf_rxif_s as *mut lgw_conf_rxif_s) { return Err(SX1302Error::ConfigRxIFSetConfError(7))};
        
            // Set SF Any Bandwidth channels
            if LGW_HAL_SUCCESS != lgw_rxif_setconf(8, &config.rx_8_lora_any_bandwidth as *const lgw_conf_rxif_s as *mut lgw_conf_rxif_s) { return Err(SX1302Error::ConfigRxIFSetConfError(8));}

            // (G)FSK channel
            if LGW_HAL_SUCCESS != lgw_rxif_setconf(9, &config.rx_9_fsk as *const lgw_conf_rxif_s as *mut lgw_conf_rxif_s) { return Err(SX1302Error::ConfigRxIFSetConfError(9))};
        }

        // radio 0 (RX TX) configuration
        unsafe {
            let mut conf = lgw_conf_rxrf_s {
                enable: config.radio_0_rx_tx.enable,
                freq_hz: config.radio_0_rx_tx.center_freq_hz,
                rssi_offset: config.radio_0_rx_tx.rssi_offset,
                rssi_tcomp: lgw_rssi_tcomp_s { coeff_a: config.radio_0_rx_tx.rssi_temp_comp[0], coeff_b: config.radio_0_rx_tx.rssi_temp_comp[1], coeff_c: config.radio_0_rx_tx.rssi_temp_comp[2], coeff_d: config.radio_0_rx_tx.rssi_temp_comp[3], coeff_e: config.radio_0_rx_tx.rssi_temp_comp[4] },
                r#type: config.radio_0_rx_tx.radio_type.into(),
                tx_enable: true, //config.radio_0_rx_tx.tx_enable,
                single_input_mode: config.radio_0_rx_tx.input_mode == conf::RadioInputMode::Single,
            }; 
            if LGW_HAL_SUCCESS != lgw_rxrf_setconf(conf::Radios::Radio0RxTx as u8, &mut conf as *mut lgw_conf_rxrf_s) {
                return Err(SX1302Error::ConfigRxRFSetConfError(conf::Radios::Radio0RxTx as u8));
            };
        }

        // radio 1 (RX) configuration
        unsafe {
            let mut conf = lgw_conf_rxrf_s {
                enable: config.radio_1_rx_only.enable,
                freq_hz: config.radio_1_rx_only.center_freq_hz,
                rssi_offset: config.radio_1_rx_only.rssi_offset,
                rssi_tcomp: lgw_rssi_tcomp_s { coeff_a: config.radio_1_rx_only.rssi_temp_comp[0], coeff_b: config.radio_1_rx_only.rssi_temp_comp[1], coeff_c: config.radio_1_rx_only.rssi_temp_comp[2], coeff_d: config.radio_1_rx_only.rssi_temp_comp[3], coeff_e: config.radio_1_rx_only.rssi_temp_comp[4] },
                r#type: config.radio_1_rx_only.radio_type.into(),
                tx_enable: false, //config.radio_1_rx_only.tx_enable,
                single_input_mode: config.radio_1_rx_only.input_mode == conf::RadioInputMode::Single,
            }; 
            if LGW_HAL_SUCCESS != lgw_rxrf_setconf(conf::Radios::Radio1RxOnly as u8, &mut conf as *mut lgw_conf_rxrf_s) {
                return Err(SX1302Error::ConfigRxRFSetConfError(conf::Radios::Radio1RxOnly as u8));
            };
        }

        // radio 0 TX gains configuration
        unsafe {
            let mut conf = config.tx_gains;
        
            if LGW_HAL_SUCCESS != lgw_txgain_setconf(conf::Radios::Radio0RxTx as u8, &mut conf as *mut lgw_tx_gain_lut_s) {
                return Err(SX1302Error::ConfigTxGainSetConfError(conf::Radios::Radio0RxTx as u8));
            }
        }

        println!("INFO SX1302: radio configuration finished");
        Ok(())
    }

    /// Start the SX1302 radio
    fn start() -> Result<(), SX1302Error> {
        unsafe {
            if LGW_HAL_SUCCESS != lgw_start() {
                return Err(SX1302Error::FailedToStart);
            }
        }
        println!("INFO SX1302: Gateway susscessfully started operation.");
        Ok(())
    }

    /// Stop the SX1302 radio
    fn stop() -> Result<(), SX1302Error> {
        unsafe  {
            if LGW_HAL_SUCCESS != lgw_stop() {
                return Err(SX1302Error::FailedToStop) ;
            }
        }
        println!("INFO SX1302: Gateway susscessfully stopped operation.");
        Ok(())
    }

    /// try receiving packets from sx1302, only valid packets are returned
    fn try_receive() -> Result<Vec<Vec<u8>>, SX1302Error> {
        // SAFETY: RawPacketHolder can be zero initialized 
        let mut holder: RawPacketHolder = unsafe { MaybeUninit::zeroed().assume_init() }; 
        let count = match unsafe { lgw_receive(RAW_PACKET_HOLDER_SIZE, &mut holder.packets as *mut lgw_pkt_rx_s) } {
            LGW_HAL_ERROR => return Err(SX1302Error::TryReceiveFailed),
            v => v
        };
        
        let mut raw_data: Vec<Vec<u8>> = Vec::with_capacity(count as usize); 
        for i in 0..count as usize {
            let packet = &holder.packets[i];
            println!("INFO SX1302: Got new packet: {:#?}", packet);
            
            if RxPacketStatus::check_status(packet.status, RxPacketStatus::GoodCRC).is_err() {
                println!("WARN SX1302: last packet was bad due to CRC mismatch, skipping this packet!");
                continue;
            };

            let mut vec = Vec::with_capacity(packet.size as usize); 
            vec.copy_from_slice(&packet.payload[0..packet.size as usize]);
            raw_data.push(vec);
        }
        Ok(raw_data)
    }

    fn try_send() -> Result<(), super::SX1302Error> {
        todo!()
    }

    // Get the SX1302 temperature in degrees celcius
    fn get_temperature_celcius() -> Result<f32, super::SX1302Error> {
        let mut temp: f32 = 0.0;
        unsafe {
            if LGW_HAL_SUCCESS != lgw_get_temperature(&mut temp as *mut f32) {
                return Err(SX1302Error::TryReceiveFailed);
            };
        }
        Ok(temp)
    }
}

// Get the RX radio status
pub fn get_status_rx() -> Result<StatusRX, ErrorType> {
    unimplemented!()
    // let mut code: u8 = 0;
    // unsafe {
    //     check(lgw_status(RX_RADIO_RF_CHAIN, bindings_loragw_hal::RX_STATUS, &mut code as *mut u8))?;
    // }
    // match code {
    //     bindings_loragw_hal::RX_ON => Ok(StatusRX::On),
    //     bindings_loragw_hal::RX_SUSPENDED => Ok(StatusRX::Suspended),
    //     bindings_loragw_hal::RX_OFF => Ok(StatusRX::Off),
    //     bindings_loragw_hal::RX_STATUS_UNKNOWN => Ok(StatusRX::Unknown),
    //     _ => Ok(StatusRX::Unexpected)
    // }
}



///////////////
/// Helpers ///
///////////////

/// Packet status codes
#[derive(PartialEq)]
enum RxPacketStatus {
    Unexpected,
    Undefined,
    NoCRC,
    BadCRC,
    GoodCRC,
}
impl RxPacketStatus {
    /// check if the return c status code matches an expected status enum, okay if expected, Err if not
    fn check_status(status: u8, expected: RxPacketStatus) -> Result<RxPacketStatus, RxPacketStatus>{
        match status {
            bindings_loragw_hal::STAT_UNDEFINED => if expected == RxPacketStatus::Undefined {Ok(RxPacketStatus::Undefined)} else {Err(RxPacketStatus::Undefined)}
            bindings_loragw_hal::STAT_NO_CRC => if expected == RxPacketStatus::NoCRC {Ok(RxPacketStatus::NoCRC)} else {Err(RxPacketStatus::NoCRC)}
            bindings_loragw_hal::STAT_CRC_BAD => if expected == RxPacketStatus::BadCRC {Ok(RxPacketStatus::BadCRC)} else {Err(RxPacketStatus::BadCRC)}
            bindings_loragw_hal::STAT_CRC_OK => if expected == RxPacketStatus::GoodCRC {Ok(RxPacketStatus::GoodCRC)} else {Err(RxPacketStatus::GoodCRC)}
            _ => if expected == RxPacketStatus::Unexpected {Ok(RxPacketStatus::Unexpected)} else {Err(RxPacketStatus::Unexpected)}
        }
    }
}

const RAW_PACKET_HOLDER_SIZE: u8 = 10;
/// holds up to 10 raw lora packets, type def to force C layout on rust array
///
/// This struct can be properly initialized through zero initialization
#[repr(C)]
struct RawPacketHolder {
    packets: [lgw_pkt_rx_s; RAW_PACKET_HOLDER_SIZE as usize]
}

