use std::{error::Error, ffi, fmt, mem::MaybeUninit};

use crate::sx1302::{self, MAX_PAYLOAD_SIZE, Payload, backing::{DeviceBackingAPI, PhysicalDevice}, bindings_loragw_hal::{self, LGW_HAL_ERROR, LGW_HAL_SUCCESS, lgw_com_type_t, lgw_conf_board_s, lgw_conf_chan_lbt_s, lgw_conf_demod_s, lgw_conf_ftime_s, lgw_conf_rxif_s, lgw_conf_rxrf_s, lgw_ftime_mode_t, lgw_pkt_rx_s, lgw_pkt_tx_s, lgw_radio_type_t, lgw_rssi_tcomp_s,lgw_tx_gain_lut_s, lgw_tx_gain_s}, conf::{self}, error::{ConfigureError, FailedToGetStatus, FailedToGetTemp, FailedToStart, FailedToStop, FailedToTryReceive, TrySendError}}; 
use crate::sx1302::types::*;

/////////////////
/// Functions ///
/////////////////

pub struct SX1302<B: DeviceBackingAPI> {
    phantom_backing_api: std::marker::PhantomData<B>,
    config: conf::SX1302Configuration,
    is_configured: bool,
    // this array must be sorted
    valid_rf_power_levels: Vec<i8>,
}
impl Default for SX1302<PhysicalDevice> {
    /// creates a new SX1302 with a default config using conf::DEFAULT_SX1302_CONFIG and backed by the Physcial device
    fn default() -> SX1302<PhysicalDevice> {
        Self::new(conf::DEFAULT_SX1302_CONFIG)
    }
}
impl<B: DeviceBackingAPI> SX1302<B> {
    /// creates a new SX1302 radio with configuration
    fn new(config: conf::SX1302Configuration) -> Self {
        SX1302::<B> { 
            phantom_backing_api: std::marker::PhantomData,
            config,
            is_configured: false,
            valid_rf_power_levels: Vec::new()
        }
    }

