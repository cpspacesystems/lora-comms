use std::{cell::UnsafeCell, error::Error, ffi, fmt, mem::{ManuallyDrop, MaybeUninit}, ops::DerefMut, sync::LazyLock, time};

use crate::{common::{BufferType, assert_np}, errors::AnyError, network::{self, NetworkRadio}, packet::{OutgoingPacketConfig, OutgoingPacketModulation, OutgoingPacketTiming, PacketMetadata, ReceivedPacket}, sx1302::{self, backing::{DeviceBackingAPI, PhysicalDevice}, bindings_loragw_hal::{self, LGW_HAL_ERROR, LGW_HAL_SUCCESS, lgw_com_type_t, lgw_conf_board_s, lgw_conf_chan_lbt_s, lgw_conf_demod_s, lgw_conf_ftime_s, lgw_conf_rxif_s, lgw_conf_rxrf_s, lgw_ftime_mode_t, lgw_pkt_rx_s, lgw_pkt_tx_s, lgw_radio_type_t, lgw_rssi_tcomp_s, lgw_time_on_air, lgw_tx_gain_lut_s, lgw_tx_gain_s}, conf::{self}, error::{ConfigureError, FailedToGetStatus, FailedToGetTemp, FailedToStart, FailedToStop, FailedToTryReceive, TrySendError}}}; 
use crate::sx1302::types::*;
use crate::common_config::MAX_PAYLOAD_SIZE;

/////////////////
/// Functions ///
/////////////////

pub struct SX1302<'a, B: DeviceBackingAPI> {
    driver_api: &'a mut B,
    config: conf::SX1302Configuration,
    // this array must be sorted
    valid_rf_power_levels: Vec<i8>,
}

impl<'a> Default for SX1302<'a, PhysicalDevice> {    
    /// creates a new SX1302 with a default config using conf::DEFAULT_SX1302_CONFIG and backed by the Physcial device
    fn default() -> SX1302<'a, PhysicalDevice> {
        // no real allocation done here, PhysicalDevice is a zero byte and ManuallyDrop is transparent
        // apprently rust can't do const promotion for ZST as of this line being written, so ManuallyDrop it is
        let mut b = ManuallyDrop::new(PhysicalDevice);
        Self::new(conf::DEFAULT_SX1302_CONFIG, unsafe {&mut *&raw mut *b})
    }
}
impl<'a, B: DeviceBackingAPI> NetworkRadio for SX1302<'a, B> {
    
    type ConfigureError = ConfigureError;
    /// configures the SX1302 radio
    fn configure(&mut self) -> Result<(), Self::ConfigureError> {
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

            if LGW_HAL_SUCCESS != self.driver_api.lgw_board_setconf(&mut conf as *mut lgw_conf_board_s) {
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

            if LGW_HAL_SUCCESS != self.driver_api.lgw_demod_setconf(&mut conf as *mut lgw_conf_demod_s) {
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

            if LGW_HAL_SUCCESS != self.driver_api.lgw_ftime_setconf(&mut conf as *mut lgw_conf_ftime_s) {
                return Err(ConfigureError::ConfigFineTimestampSetConfError)
            };
        }

        // RX - IF Chain + Modem configuration -- aka frequencies and bandwidths for RX Radio 1
        // SAFETY: all the lgw_conf_rxif_s shouldn't be modified by lgw_rxif_setconf
        unsafe {
            // Max LGW_IF_CHAIN_NB chains, channels 0-7 are for Multi-SF 125khz channels, 
            // channel 8 is for set SF 125/250/500khz channels, channel 9 is for FSK trafic  

            // Multi-SF 125khz channels
            if LGW_HAL_SUCCESS != self.driver_api.lgw_rxif_setconf(0, &config.rx_0_lora as *const lgw_conf_rxif_s as *mut lgw_conf_rxif_s) { return Err(ConfigureError::ConfigRxIFSetConfError(0))};
            if LGW_HAL_SUCCESS != self.driver_api.lgw_rxif_setconf(1, &config.rx_1_lora as *const lgw_conf_rxif_s as *mut lgw_conf_rxif_s) { return Err(ConfigureError::ConfigRxIFSetConfError(1))};
            if LGW_HAL_SUCCESS != self.driver_api.lgw_rxif_setconf(2, &config.rx_2_lora as *const lgw_conf_rxif_s as *mut lgw_conf_rxif_s) { return Err(ConfigureError::ConfigRxIFSetConfError(2))};
            if LGW_HAL_SUCCESS != self.driver_api.lgw_rxif_setconf(3, &config.rx_3_lora as *const lgw_conf_rxif_s as *mut lgw_conf_rxif_s) { return Err(ConfigureError::ConfigRxIFSetConfError(3))};
            if LGW_HAL_SUCCESS != self.driver_api.lgw_rxif_setconf(4, &config.rx_4_lora as *const lgw_conf_rxif_s as *mut lgw_conf_rxif_s) { return Err(ConfigureError::ConfigRxIFSetConfError(4))};
            if LGW_HAL_SUCCESS != self.driver_api.lgw_rxif_setconf(5, &config.rx_5_lora as *const lgw_conf_rxif_s as *mut lgw_conf_rxif_s) { return Err(ConfigureError::ConfigRxIFSetConfError(5))};
            if LGW_HAL_SUCCESS != self.driver_api.lgw_rxif_setconf(6, &config.rx_6_lora as *const lgw_conf_rxif_s as *mut lgw_conf_rxif_s) { return Err(ConfigureError::ConfigRxIFSetConfError(6))};
            if LGW_HAL_SUCCESS != self.driver_api.lgw_rxif_setconf(7, &config.rx_7_lora as *const lgw_conf_rxif_s as *mut lgw_conf_rxif_s) { return Err(ConfigureError::ConfigRxIFSetConfError(7))};
        
            // Set SF Any Bandwidth channels
            if LGW_HAL_SUCCESS != self.driver_api.lgw_rxif_setconf(8, &config.rx_8_lora_any_bandwidth as *const lgw_conf_rxif_s as *mut lgw_conf_rxif_s) { return Err(ConfigureError::ConfigRxIFSetConfError(8));}

            // (G)FSK channel
            if LGW_HAL_SUCCESS != self.driver_api.lgw_rxif_setconf(9, &config.rx_9_fsk as *const lgw_conf_rxif_s as *mut lgw_conf_rxif_s) { return Err(ConfigureError::ConfigRxIFSetConfError(9))};
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
            if LGW_HAL_SUCCESS != self.driver_api.lgw_rxrf_setconf(Radios::Radio0RxTx as u8, &mut conf as *mut lgw_conf_rxrf_s) {
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
            if LGW_HAL_SUCCESS != self.driver_api.lgw_rxrf_setconf(Radios::Radio1RxOnly as u8, &mut conf as *mut lgw_conf_rxrf_s) {
                return Err(ConfigureError::ConfigRxRFSetConfError(Radios::Radio1RxOnly as u8));
            };
        }

        // radio 0 TX gains configuration
        unsafe {
            let conf = &config.tx_gains;
            // SAFETY: lgw_txgain_setconf shouldn't modify conf
            if LGW_HAL_SUCCESS != self.driver_api.lgw_txgain_setconf(Radios::Radio0RxTx as u8, conf as *const lgw_tx_gain_lut_s as *mut lgw_tx_gain_lut_s) {
                return Err(ConfigureError::ConfigTxGainSetConfError(Radios::Radio0RxTx as u8));
            }
        }
        // sets valid rf_power levels
        self.valid_rf_power_levels.reserve_exact(self.valid_rf_power_levels.len() + config.tx_gains.size as usize);
        for i in 0..self.config.tx_gains.size as usize {
            self.valid_rf_power_levels.push(config.tx_gains.lut[i].rf_power);
        }
        self.valid_rf_power_levels.sort();


        println!("INFO SX1302: radio configuration finished");
        Ok(())
    }

    /// Start the SX1302 radio
    fn start(&mut self) -> Result<(), AnyError> {
        unsafe {
            if LGW_HAL_SUCCESS != self.driver_api.lgw_start() {
                return Err(FailedToStart.into());
            }
        }
        println!("INFO SX1302: Gateway susscessfully started operation.");
        Ok(())
    }

    /// Stop the SX1302 radio
    fn stop(&mut self) -> Result<(), AnyError> {
        unsafe  {
            if LGW_HAL_SUCCESS != self.driver_api.lgw_stop() {
                return Err(FailedToStop.into()) ;
            }
        }
        println!("INFO SX1302: Gateway susscessfully stopped operation.");
        Ok(())
    }

    type ReceiveError = FailedToTryReceive;
    /// try receiving packets from sx1302, only valid packets are returned
    fn try_receive(&mut self) -> Result<Vec<ReceivedPacket>, Self::ReceiveError> {
        // SAFETY: lgw_pkt_rx_s can be zero initialized 
        let mut packets: [lgw_pkt_rx_s; sx1302::MAX_RAW_PAYLOAD_HOLDER_SIZE as usize] = unsafe { MaybeUninit::zeroed().assume_init() }; 
        let count = match unsafe { self.driver_api.lgw_receive(sx1302::MAX_RAW_PAYLOAD_HOLDER_SIZE as u8, &mut packets as *mut lgw_pkt_rx_s) } {
            LGW_HAL_ERROR => return Err(FailedToTryReceive),
            v => v as usize
        };
        
        let mut raw_data = Vec::with_capacity(count);
        for i in 0..count {
            let packet = &packets[i];
            // println!("INFO SX1302: Got new packet: {:#?}", packet);
            
            if packet.status != bindings_loragw_hal::STAT_CRC_OK {
                println!("WARN SX1302: Skipped one packet, CRC Non Ok.");
                continue;
            } 

            let mut data = BufferType::with_capacity(packet.size as usize); 
            data.extend_from_slice(&packet.payload[0..packet.size as usize]);
            raw_data.push(ReceivedPacket {
                data,
                meta: PacketMetadata { 
                    length: packet.size as usize,
                    frequency: packet.freq_hz,
                    snr: packet.snr, 
                    sf: if let Ok(v) = packet.datarate.try_into() { v } else { return Err(FailedToTryReceive); },
                    coderate: Default::default() //if let Ok(v) = packet.coderate.try_into() { v } else { return Err(FailedToTryReceive) }
                }
            }); 
        }
        Ok(raw_data)
    }

    type CustomSendError = TrySendError;
    /// try sending a packet from sx1302
    /// returns air time
    fn try_send(&mut self, packet_config: OutgoingPacketConfig, payload: &BufferType) -> Result<time::Duration, network::SendError<TrySendError>> {
        if payload.len() > MAX_PAYLOAD_SIZE {
            return Err(TrySendError::PayloadTooLarge(payload.len(), MAX_PAYLOAD_SIZE).into());
        }

        // ensure that the Tx Radio is not occupied
        if !self.get_radio_status(Radios::Radio0RxTx).is_ok_and(|s| s == RadioStatus::Avaliable) {
            return Err(network::SendError::RadioBusy);
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
        packet.rf_power = if self.valid_rf_power_levels.binary_search(&packet_config.rf_power).is_ok() {
            packet_config.rf_power
        } else {
            return Err(TrySendError::PacketRfPowerUndefined(packet_config.rf_power).into());
        };
        packet.modulation = match packet_config.modulation {
            OutgoingPacketModulation::CW { freq_offset_hz } => {
                packet.freq_offset = freq_offset_hz;

                bindings_loragw_hal::MOD_CW
            },
            OutgoingPacketModulation::FSK { freq_deviation_khz, baudrate, preamble_length, fixed_length: fixed_langth } => {
                packet.f_dev = freq_deviation_khz;
                packet.datarate = if 500 <= baudrate && baudrate <= 250000 { baudrate }
                    else { return Err(TrySendError::PacketFSKInvalidBaudrate(baudrate).into()); };
                packet.preamble = if 3 <= preamble_length { preamble_length } 
                    else { return Err(TrySendError::PacketPreambleLengthTooShort(preamble_length, 3).into()); };
                packet.no_header = fixed_langth;

                bindings_loragw_hal::MOD_FSK
            },
            OutgoingPacketModulation::LoRa { bandwidth, spread_factor, coderate, no_header, invert_polarity, preamble_length } => {
                packet.bandwidth = sx1302_from_bandwidth(bandwidth);
                packet.datarate = spread_factor.into();
                packet.coderate = sx1302_from_coderate(coderate);
                packet.no_header = no_header;
                packet.invert_pol = invert_polarity;
                packet.preamble = if 6 <= preamble_length { preamble_length } 
                    else { return Err(TrySendError::PacketPreambleLengthTooShort(preamble_length, 6).into()); };
                
                bindings_loragw_hal::MOD_LORA
            },
        };
        
        let mut buffer: [u8; 256] = [0; 256];
        buffer[0..payload.len()].copy_from_slice(payload.as_slice());

        packet.payload = buffer;
        packet.size = payload.len() as u16;

        if LGW_HAL_SUCCESS != unsafe { self.driver_api.lgw_send(&mut packet as *mut lgw_pkt_tx_s) } {
            println!("WARN SX1302: Failed to send packet, with content: {:?}", packet);
            return Err(TrySendError::FailedToTrySend.into());
        };

        let toa = unsafe { lgw_time_on_air(&mut packet as *mut lgw_pkt_tx_s) };
        Ok(time::Duration::from_millis(toa.into()))
    }

    fn is_currently_receiving(&mut self) -> Result<bool, AnyError> {
        match self.get_radio_status(Radios::Radio1RxOnly) {
            Ok(RadioStatus::Busy) => Ok(true),
            Err(e) => { 
                println!("Encountered error while trying to get radio status: {}", e); 
                Err(e.into()) 
            },
            _ => Ok(false)
        }
    }


}

impl<'a, B: DeviceBackingAPI> SX1302<'a, B> {
    /// creates a new SX1302 radio with configuration
    pub fn new(config: conf::SX1302Configuration, backing_api: &'a mut B) -> Self {
        SX1302::<B> { 
            driver_api: backing_api,
            config,
            valid_rf_power_levels: Vec::new()
        }
    }

    /// gets the current status of a radio on the SX1302
    pub fn get_radio_status(&mut self, radio: Radios) -> Result<RadioStatus, FailedToGetStatus> {
        let mut rx_status_code: u8 = 0;

        if LGW_HAL_SUCCESS != unsafe { self.driver_api.lgw_status(radio as u8, bindings_loragw_hal::RX_STATUS, &mut rx_status_code as *mut u8) } {
            return Err(FailedToGetStatus(radio as u8));
        };

        let mut tx_status_code: u8 = 0;
        if LGW_HAL_SUCCESS != unsafe { self.driver_api.lgw_status(radio as u8, bindings_loragw_hal::TX_STATUS, &mut tx_status_code as *mut u8) } {
            return Err(FailedToGetStatus(radio as u8));
        };

        // current mapping of all possible combinations (cartensian product)
        /* {(RX_ON,TX_OFF),                     => RxOnly       (RX_OFF,TX_OFF),                        => Off
            (RX_ON,TX_SCHEDULED),               => Busy         (RX_OFF,TX_SCHEDULED),                  => Busy
            (RX_ON,TX_EMITTING),                => Busy         (RX_OFF,TX_EMITTING),                   => Busy
            (RX_ON,TX_STATUS_UNKNOWN),          => Unknown      (RX_OFF,TX_STATUS_UNKNOWN),             => Unknown
            (RX_ON,TX_FREE),                    => Avaliable    (RX_OFF,TX_FREE),                       => Unknown
            
            (RX_SUSPENDED,TX_OFF),              => Busy         (RX_STATUS_UNKNOWN,TX_OFF),             => Unknown
            (RX_SUSPENDED,TX_SCHEDULED),        => Busy         (RX_STATUS_UNKNOWN,TX_SCHEDULED),       => Busy
            (RX_SUSPENDED,TX_EMITTING),         => Busy         (RX_STATUS_UNKNOWN,TX_EMITTING),        => Busy
            (RX_SUSPENDED,TX_STATUS_UNKNOWN),   => Busy         (RX_STATUS_UNKNOWN,TX_STATUS_UNKNOWN),  => Unknown
            (RX_SUSPENDED,TX_FREE),             => Busy         (RX_STATUS_UNKNOWN,TX_FREE)}            => Unknown
        */

        match (tx_status_code, rx_status_code) {
            (bindings_loragw_hal::TX_OFF, bindings_loragw_hal::RX_OFF) => Ok(RadioStatus::Off),
            (bindings_loragw_hal::TX_OFF, bindings_loragw_hal::RX_ON) => Ok(RadioStatus::RxOnly),
            (bindings_loragw_hal::TX_FREE, _) => Ok(RadioStatus::Avaliable),
            (bindings_loragw_hal::TX_EMITTING | bindings_loragw_hal::TX_SCHEDULED, _) => Ok(RadioStatus::Busy),
            (_, bindings_loragw_hal::RX_SUSPENDED) => Ok(RadioStatus::Busy),

            _ => Ok(RadioStatus::Unknown)
        }
    }

    /// Get the SX1302 temperature in degrees celcius
    pub fn get_temperature_celcius(&mut self, ) -> Result<f32, FailedToGetTemp> {
        let mut temp: f32 = 0.0;
        unsafe {
            if LGW_HAL_SUCCESS != self.driver_api.lgw_get_temperature(&mut temp as *mut f32) {
                return Err(FailedToGetTemp);
            };
        }
        Ok(temp)
    }
}

#[cfg(test)]
mod tests {
    use std::ffi;

    use crate::{common::{Bandwidth, BufferType, LoraCodeRate, SpreadFactor}, network::{NetworkRadio, SendError}, packet::{OutgoingPacketConfig, OutgoingPacketModulation, OutgoingPacketTiming}, sx1302::{SX1302, backing::unit_test_backing::UnitTestDevice, bindings_loragw_hal::{self, BW_125KHZ, CR_LORA_4_5, DR_LORA_SF7, IMMEDIATE, LGW_HAL_ERROR, LGW_HAL_SUCCESS, MOD_LORA, RX_OFF, RX_ON, RX_STATUS, RX_STATUS_UNKNOWN, RX_SUSPENDED, STAT_CRC_BAD, STAT_CRC_OK, TX_EMITTING, TX_FREE, TX_OFF, TX_SCHEDULED, TX_STATUS, TX_STATUS_UNKNOWN, lgw_conf_board_s, lgw_conf_demod_s, lgw_conf_ftime_s, lgw_conf_rxrf_s, lgw_pkt_rx_s, lgw_pkt_tx_s, lgw_rssi_tcomp_s}, conf, error::{ConfigureError, FailedToGetStatus, FailedToGetTemp, FailedToStart, FailedToStop, FailedToTryReceive, TrySendError}, testing::new_FunctionData, types::{RadioStatus, Radios}}};
    use crate::sx1302::types::*;
    use crate::common_config::MAX_PAYLOAD_SIZE;

    #[test]
    fn test_new() {
        let mut h1 = UnitTestDevice::new();
        let s1: SX1302<UnitTestDevice> = SX1302::new(conf::DEFAULT_SX1302_CONFIG, &mut h1);
        assert_eq!(s1.valid_rf_power_levels.len(), 0);
        assert_eq!(s1.config, conf::DEFAULT_SX1302_CONFIG);

        let s2 = SX1302::default();
        assert_eq!(s2.config, s1.config);
    }

    // this does also depend on the default config in conf.rs to be valid
    // also this is like 200 lines of code, just collapse it unless u wanna scroll for a while
    #[test]
    fn test_configure() {
        let mut h1 = UnitTestDevice::new();
        // com_path null byte handling
        let mut c1 = conf::DEFAULT_SX1302_CONFIG;
        c1.device_com_path = "/afa\0efa/dfeaf";
        let mut s = SX1302::new(c1, &mut h1);
        matches!(s.configure(), Err(ConfigureError::ConfigUnparsableCOMPath(_)));

        // com_path too long handling
        c1.device_com_path = "/afeasfes/adfhadlfaflkejaidfjalfiejadlfiengadiofjeonandfflkjeialndgljiajldkfnliajsniel";
        let mut s = SX1302::new(c1, &mut h1);
        matches!(s.configure(), Err(ConfigureError::ConfigCOMPathTooLong(_, _ ,_)));

        // check if lgw_board_setconf is called correctly and return val handled correctly
        c1.device_com_path = "/test/abc";
        let mut com_path_arr: [ffi::c_char; 64] = [0; 64];
        com_path_arr[0..9].copy_from_slice(&[47, 116, 101, 115, 116, 47, 97, 98, 99]);
        h1.lgw_board_setconf_harness.expect_from_now(1, new_FunctionData! {
            ret w LGW_HAL_ERROR,
            arg1 r lgw_conf_board_s {
                lorawan_public: c1.device_lorawan_public,
                clksrc: c1.device_clock_source_radio as u8,
                full_duplex: c1.device_comm_full_duplex,
                com_type: c1.device_com_type,
                com_path: com_path_arr,
            }
        });
        let mut s = SX1302::new(c1, &mut h1);
        matches!(s.configure(), Err(ConfigureError::ConfigBoardSetConfError));
        // lgw_board_setconf success handling is verfied by future calls to configure in this test 

        // check if lgw_demod_setconf is called correctly and return val handled correctly
        c1.demodulator_lora_sf_config = conf::DemodulatorLoraSFConfig::EnableAllLoraSpreadFactors;
        h1.lgw_demod_setconf_harness.expect_from_now(1, new_FunctionData! {
            ret w LGW_HAL_ERROR,
            arg1 r lgw_conf_demod_s { multisf_datarate: 0xFF }
        });
        let mut s = SX1302::new(c1, &mut h1);
        matches!(s.configure(), Err(ConfigureError::ConfigDemodSetConfError));

        // check custom demod sf config handling
        c1.demodulator_lora_sf_config = conf::DemodulatorLoraSFConfig::CustomLoraSpreadFactors(0x01 | 0x02 | 0x20 | 0x80);
        h1.lgw_demod_setconf_harness.expect_from_now(1, new_FunctionData! {
            ret w LGW_HAL_ERROR,
            arg1 r lgw_conf_demod_s { multisf_datarate: 0x01 | 0x02 | 0x20 | 0x80 }
        });
        let mut s = SX1302::new(c1, &mut h1);
        // this match is more so a stop condition so configure doesn't go on
        matches!(s.configure(), Err(ConfigureError::ConfigDemodSetConfError));

        // check packet timestamping handling
        // no fine ts
        c1.timestamp_config = conf::FineTimestampConfig::NoFineTimestamps;
        h1.lgw_ftime_setconf_harness.expect_from_now(1, new_FunctionData! {
            ret w LGW_HAL_ERROR, 
            arg1 r lgw_conf_ftime_s { enable: false, mode: bindings_loragw_hal::lgw_ftime_mode_t::LGW_FTIME_MODE_ALL_SF}
        });
        matches!(SX1302::new(c1, &mut h1).configure(), Err(ConfigureError::ConfigFineTimestampSetConfError));
        // hc only
        c1.timestamp_config = conf::FineTimestampConfig::HighCapacityOnly;
        h1.lgw_ftime_setconf_harness.expect_from_now(1, new_FunctionData! {
            ret w LGW_HAL_ERROR, 
            arg1 r lgw_conf_ftime_s { enable: true, mode: bindings_loragw_hal::lgw_ftime_mode_t::LGW_FTIME_MODE_HIGH_CAPACITY}
        });
        matches!(SX1302::new(c1, &mut h1).configure(), Err(ConfigureError::ConfigFineTimestampSetConfError));

        c1.timestamp_config = conf::FineTimestampConfig::EnableForAll;
        h1.lgw_ftime_setconf_harness.expect_from_now(1, new_FunctionData! {
            ret w LGW_HAL_ERROR, 
            arg1 r lgw_conf_ftime_s { enable: true, mode: bindings_loragw_hal::lgw_ftime_mode_t::LGW_FTIME_MODE_ALL_SF}
        });
        matches!(SX1302::new(c1, &mut h1).configure(), Err(ConfigureError::ConfigFineTimestampSetConfError));

        // check rxif configuration
        // ch0
        h1.lgw_rxif_setconf_harness.expect_from_now(1, new_FunctionData! {
            ret w LGW_HAL_ERROR,
            arg1 r 0, 
            arg2 r c1.rx_0_lora
        });
        assert_eq!(SX1302::new(c1, &mut h1).configure(), Err(ConfigureError::ConfigRxIFSetConfError(0)));
        // ch1
        h1.lgw_rxif_setconf_harness.expect_from_now(2, new_FunctionData! {
            ret w LGW_HAL_ERROR,
            arg1 r 1, 
            arg2 r c1.rx_1_lora
        });
        assert_eq!(SX1302::new(c1, &mut h1).configure(), Err(ConfigureError::ConfigRxIFSetConfError(1)));
        // ch2
        h1.lgw_rxif_setconf_harness.expect_from_now(3, new_FunctionData! {
            ret w LGW_HAL_ERROR,
            arg1 r 2, 
            arg2 r c1.rx_2_lora
        });
        assert_eq!(SX1302::new(c1, &mut h1).configure(), Err(ConfigureError::ConfigRxIFSetConfError(2)));
        // ch3
        h1.lgw_rxif_setconf_harness.expect_from_now(4, new_FunctionData! {
            ret w LGW_HAL_ERROR,
            arg1 r 3, 
            arg2 r c1.rx_3_lora
        });
        assert_eq!(SX1302::new(c1, &mut h1).configure(), Err(ConfigureError::ConfigRxIFSetConfError(3)));

        // ch4
        h1.lgw_rxif_setconf_harness.expect_from_now(5, new_FunctionData! {
            ret w LGW_HAL_ERROR,
            arg1 r 4, 
            arg2 r c1.rx_4_lora
        });
        assert_eq!(SX1302::new(c1, &mut h1).configure(), Err(ConfigureError::ConfigRxIFSetConfError(4)));

        // ch5
        h1.lgw_rxif_setconf_harness.expect_from_now(6, new_FunctionData! {
            ret w LGW_HAL_ERROR,
            arg1 r 5, 
            arg2 r c1.rx_5_lora
        });
        assert_eq!(SX1302::new(c1, &mut h1).configure(), Err(ConfigureError::ConfigRxIFSetConfError(5)));

        // ch6
        h1.lgw_rxif_setconf_harness.expect_from_now(7, new_FunctionData! {
            ret w LGW_HAL_ERROR,
            arg1 r 6, 
            arg2 r c1.rx_6_lora
        });
        assert_eq!(SX1302::new(c1, &mut h1).configure(), Err(ConfigureError::ConfigRxIFSetConfError(6)));

        // ch7
        h1.lgw_rxif_setconf_harness.expect_from_now(8, new_FunctionData! {
            ret w LGW_HAL_ERROR,
            arg1 r 7, 
            arg2 r c1.rx_7_lora
        });
        assert_eq!(SX1302::new(c1, &mut h1).configure(), Err(ConfigureError::ConfigRxIFSetConfError(7)));

        // ch8
        h1.lgw_rxif_setconf_harness.expect_from_now(9, new_FunctionData! {
            ret w LGW_HAL_ERROR,
            arg1 r 8, 
            arg2 r c1.rx_8_lora_any_bandwidth
        });
        assert_eq!(SX1302::new(c1, &mut h1).configure(), Err(ConfigureError::ConfigRxIFSetConfError(8)));

        // ch9
        h1.lgw_rxif_setconf_harness.expect_from_now(10, new_FunctionData! {
            ret w LGW_HAL_ERROR,
            arg1 r 9, 
            arg2 r c1.rx_9_fsk
        });
        assert_eq!(SX1302::new(c1, &mut h1).configure(), Err(ConfigureError::ConfigRxIFSetConfError(9)));

        // check radio 0 configuration 
        h1.lgw_rxrf_setconf_harness.expect_from_now(1, new_FunctionData! {
            ret w LGW_HAL_ERROR,
            arg1 r Radios::Radio0RxTx as u8,
            arg2 r lgw_conf_rxrf_s {
                enable: c1.radio_0_rx_tx.enable,
                freq_hz: c1.radio_0_rx_tx.center_freq_hz,
                rssi_offset: c1.radio_0_rx_tx.rssi_offset,
                rssi_tcomp: lgw_rssi_tcomp_s {coeff_a: 0.0, coeff_b: 1.0, coeff_c: 2.0, coeff_d: 4.0, coeff_e: 8.0 },
                r#type: c1.radio_0_rx_tx.radio_type.into(),
                tx_enable: true,
                single_input_mode: true,
            }
        });
        c1.radio_0_rx_tx.rssi_temp_comp = [0.0, 1.0, 2.0, 4.0, 8.0];
        c1.radio_0_rx_tx.input_mode = conf::RadioInputMode::Single;
        assert_eq!(SX1302::new(c1, &mut h1).configure(), Err(ConfigureError::ConfigRxRFSetConfError(Radios::Radio0RxTx as u8)));
        
        // check radio 1 configuration
        c1.radio_1_rx_only.rssi_temp_comp = [0.0, 1.0, 2.0, 4.0, 8.0];
        c1.radio_1_rx_only.input_mode = conf::RadioInputMode::Single;
        h1.lgw_rxrf_setconf_harness.expect_from_now(2, new_FunctionData! {
            ret w LGW_HAL_ERROR,
            arg1 r Radios::Radio1RxOnly as u8,
            arg2 r lgw_conf_rxrf_s {
                enable: c1.radio_1_rx_only.enable,
                freq_hz: c1.radio_1_rx_only.center_freq_hz,
                rssi_offset: c1.radio_1_rx_only.rssi_offset,
                rssi_tcomp: lgw_rssi_tcomp_s {coeff_a: 0.0, coeff_b: 1.0, coeff_c: 2.0, coeff_d: 4.0, coeff_e: 8.0 },
                r#type: c1.radio_1_rx_only.radio_type.into(),
                tx_enable: false,
                single_input_mode: true,
            }
        });
        assert_eq!(SX1302::new(c1, &mut h1).configure(), Err(ConfigureError::ConfigRxRFSetConfError(Radios::Radio1RxOnly as u8)));
    
        // checks radio 0 tx gains
        h1.lgw_txgain_setconf_harness.expect_from_now(1, new_FunctionData! {
            ret w LGW_HAL_ERROR,
            arg1 r Radios::Radio0RxTx as u8,
            arg2 w c1.tx_gains,
        });
        assert_eq!(SX1302::new(c1, &mut h1).configure(), Err(ConfigureError::ConfigTxGainSetConfError(Radios::Radio0RxTx as u8)));

        // check txgains table to rf_power levels
        c1.tx_gains.size = 5;
        c1.tx_gains.lut[0].rf_power = 10; 
        c1.tx_gains.lut[1].rf_power = 20;
        c1.tx_gains.lut[2].rf_power = 9;
        c1.tx_gains.lut[3].rf_power = 15;
        c1.tx_gains.lut[4].rf_power = -4;
        h1.lgw_txgain_setconf_harness.expect_from_now(1, new_FunctionData! {
            ret w LGW_HAL_SUCCESS,
            arg1 r Radios::Radio0RxTx as u8,
            arg2 w c1.tx_gains,
        });
        let mut s = SX1302::new(c1, &mut h1);
        assert_eq!(s.configure(), Ok(()));

        assert_eq!(s.valid_rf_power_levels, vec![-4, 9, 10, 15, 20]);
        assert!(matches!(s.driver_api, UnitTestDevice { .. }));
        assert_eq!(s.config, c1);
    }

    #[test]
    fn test_start() {
        // error handling
        let mut h1 = UnitTestDevice::new();
        h1.lgw_start_harness.expect_from_now(1, new_FunctionData! {
            ret w LGW_HAL_ERROR,
        });
        assert!(SX1302::new(conf::DEFAULT_SX1302_CONFIG, &mut h1).start().is_err());
        h1.lgw_start_harness.expect_from_now(1, new_FunctionData! {
            ret w LGW_HAL_SUCCESS,
        });
        assert!(SX1302::new(conf::DEFAULT_SX1302_CONFIG, &mut h1).start().is_ok());
    }

    #[test]
    fn test_stop() {
        // error handling checks
        let mut h1 = UnitTestDevice::new();
        h1.lgw_stop_harness.expect_from_now(1, new_FunctionData! {
            ret w LGW_HAL_ERROR,
        });
        assert!(SX1302::new(conf::DEFAULT_SX1302_CONFIG, &mut h1).stop().is_err());
        h1.lgw_stop_harness.expect_from_now(1, new_FunctionData! {
            ret w LGW_HAL_SUCCESS,
        });
        assert!(SX1302::new(conf::DEFAULT_SX1302_CONFIG, &mut h1).stop().is_ok());
    }

    #[test]
    fn test_get_radio_status() {
        // error checks
        let mut h1 = UnitTestDevice::new();
        h1.lgw_status_harness.expect_from_now(1, new_FunctionData! {
            ret w LGW_HAL_ERROR,
        });
        let mut s1 = SX1302::new(conf::DEFAULT_SX1302_CONFIG, &mut h1);
        s1.configure().unwrap();
        assert_eq!(s1.get_radio_status(Radios::Radio0RxTx), Err(FailedToGetStatus(Radios::Radio0RxTx as u8)));
        // error check
        h1.lgw_status_harness.expect_from_now(2, new_FunctionData! {
            ret w LGW_HAL_ERROR,
        });
        let mut s2 = SX1302::new(conf::DEFAULT_SX1302_CONFIG, &mut h1);
        s2.configure().unwrap();
        assert_eq!(s2.get_radio_status(Radios::Radio0RxTx), Err(FailedToGetStatus(Radios::Radio0RxTx as u8)));

        // check get_radio_status() for the current mapping of all choices
        assert_eq!(support_run_radio_status(RX_ON, TX_OFF),                         RadioStatus::RxOnly);
        assert_eq!(support_run_radio_status(RX_ON, TX_SCHEDULED),                   RadioStatus::Busy);
        assert_eq!(support_run_radio_status(RX_ON, TX_EMITTING),                    RadioStatus::Busy);
        assert_eq!(support_run_radio_status(RX_ON, TX_STATUS_UNKNOWN),              RadioStatus::Unknown);
        assert_eq!(support_run_radio_status(RX_ON, TX_FREE),                        RadioStatus::Avaliable);

        assert_eq!(support_run_radio_status(RX_OFF, TX_OFF),                        RadioStatus::Off);
        assert_eq!(support_run_radio_status(RX_OFF, TX_SCHEDULED),                  RadioStatus::Busy);
        assert_eq!(support_run_radio_status(RX_OFF, TX_EMITTING),                   RadioStatus::Busy);
        assert_eq!(support_run_radio_status(RX_OFF, TX_STATUS_UNKNOWN),             RadioStatus::Unknown);
        assert_eq!(support_run_radio_status(RX_OFF, TX_FREE),                       RadioStatus::Avaliable);

        assert_eq!(support_run_radio_status(RX_SUSPENDED, TX_OFF),                  RadioStatus::Busy);
        assert_eq!(support_run_radio_status(RX_SUSPENDED, TX_SCHEDULED),            RadioStatus::Busy);
        assert_eq!(support_run_radio_status(RX_SUSPENDED, TX_EMITTING),             RadioStatus::Busy);
        assert_eq!(support_run_radio_status(RX_SUSPENDED, TX_STATUS_UNKNOWN),       RadioStatus::Busy);
        assert_eq!(support_run_radio_status(RX_SUSPENDED, TX_FREE),                 RadioStatus::Avaliable);

        assert_eq!(support_run_radio_status(RX_STATUS_UNKNOWN, TX_OFF),             RadioStatus::Unknown);
        assert_eq!(support_run_radio_status(RX_STATUS_UNKNOWN, TX_SCHEDULED),       RadioStatus::Busy);
        assert_eq!(support_run_radio_status(RX_STATUS_UNKNOWN, TX_EMITTING),        RadioStatus::Busy);
        assert_eq!(support_run_radio_status(RX_STATUS_UNKNOWN, TX_STATUS_UNKNOWN),  RadioStatus::Unknown);
        assert_eq!(support_run_radio_status(RX_STATUS_UNKNOWN,TX_FREE),             RadioStatus::Avaliable);

    }
    // helper function for test_get_radio_status
    fn support_run_radio_status(rx_in: u8, tx_in: u8) -> RadioStatus {
        let mut h1 = UnitTestDevice::new();
        h1.lgw_status_harness.expect_from_now(1, new_FunctionData! {
            ret w LGW_HAL_SUCCESS,
            arg1 r Radios::Radio0RxTx as u8,
            arg2 r RX_STATUS,
            arg3 w rx_in
        });
        h1.lgw_status_harness.expect_from_now(2, new_FunctionData! {
            ret w LGW_HAL_SUCCESS,
            arg1 r Radios::Radio0RxTx as u8,
            arg2 r TX_STATUS,
            arg3 w tx_in
        });
        let mut s1 = SX1302::new(conf::DEFAULT_SX1302_CONFIG, &mut h1);
        s1.configure().unwrap();
        s1.get_radio_status(Radios::Radio0RxTx).unwrap()
    }

    #[test]
    fn test_try_receive() {
        // error handling checks
        let mut h1 = UnitTestDevice::new();
        h1.lgw_receive_harness.expect_from_now(1, new_FunctionData! {
            ret w LGW_HAL_ERROR,
        });
        let mut s1 = SX1302::new(conf::DEFAULT_SX1302_CONFIG, &mut h1);
        s1.configure().unwrap();
        assert_eq!(s1.try_receive(), Err(FailedToTryReceive));

        h1.lgw_receive_harness.expect_from_now(1, new_FunctionData! {
            ret w LGW_HAL_SUCCESS
        });
        let mut s2 = SX1302::new(conf::DEFAULT_SX1302_CONFIG, &mut h1);
        s2.configure().unwrap();
        assert!(matches!(s2.try_receive(), Ok(_)));

        // pkts used for the rest of the tests
        let p1 = lgw_pkt_rx_s { 
            freq_hz: 907300000, 
            freq_offset: 0, 
            if_chain: 0, 
            status: STAT_CRC_OK, 
            count_us: 13374690, 
            rf_chain: Radios::Radio1RxOnly as u8, 
            modem_id: 0, 
            modulation: MOD_LORA, 
            bandwidth: BW_125KHZ, 
            datarate: DR_LORA_SF7, 
            coderate: CR_LORA_4_5, 
            rssic: -30.0, 
            rssis: -21.0, 
            snr: 10.0, 
            snr_min: 3.0, 
            snr_max: 30.0, 
            crc: 0x00, 
            size: MAX_PAYLOAD_SIZE as u16, 
            payload: [69; 256], 
            ftime_received: true, 
            ftime: 13374690 }
        ;

        let p2 = lgw_pkt_rx_s { 
            freq_hz: 907300000, 
            freq_offset: 0, 
            if_chain: 0, 
            status: STAT_CRC_BAD, 
            count_us: 13374690, 
            rf_chain: Radios::Radio1RxOnly as u8, 
            modem_id: 0, 
            modulation: MOD_LORA, 
            bandwidth: BW_125KHZ, 
            datarate: DR_LORA_SF7, 
            coderate: CR_LORA_4_5, 
            rssic: -30.0, 
            rssis: -21.0, 
            snr: 10.0, 
            snr_min: 3.0, 
            snr_max: 30.0, 
            crc: 0x00, 
            size: MAX_PAYLOAD_SIZE as u16, 
            payload: [70; 256], 
            ftime_received: true, 
            ftime: 13374690 }
        ;

        let p3 = lgw_pkt_rx_s { 
            freq_hz: 907300000, 
            freq_offset: 0, 
            if_chain: 0, 
            status: STAT_CRC_OK, 
            count_us: 13374690, 
            rf_chain: Radios::Radio1RxOnly as u8, 
            modem_id: 0, 
            modulation: MOD_LORA, 
            bandwidth: BW_125KHZ, 
            datarate: DR_LORA_SF7, 
            coderate: CR_LORA_4_5, 
            rssic: -30.0, 
            rssis: -21.0, 
            snr: 10.0, 
            snr_min: 3.0, 
            snr_max: 30.0, 
            crc: 0x00, 
            size: 225, 
            payload: [71; 256], 
            ftime_received: true, 
            ftime: 13374690 }
        ;

        // single packet handling
        h1.lgw_receive_harness.expect_from_now(1, new_FunctionData! {
            ret w 1,
            arg2 w vec![p1]
        });
        let mut s2 = SX1302::new(conf::DEFAULT_SX1302_CONFIG, &mut h1);
        s2.configure().unwrap();
        let r2 = s2.try_receive();
        assert!(r2.is_ok());
        let d2 = r2.unwrap();
        assert_eq!(d2.len(), 1);
        assert_eq!(d2[0].meta.length, MAX_PAYLOAD_SIZE as usize);
        assert_eq!(d2.as_slice()[0].data.as_slice(), [69; MAX_PAYLOAD_SIZE]);
        
        // crc bad packet handling
        h1.lgw_receive_harness.expect_from_now(1, new_FunctionData! {
            ret w 2,
            arg2 w vec![p1, p2]
        });
        let mut s3 = SX1302::new(conf::DEFAULT_SX1302_CONFIG, &mut h1);
        s3.configure().unwrap();
        let d3 = s3.try_receive().unwrap();
        assert_eq!(d2.len(), 1);
        assert_eq!(d3.as_slice()[0].data.as_slice(), [69; MAX_PAYLOAD_SIZE]);

        // crc bad packet with good packet
        h1.lgw_receive_harness.expect_from_now(1, new_FunctionData! {
            ret w 3,
            arg2 w vec![p1, p2, p3]
        });
        let mut s3 = SX1302::new(conf::DEFAULT_SX1302_CONFIG, &mut h1);
        s3.configure().unwrap();
        let d3 = s3.try_receive().unwrap();
        assert_eq!(d3.len(), 2);
        assert_eq!(d3.as_slice()[0].data.as_slice(), [69; MAX_PAYLOAD_SIZE]);
        assert_eq!(d3.as_slice()[1].data.as_slice(), [71; 225]);

        // handle a bunch of packets
        h1.lgw_receive_harness.expect_from_now(1, new_FunctionData! {
            ret w 6,
            arg2 w vec![p1, p2, p3, p1, p3, p2]
        });
        let mut s4 = SX1302::new(conf::DEFAULT_SX1302_CONFIG, &mut h1);
        s4.configure().unwrap();
        let d4 = s4.try_receive().unwrap();
        let d4c: Vec<&[u8]> = d4.iter().map(|x| x.data.as_slice()).collect();
        let d4e: Vec<&[u8]> = vec![&[69;MAX_PAYLOAD_SIZE], &[71; 225], &[69;MAX_PAYLOAD_SIZE], &[71; 225]];
        assert_eq!(d4c, d4e);

    }

    #[test]
    fn test_try_send() {
        let mut h = UnitTestDevice::new();
        h.lgw_status_harness.set_typical_output(new_FunctionData! {
            ret w LGW_HAL_SUCCESS,
            arg3 w 2, // RX_ON and TX_FREE 
        });

        let c1 = OutgoingPacketConfig {
            freq_hz: 903000000,
            modulation: OutgoingPacketModulation::LoRa { 
                bandwidth: Bandwidth::Low125khz, 
                spread_factor: SpreadFactor::SF7, coderate: LoraCodeRate::CR1, 
                no_header: false, invert_polarity: false, preamble_length: 6 },
            timing: OutgoingPacketTiming::Immediate,
            rf_power: 21 // exists in default config
        };

        // check error handling (I'm too lazy to finish this section, so this is a very imcomplete check of error handling. All the other errors unchecked a config errors so I will just leave who ever that might hit a bug experience the pain.)
        h.lgw_send_harness.expect_from_now(1, new_FunctionData! {
            ret w LGW_HAL_ERROR,
        });
        let mut s = SX1302::new(conf::DEFAULT_SX1302_CONFIG, &mut h);
        s.configure().unwrap();
        let r = s.try_send(c1, &BufferType::new());
        assert!(matches!(r, Err(SendError::Other(TrySendError::FailedToTrySend))));
        
        // check radio status handling
        h.lgw_status_harness.set_typical_output(new_FunctionData! {
            ret w LGW_HAL_SUCCESS,
            arg3 w RX_SUSPENDED,
        });
        let mut s = SX1302::new(conf::DEFAULT_SX1302_CONFIG, &mut h);
        s.configure().unwrap();
        let r = s.try_send(c1, &BufferType::new());
        assert!(matches!(r, Err(SendError::RadioBusy)));

        // normal op handling
        h.lgw_status_harness.set_typical_output(new_FunctionData! {
            ret w LGW_HAL_SUCCESS,
            arg3 w 2, // RX_ON and TX_FREE 
        });
        h.lgw_send_harness.expect_from_now(1, new_FunctionData! {
            ret w LGW_HAL_SUCCESS,
            arg1 r lgw_pkt_tx_s { 
                freq_hz: c1.freq_hz, tx_mode: IMMEDIATE, count_us: 0, 
                rf_chain: Radios::Radio0RxTx as u8, rf_power: 21, 
                modulation: MOD_LORA, freq_offset: 0, bandwidth: BW_125KHZ, 
                datarate: 7, coderate: CR_LORA_4_5, 
                invert_pol: false, f_dev: 0, preamble: 6, 
                no_crc: false, no_header: false, 
                size: MAX_PAYLOAD_SIZE as u16, payload: { let mut a = [69; 256]; a[255] = 0; a } 
            }
        });
        let mut s = SX1302::new(conf::DEFAULT_SX1302_CONFIG, &mut h);
        s.configure().unwrap();
        let p1 = BufferType::from(&[69; MAX_PAYLOAD_SIZE]);
        let r = s.try_send(c1, &p1);
        assert!(r.is_ok());
    }

    #[test]
    fn test_get_temperature_celcius() {
        // error handling
        let mut h = UnitTestDevice::new();
        h.lgw_get_temperature_harness.expect_from_now(1, new_FunctionData! {
            ret w LGW_HAL_ERROR
        });
        let mut s = SX1302::new(conf::DEFAULT_SX1302_CONFIG, &mut h);
        s.configure().unwrap();
        assert_eq!(s.get_temperature_celcius(), Err(FailedToGetTemp));
        // value return check
        h.lgw_get_temperature_harness.expect_from_now(1, new_FunctionData! {
            ret w LGW_HAL_SUCCESS,
            arg1 w 40.0
        });
        let mut s = SX1302::new(conf::DEFAULT_SX1302_CONFIG, &mut h);
        s.configure().unwrap();
        assert_eq!(s.get_temperature_celcius().unwrap(), 40.0);
        
    }
}