    /// configures the SX1302 radio
    fn configure(&mut self) -> Result<(), ConfigureError> {
        let config = &self.config;

        // board configuration
        let mut com_path: [ffi::c_char; 64] = [0; 64]; 
        let cstr = if let Ok(v) = ffi::CString::new(config.device_com_path) { v } else {
            return Err(ConfigureError::ConfigUnparsableCOMPath(config.device_com_path.to_string()))
        };
        if cstr.count_bytes() > std::mem::size_of_val(&com_path) {
            return Err(ConfigureError::ConfigCOMPathTooLong(config.device_com_path.to_string(), cstr.count_bytes(), std::mem::size_of_val(&com_path)));
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

            if LGW_HAL_SUCCESS != B::lgw_board_setconf(&mut conf as *mut lgw_conf_board_s) {
                return Err(ConfigureError::ConfigBoardSetConfError);
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

            if LGW_HAL_SUCCESS != B::lgw_demod_setconf(&mut conf as *mut lgw_conf_demod_s) {
                return Err(ConfigureError::ConfigDemodSetConfError);
            };
        }

        // packet timestamping
        unsafe {
            let mut conf = lgw_conf_ftime_s {
                enable: config.timestamp_config != conf::FineTimestampConfig::NoFineTimestamps,
                mode: match config.timestamp_config {
                    conf::FineTimestampConfig::NoFineTimestamps => lgw_ftime_mode_t::LGW_FTIME_MODE_ALL_SF, // this doesn matter any ways if it's disabled
                    conf::FineTimestampConfig::EnableForAll => lgw_ftime_mode_t::LGW_FTIME_MODE_ALL_SF,
                    conf::FineTimestampConfig::HighCapacityOnly => lgw_ftime_mode_t::LGW_FTIME_MODE_HIGH_CAPACITY,
                },
            };

            if LGW_HAL_SUCCESS != B::lgw_ftime_setconf(&mut conf as *mut lgw_conf_ftime_s) {
                return Err(ConfigureError::ConfigFineTimestampSetConfError)
            };
        }

        // RX - IF Chain + Modem configuration -- aka frequencies and bandwidths for RX Radio 1
        // SAFETY: all the lgw_conf_rxif_s shouldn't be modified by lgw_rxif_setconf
        unsafe {
            // Max LGW_IF_CHAIN_NB chains, channels 0-7 are for Multi-SF 125khz channels, 
            // channel 8 is for set SF 125/250/500khz channels, channel 9 is for FSK trafic  

            // Multi-SF 125khz channels
            if LGW_HAL_SUCCESS != B::lgw_rxif_setconf(0, &config.rx_0_lora as *const lgw_conf_rxif_s as *mut lgw_conf_rxif_s) { return Err(ConfigureError::ConfigRxIFSetConfError(0))};
            if LGW_HAL_SUCCESS != B::lgw_rxif_setconf(1, &config.rx_1_lora as *const lgw_conf_rxif_s as *mut lgw_conf_rxif_s) { return Err(ConfigureError::ConfigRxIFSetConfError(1))};
            if LGW_HAL_SUCCESS != B::lgw_rxif_setconf(2, &config.rx_2_lora as *const lgw_conf_rxif_s as *mut lgw_conf_rxif_s) { return Err(ConfigureError::ConfigRxIFSetConfError(2))};
            if LGW_HAL_SUCCESS != B::lgw_rxif_setconf(3, &config.rx_3_lora as *const lgw_conf_rxif_s as *mut lgw_conf_rxif_s) { return Err(ConfigureError::ConfigRxIFSetConfError(3))};
            if LGW_HAL_SUCCESS != B::lgw_rxif_setconf(4, &config.rx_4_lora as *const lgw_conf_rxif_s as *mut lgw_conf_rxif_s) { return Err(ConfigureError::ConfigRxIFSetConfError(4))};
            if LGW_HAL_SUCCESS != B::lgw_rxif_setconf(5, &config.rx_5_lora as *const lgw_conf_rxif_s as *mut lgw_conf_rxif_s) { return Err(ConfigureError::ConfigRxIFSetConfError(5))};
            if LGW_HAL_SUCCESS != B::lgw_rxif_setconf(6, &config.rx_6_lora as *const lgw_conf_rxif_s as *mut lgw_conf_rxif_s) { return Err(ConfigureError::ConfigRxIFSetConfError(6))};
            if LGW_HAL_SUCCESS != B::lgw_rxif_setconf(7, &config.rx_7_lora as *const lgw_conf_rxif_s as *mut lgw_conf_rxif_s) { return Err(ConfigureError::ConfigRxIFSetConfError(7))};
        
            // Set SF Any Bandwidth channels
            if LGW_HAL_SUCCESS != B::lgw_rxif_setconf(8, &config.rx_8_lora_any_bandwidth as *const lgw_conf_rxif_s as *mut lgw_conf_rxif_s) { return Err(ConfigureError::ConfigRxIFSetConfError(8));}

            // (G)FSK channel
            if LGW_HAL_SUCCESS != B::lgw_rxif_setconf(9, &config.rx_9_fsk as *const lgw_conf_rxif_s as *mut lgw_conf_rxif_s) { return Err(ConfigureError::ConfigRxIFSetConfError(9))};
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
            if LGW_HAL_SUCCESS != B::lgw_rxrf_setconf(Radios::Radio0RxTx as u8, &mut conf as *mut lgw_conf_rxrf_s) {
                return Err(ConfigureError::ConfigRxRFSetConfError(Radios::Radio0RxTx as u8));
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
            if LGW_HAL_SUCCESS != B::lgw_rxrf_setconf(Radios::Radio1RxOnly as u8, &mut conf as *mut lgw_conf_rxrf_s) {
                return Err(ConfigureError::ConfigRxRFSetConfError(Radios::Radio1RxOnly as u8));
            };
        }

        // radio 0 TX gains configuration
        unsafe {
            let conf = &config.tx_gains;
            // SAFETY: lgw_txgain_setconf shouldn't modify conf
            if LGW_HAL_SUCCESS != B::lgw_txgain_setconf(Radios::Radio0RxTx as u8, conf as *const lgw_tx_gain_lut_s as *mut lgw_tx_gain_lut_s) {
                return Err(ConfigureError::ConfigTxGainSetConfError(Radios::Radio0RxTx as u8));
            }
        }
        // sets valid rf_power levels
        self.valid_rf_power_levels.reserve_exact(self.valid_rf_power_levels.len() + config.tx_gains.size as usize);
        for i in 0..self.config.tx_gains.size as usize {
            self.valid_rf_power_levels.push(config.tx_gains.lut[i].rf_power);
        }
        self.valid_rf_power_levels.sort();


        self.is_configured = true;

        println!("INFO SX1302: radio configuration finished");
        Ok(())
    }

    /// Start the SX1302 radio
    fn start(&mut self) -> Result<(), FailedToStart> {
        unsafe {
            if LGW_HAL_SUCCESS != B::lgw_start() {
                return Err(FailedToStart);
            }
        }
        println!("INFO SX1302: Gateway susscessfully started operation.");
        Ok(())
    }

    /// Stop the SX1302 radio
    fn stop(&mut self) -> Result<(), FailedToStop> {
        unsafe  {
            if LGW_HAL_SUCCESS != B::lgw_stop() {
                return Err(FailedToStop) ;
            }
        }
        println!("INFO SX1302: Gateway susscessfully stopped operation.");
        Ok(())
    }

    /// try receiving packets from sx1302, only valid packets are returned
    fn try_receive(&mut self) -> Result<FixedVec<FixedVec<u8, { sx1302::MAX_PAYLOAD_SIZE }>, { sx1302::MAX_RAW_PAYLOAD_HOLDER_SIZE }>, FailedToTryReceive> {
        // SAFETY: lgw_pkt_rx_s can be zero initialized 
        let mut packets: [lgw_pkt_rx_s; sx1302::MAX_RAW_PAYLOAD_HOLDER_SIZE as usize] = unsafe { MaybeUninit::zeroed().assume_init() }; 
        let count = match unsafe { B::lgw_receive(sx1302::MAX_RAW_PAYLOAD_HOLDER_SIZE as u8, &mut packets as *mut lgw_pkt_rx_s) } {
            LGW_HAL_ERROR => return Err(FailedToTryReceive),
            v => v
        };
        
        let mut raw_data = FixedVec::new(Payload::new(0));
        for i in 0..count as usize {
            let packet = &packets[i];
            println!("INFO SX1302: Got new packet: {:#?}", packet);
            
            if RxPacketStatus::check_status(packet.status, RxPacketStatus::GoodCRC).is_err() {
                println!("WARN SX1302: last packet was bad due to CRC mismatch, skipping this packet!");
                continue;
            };

            let mut vec = Payload::new(0); 
            if vec.concat_from_slice(&packet.payload[0..packet.size as usize]).is_err() 
                { return Err(FailedToTryReceive); }; // bascially impossible to happen, but let's not panic ever
            if raw_data.push(vec).is_err() 
                { return Err(FailedToTryReceive); }; // bascially impossible to happen, but let's not panic ever
        }
        Ok(raw_data)
    }

    /// try sending a packet from sx1302
    fn try_send(&mut self, packet_config: OutgoingPacketConfig, payload: Payload) -> Result<(), TrySendError> {
        if payload.len() > MAX_PAYLOAD_SIZE {
            return Err(TrySendError::PayloadTooLarge(payload.len(), MAX_PAYLOAD_SIZE));
        }

        // ensure that the Tx Radio is not occupied
        if !self.get_radio_status(Radios::Radio0RxTx).is_ok_and(|s| s == RadioStatus::Avaliable) {
            return Err(TrySendError::RadioBusy);
        }

        // verify packet configuration 
        // SAFETY: lgw_pkt_tx_s can be safely initialized with 0 values
        let mut packet: lgw_pkt_tx_s = unsafe { std::mem::zeroed() };
        packet.rf_chain = Radios::Radio0RxTx as u8;
        packet.freq_hz = packet_config.freq_hz;
        packet.tx_mode = match packet_config.timing {
            OutgoingPacketTiming::Immediate => bindings_loragw_hal::IMMEDIATE,
            OutgoingPacketTiming::Timestamped(us) => {
                packet.count_us = us;
                bindings_loragw_hal::TIMESTAMPED
            },
            OutgoingPacketTiming::GPSTriggered => bindings_loragw_hal::ON_GPS,
        };
        packet.rf_power = if self.valid_rf_power_levels.binary_search(&packet.rf_power).is_ok() {
            packet_config.rf_power
        } else {
            return Err(TrySendError::PacketRfPowerUndefined(packet_config.rf_power));
        };
        packet.modulation = match packet_config.modulation {
            OutgoingPacketModulation::CW { freq_offset_hz } => {
                packet.freq_offset = freq_offset_hz;

                bindings_loragw_hal::MOD_CW
            },
            OutgoingPacketModulation::FSK { freq_deviation_khz, baudrate, preamble_length, fixed_length: fixed_langth } => {
                packet.f_dev = freq_deviation_khz;
                packet.datarate = if 500 <= baudrate && baudrate <= 250000 { baudrate }
                    else { return Err(TrySendError::PacketFSKInvalidBaudrate(baudrate)); };
                packet.preamble = if 3 <= preamble_length { preamble_length } 
                    else { return Err(TrySendError::PacketPreambleLengthTooShort(preamble_length, 3)); };
                packet.no_header = fixed_langth;

                bindings_loragw_hal::MOD_FSK
            },
            OutgoingPacketModulation::LoRa { bandwidth, spread_factor, coderate, no_header, invert_polarity, preamble_length } => {
                packet.bandwidth = bandwidth as u8;
                packet.datarate = if 5 <= spread_factor && spread_factor <= 12 { spread_factor } 
                    else { return Err(TrySendError::PacketLoraSFUnsupported(spread_factor)); };
                packet.coderate = coderate as u8;
                packet.no_header = no_header;
                packet.invert_pol = invert_polarity;
                packet.preamble = if 6 <= preamble_length { preamble_length } 
                    else { return Err(TrySendError::PacketPreambleLengthTooShort(preamble_length, 6)); };
                
                bindings_loragw_hal::MOD_LORA
            },
        };
        
        let mut buffer: [u8; MAX_PAYLOAD_SIZE] = [0; 256];
        buffer[0..payload.len()].copy_from_slice(payload.as_slice());

        packet.payload = buffer;
        packet.size = payload.len() as u16;

        if LGW_HAL_SUCCESS != unsafe { B::lgw_send(&mut packet as *mut lgw_pkt_tx_s) } {
            println!("WARN SX1302: Failed to send packet, with content: {:?}", packet);
            return Err(TrySendError::FailedToTrySend);
        };

        Ok(())
    }

    /// gets the current status of a radio on the SX1302
    fn get_radio_status(&mut self, radio: Radios) -> Result<RadioStatus, FailedToGetStatus> {
        let mut rx_status_code: u8 = 0;

        if LGW_HAL_SUCCESS != unsafe { B::lgw_status(radio as u8, bindings_loragw_hal::RX_STATUS, &mut rx_status_code as *mut u8) } {
            return Err(FailedToGetStatus(radio as u8));
        };

        let mut tx_status_code: u8 = 0;
        if LGW_HAL_SUCCESS != unsafe { B::lgw_status(radio as u8, bindings_loragw_hal::TX_STATUS, &mut tx_status_code as *mut u8) } {
            return Err(FailedToGetStatus(radio as u8));
        };

        match (tx_status_code, rx_status_code) {
            (bindings_loragw_hal::TX_OFF, bindings_loragw_hal::RX_OFF) => Ok(RadioStatus::Off),
            (bindings_loragw_hal::TX_OFF, bindings_loragw_hal::RX_ON) => Ok(RadioStatus::RxOnly),
            (bindings_loragw_hal::TX_EMITTING | bindings_loragw_hal::TX_SCHEDULED, _) => Ok(RadioStatus::Busy),
            (_, bindings_loragw_hal::RX_SUSPENDED) => Ok(RadioStatus::Busy),
            (bindings_loragw_hal::TX_FREE, bindings_loragw_hal::RX_ON) => Ok(RadioStatus::Avaliable),

            _ => Ok(RadioStatus::Unknown)
        }
    }

    /// Get the SX1302 temperature in degrees celcius
    fn get_temperature_celcius(&mut self, ) -> Result<f32, FailedToGetTemp> {
        let mut temp: f32 = 0.0;
        unsafe {
            if LGW_HAL_SUCCESS != B::lgw_get_temperature(&mut temp as *mut f32) {
                return Err(FailedToGetTemp);
            };
        }
        Ok(temp)
    }
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



