use std::ffi;

use crate::sx1302::bindings_loragw_hal::{
    self, LGW_SPECTRAL_SCAN_RESULT_SIZE, lgw_conf_board_s, lgw_conf_debug_s, lgw_conf_demod_s,
    lgw_conf_ftime_s, lgw_conf_rxif_s, lgw_conf_rxrf_s, lgw_conf_sx1261_s, lgw_pkt_rx_s,
    lgw_pkt_tx_s, lgw_spectral_scan_status_t, lgw_tx_gain_lut_s,
};

/// The common backing trait to allow for deterministic and hardware detached tests
///
/// Some functions are default unimplenmented if they are not used just to reduce the amount
/// of things that the sim/test backend needs to implenment. If you need to use some function,
/// remove the default unimplenmented block and implenment the apporiate functions in all impls
#[allow(unused_variables)]
pub trait DeviceBackingAPI {
    /**
    @brief Configure the gateway board
    @param conf structure containing the configuration parameters
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_board_setconf(&mut self, conf: *mut lgw_conf_board_s) -> ffi::c_int;

    /**
    @brief Configure an RF chain (must configure before start)
    @param rf_chain number of the RF chain to configure [0, LGW_RF_CHAIN_NB - 1]
    @param conf structure containing the configuration parameters
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_rxrf_setconf(&mut self, rf_chain: u8, conf: *mut lgw_conf_rxrf_s) -> ffi::c_int;

    /**
    @brief Configure an IF chain + modem (must configure before start)
    @param if_chain number of the IF chain + modem to configure [0, LGW_IF_CHAIN_NB - 1]
    @param conf structure containing the configuration parameters
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_rxif_setconf(&mut self, if_chain: u8, conf: *mut lgw_conf_rxif_s) -> ffi::c_int;

    /**
    @brief Configure LoRa/FSK demodulators
    @param conf structure containing the configuration parameters
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_demod_setconf(&mut self, conf: *mut lgw_conf_demod_s) -> ffi::c_int;

    /**
    @brief Configure the Tx gain LUT
    @param conf pointer to structure defining the LUT
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_txgain_setconf(&mut self, rf_chain: u8, conf: *mut lgw_tx_gain_lut_s) -> ffi::c_int;

    /**
    @brief Configure the fine timestamping
    @param conf pointer to structure defining the config to be applied
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_ftime_setconf(&mut self, conf: *mut lgw_conf_ftime_s) -> ffi::c_int;

    /*
    @brief Configure the SX1261 radio for LBT/Spectral Scan
    @param pointer to structure defining the config to be applied
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_sx1261_setconf(&mut self, conf: *mut lgw_conf_sx1261_s) -> ffi::c_int {
        unimplemented!()
    }

    /**
    @brief Configure the debug context
    @param conf pointer to structure defining the config to be applied
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_debug_setconf(&mut self, conf: *mut lgw_conf_debug_s) -> ffi::c_int {
        unimplemented!()
    }

    /**
    @brief Connect to the LoRa concentrator, reset it and configure it according to previously set parameters
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_start(&mut self, ) -> ffi::c_int;

    /**
    @brief Stop the LoRa concentrator and disconnect it
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_stop(&mut self, ) -> ffi::c_int;

    /**
    @brief A non-blocking function that will fetch up to 'max_pkt' packets from the LoRa concentrator FIFO and data buffer
    @param max_pkt maximum number of packet that must be retrieved (equal to the size of the array of struct)
    @param pkt_data pointer to an array of struct that will receive the packet metadata and payload pointers
    @return LGW_HAL_ERROR id the operation failed, else the number of packets retrieved
    */
    unsafe fn lgw_receive(&mut self, max_pkt: u8, pkt_data: *mut lgw_pkt_rx_s) -> ffi::c_int;

    /**
    @brief Schedule a packet to be send immediately or after a delay depending on tx_mode
    @param pkt_data structure containing the data and metadata for the packet to send
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else

    /!\ When sending a packet, there is a delay (approx 1.5ms) for the analog
    circuitry to start and be stable. This delay is adjusted by the HAL depending
    on the board version (lgw_i_tx_start_delay_us).

    In 'timestamp' mode, this is transparent: the modem is started
    lgw_i_tx_start_delay_us microseconds before the user-set timestamp value is
    reached, the preamble of the packet start right when the internal timestamp
    counter reach target value.

    In 'immediate' mode, the packet is emitted as soon as possible: transferring the
    packet (and its parameters) from the host to the concentrator takes some time,
    then there is the lgw_i_tx_start_delay_us, then the packet is emitted.

    In 'triggered' mode (aka PPS/GPS mode), the packet, typically a beacon, is
    emitted lgw_i_tx_start_delay_us microsenconds after a rising edge of the
    trigger signal. Because there is no way to anticipate the triggering event and
    start the analog circuitry beforehand, that delay must be taken into account in
    the protocol.
    */
    unsafe fn lgw_send(&mut self, pkt_data: *mut lgw_pkt_tx_s) -> ffi::c_int;

    /**
    @brief Give the the status of different part of the LoRa concentrator
    @param select is used to select what status we want to know
    @param code is used to return the status code
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_status(&mut self, rf_chain: u8, select: u8, code: *mut u8) -> ffi::c_int;

    /**
    @brief Abort a currently scheduled or ongoing TX
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_abort_tx(&mut self, rf_chain: u8) -> ffi::c_int {
        unimplemented!()
    }

    /**
    @brief Return value of internal counter when latest event (eg GPS pulse) was captured
    @param trig_cnt_us pointer to receive timestamp value
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_get_trigcnt(&mut self, trig_cnt_us: *mut u32) -> ffi::c_int {
        unimplemented!()
    }

    /**
    @brief Return instateneous value of internal counter
    @param inst_cnt_us pointer to receive timestamp value
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_get_instcnt(&mut self, inst_cnt_us: *mut u32) -> ffi::c_int {
        unimplemented!()
    }

    /**
    @brief Return the LoRa concentrator EUI (Extended Unique Identifier)
    @param eui pointer to receive eui
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_get_eui(&mut self, eui: *mut u64) -> ffi::c_int;

    /**
    @brief Return the temperature measured by the LoRa concentrator sensor
    @param temperature The temperature measured, in degree celcius
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_get_temperature(&mut self, temperature: *mut ffi::c_float) -> ffi::c_int;

    /**
    @brief Allow user to check the version/options of the library once compiled
    @return pointer on a human-readable null terminated string
    */
    unsafe fn lgw_version_info(&mut self, ) -> *const ffi::c_char;

    /**
    @brief Return time on air of given packet, in milliseconds
    @param packet is a pointer to the packet structure
    @return the packet time on air in milliseconds
    */
    unsafe fn lgw_time_on_air(&mut self, packet: *const lgw_pkt_tx_s) -> u32 {
        unimplemented!()
    }

    /**
    @brief Start scaning the channel centered on the given frequency
    @param freq_hz channel center frequency
    @param nb_scan number of measures to be done for the scan
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_spectral_scan_start(&mut self, freq_hz: u32, nb_scan: u16) -> ffi::c_int {
        unimplemented!()
    }

    /**
    @brief Get the current scan status
    @param status a pointer to the returned status
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_spectral_scan_get_status(&mut self, status: *mut lgw_spectral_scan_status_t) -> ffi::c_int {
        unimplemented!()
    }

    /**
    @brief Get the channel scan results
    @param levels an array containing the power levels for which the scan results are given
    @param values ar array containing the results of the scan for each power levels
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_spectral_scan_get_results(
        levels_dbm: *mut [i16; LGW_SPECTRAL_SCAN_RESULT_SIZE],
        results: *mut [u16; LGW_SPECTRAL_SCAN_RESULT_SIZE],
    ) -> ffi::c_int {
        unimplemented!()
    }

    /**
    @brief Abort the current scan
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_spectral_scan_abort(&mut self, ) -> ffi::c_int {
        unimplemented!()
    }
}

/// using the physical device to back this build (aka hardware attached)
pub struct PhysicalDevice;
impl DeviceBackingAPI for PhysicalDevice {
    unsafe fn lgw_board_setconf(&mut self, conf: *mut lgw_conf_board_s) -> ffi::c_int {
        unsafe { bindings_loragw_hal::lgw_board_setconf(conf) }
    }

    unsafe fn lgw_rxrf_setconf(&mut self, rf_chain: u8, conf: *mut lgw_conf_rxrf_s) -> ffi::c_int {
        unsafe { bindings_loragw_hal::lgw_rxrf_setconf(rf_chain, conf) }
    }

    unsafe fn lgw_rxif_setconf(&mut self, if_chain: u8, conf: *mut lgw_conf_rxif_s) -> ffi::c_int {
        unsafe { bindings_loragw_hal::lgw_rxif_setconf(if_chain, conf) }
    }

    unsafe fn lgw_demod_setconf(&mut self, conf: *mut lgw_conf_demod_s) -> ffi::c_int {
        unsafe { bindings_loragw_hal::lgw_demod_setconf(conf) }
    }

    unsafe fn lgw_txgain_setconf(&mut self, rf_chain: u8, conf: *mut lgw_tx_gain_lut_s) -> ffi::c_int {
        unsafe { bindings_loragw_hal::lgw_txgain_setconf(rf_chain, conf) }
    }

    unsafe fn lgw_ftime_setconf(&mut self, conf: *mut lgw_conf_ftime_s) -> ffi::c_int {
        unsafe { bindings_loragw_hal::lgw_ftime_setconf(conf) }
    }

    unsafe fn lgw_start(&mut self) -> ffi::c_int {
        unsafe { bindings_loragw_hal::lgw_start() }
    }

    unsafe fn lgw_stop(&mut self) -> ffi::c_int {
        unsafe { bindings_loragw_hal::lgw_stop() }
    }

    unsafe fn lgw_receive(&mut self, max_pkt: u8, pkt_data: *mut lgw_pkt_rx_s) -> ffi::c_int {
        unsafe { bindings_loragw_hal::lgw_receive(max_pkt, pkt_data) }
    }

    unsafe fn lgw_send(&mut self, pkt_data: *mut lgw_pkt_tx_s) -> ffi::c_int {
        unsafe { bindings_loragw_hal::lgw_send(pkt_data) }
    }

    unsafe fn lgw_status(&mut self, rf_chain: u8, select: u8, code: *mut u8) -> ffi::c_int {
        unsafe { bindings_loragw_hal::lgw_status(rf_chain, select, code) }
    }

    unsafe fn lgw_get_eui(&mut self, eui: *mut u64) -> ffi::c_int {
        unsafe { bindings_loragw_hal::lgw_get_eui(eui) }
    }

    unsafe fn lgw_get_temperature(&mut self, temperature: *mut ffi::c_float) -> ffi::c_int {
        unsafe { bindings_loragw_hal::lgw_get_temperature(temperature) }
    }

    unsafe fn lgw_version_info(&mut self, ) -> *const ffi::c_char {
        unsafe { bindings_loragw_hal::lgw_version_info() }
    }
}

/// backing and test harness used for unit tests and driver integration tests  
/// 
/// DO NOT USE FOR NON-TESTING/DEBUG/BENCHMARK PURPOSES
#[cfg(test)]
#[allow(non_upper_case_globals)]
pub mod unit_test_backing {
    use std::{cell::UnsafeCell, mem::MaybeUninit};

    use crate::sx1302::{bindings_loragw_hal::{LGW_HAL_ERROR, LGW_HAL_SUCCESS}, testing::{FunctionState, Nothing, ParamType, TestHarness, call_harness_hook, new_FunctionData, new_TestHarness}};

    use super::*;

    /// this is a specific backing implenmentation so that tests can hook onto the lora module driver API calls
    /// and check if the upstream code is passing in correct arguments and handling returns by driver API correctly
    /// 
    /// NOTE: These test harness can only provide hooks for the most recent function call.
    /// 
    /// Users of test hooks must assign the apporiate fields before the test harnesses are invoked by what ever you are testing
    /// 
    /// Read backing.rs for more details
    pub struct UnitTestDevice {
        pub lgw_board_setconf_harness: TestHarness<ffi::c_int, lgw_conf_board_s>,
        pub lgw_rxrf_setconf_harness: TestHarness<ffi::c_int, u8, lgw_conf_rxrf_s>,
        pub lgw_rxif_setconf_harness: TestHarness<ffi::c_int, u8, lgw_conf_rxif_s>,
        pub lgw_demod_setconf_harness: TestHarness<ffi::c_int, lgw_conf_demod_s>,
        pub lgw_txgain_setconf_harness: TestHarness<ffi::c_int, u8, lgw_tx_gain_lut_s>,
        pub lgw_ftime_setconf_harness: TestHarness<ffi::c_int, lgw_conf_ftime_s>,
        pub lgw_start_harness: TestHarness<ffi::c_int>,
        pub lgw_stop_harness: TestHarness<ffi::c_int>,
        pub lgw_receive_harness: TestHarness<ffi::c_int, u8, Vec<lgw_pkt_rx_s>>, // Vec is used here for convience
        pub lgw_send_harness: TestHarness<ffi::c_int, lgw_pkt_tx_s>,
        pub lgw_status_harness: TestHarness<ffi::c_int, u8, u8, u8>,
        pub lgw_get_eui_harness: TestHarness<ffi::c_int, u64>,
        pub lgw_get_temperature_harness: TestHarness<ffi::c_int, ffi::c_float>,
        pub lgw_version_info_harness: TestHarness<ffi::CString>, // Cstring is used here for convenience
    }
    impl UnitTestDevice {
        pub fn new() -> Self {
            Self {
                lgw_board_setconf_harness: TestHarness::new(new_FunctionData! {
                    ret w LGW_HAL_SUCCESS,
                }),
                lgw_rxrf_setconf_harness: new_TestHarness!{
                    ret w LGW_HAL_SUCCESS,
                },
                lgw_rxif_setconf_harness: new_TestHarness!(
                    ret w LGW_HAL_SUCCESS,
                ),
                lgw_demod_setconf_harness: new_TestHarness!(
                    ret w LGW_HAL_SUCCESS,
                ),
                lgw_txgain_setconf_harness: new_TestHarness!(
                    ret w LGW_HAL_SUCCESS,
                ),
                lgw_ftime_setconf_harness: new_TestHarness!(
                    ret w LGW_HAL_SUCCESS,
                ),
                lgw_start_harness: new_TestHarness!(
                    ret w LGW_HAL_SUCCESS,
                ),
                lgw_stop_harness: new_TestHarness!(
                    ret w LGW_HAL_SUCCESS,
                ),
                lgw_receive_harness: new_TestHarness!(
                    ret w 0,
                    arg2 w Vec::new()
                ),
                lgw_send_harness: new_TestHarness!(
                    ret w LGW_HAL_SUCCESS,
                ),
                lgw_status_harness: new_TestHarness!(
                    ret w LGW_HAL_SUCCESS,
                    arg3 w 0 // TX/RX status unknown
                ),
                lgw_get_eui_harness: new_TestHarness!(
                    ret w LGW_HAL_SUCCESS,
                    arg1 w 0xE28C26247951561C // random
                ),
                lgw_get_temperature_harness: new_TestHarness!(
                    ret w LGW_HAL_SUCCESS,
                    arg1 w 30.0,
                ),
                lgw_version_info_harness: new_TestHarness!(
                    ret w ffi::CString::new("cpss_testing_default").unwrap(),
                ),

            }
        }
    }

    impl DeviceBackingAPI for UnitTestDevice {
        unsafe fn lgw_board_setconf(&mut self, conf: *mut lgw_conf_board_s) -> ffi::c_int {
            call_harness_hook!(self.lgw_board_setconf_harness, unsafe {&mut *conf})
                .expect("TESTING: lgw_board_setconf expects a return value.")
        }

        unsafe fn lgw_rxrf_setconf(&mut self, rf_chain: u8, conf: *mut lgw_conf_rxrf_s) -> ffi::c_int {
            call_harness_hook!(self.lgw_rxrf_setconf_harness, UnsafeCell::new(rf_chain).get_mut(), unsafe {&mut *conf})
                .expect("TESTING: lgw_rxrf_setconf expects a return value.")
        }

        unsafe fn lgw_rxif_setconf(&mut self, if_chain: u8, conf: *mut lgw_conf_rxif_s) -> ffi::c_int {
            call_harness_hook!(self.lgw_rxif_setconf_harness, UnsafeCell::new(if_chain).get_mut(), unsafe {&mut *conf})
                .expect("TESTING: lgw_rxif_setconf expects a return value.")
        }

        unsafe fn lgw_demod_setconf(&mut self, conf: *mut lgw_conf_demod_s) -> ffi::c_int {
            call_harness_hook!(self.lgw_demod_setconf_harness, unsafe {&mut *conf})
                .expect("TESTING: lgw_demod_setconf expects a return value.")
        }

        unsafe fn lgw_txgain_setconf(&mut self, rf_chain: u8, conf: *mut lgw_tx_gain_lut_s) -> ffi::c_int {
            call_harness_hook!(self.lgw_txgain_setconf_harness, UnsafeCell::new(rf_chain).get_mut(), unsafe {&mut *conf})
                .expect("TESTING: lgw_txgain_setconf expects a return value.")
        }

        unsafe fn lgw_ftime_setconf(&mut self, conf: *mut lgw_conf_ftime_s) -> ffi::c_int {
            call_harness_hook!(self.lgw_ftime_setconf_harness, unsafe {&mut *conf})
                .expect("TESTING: lgw_ftime_setconf expects a return value.")
        }

        unsafe fn lgw_start(&mut self) -> ffi::c_int {
            call_harness_hook!(self.lgw_start_harness)
                .expect("TESTING: lgw_start expects a return value.")
        }

        unsafe fn lgw_stop(&mut self) -> ffi::c_int {
            call_harness_hook!(self.lgw_stop_harness)
                .expect("TESTING: lgw_stop expects a return value.")
        }

        unsafe fn lgw_receive(&mut self, max_pkt: u8, pkt_data: *mut lgw_pkt_rx_s) -> ffi::c_int {
            let mut pkt_data_buf: Vec<lgw_pkt_rx_s> = Vec::with_capacity(max_pkt as usize);
            let v = call_harness_hook!(self.lgw_receive_harness, UnsafeCell::new(max_pkt).get_mut(), &mut pkt_data_buf)
                    .expect("TESTING: lgw_receive expects a return value.");
                        
            // SAFETY: assuming pkt_data is a correctly sized continous memory array passed from caller
            let pkt_data_slice = unsafe { std::slice::from_raw_parts_mut(pkt_data, max_pkt as usize) };
            let end_size = if pkt_data_buf.len() < max_pkt as usize { pkt_data_buf.len() } else { max_pkt as usize};
            pkt_data_slice[0..end_size].copy_from_slice(&pkt_data_buf[0..end_size]);

            v
        }

        unsafe fn lgw_send(&mut self, pkt_data: *mut lgw_pkt_tx_s) -> ffi::c_int {
            call_harness_hook!(self.lgw_send_harness, unsafe { &mut *pkt_data } )
                .expect("TESTING: lgw_send expects a return value.")
        }

        unsafe fn lgw_status(&mut self, rf_chain: u8, select: u8, code: *mut u8) -> ffi::c_int {
            call_harness_hook!(self.lgw_status_harness, UnsafeCell::new(rf_chain).get_mut(), UnsafeCell::new(select).get_mut(), unsafe {&mut *code} )
                .expect("TESTING: lgw_status expects a return value.")
        }

        unsafe fn lgw_get_eui(&mut self, eui: *mut u64) -> ffi::c_int {
            call_harness_hook!(self.lgw_get_eui_harness, unsafe { &mut *eui } )
                .expect("TESTING: lgw_get_eui expects a return value.")
        }

        unsafe fn lgw_get_temperature(&mut self, temperature: *mut ffi::c_float) -> ffi::c_int {
            call_harness_hook!(self.lgw_get_temperature_harness, unsafe { &mut *temperature })
                .expect("TESTING: lgw_version_info expects a return value.")
        }

        unsafe fn lgw_version_info(&mut self) -> *const ffi::c_char {
            call_harness_hook!(self.lgw_version_info_harness)
                .expect("TESTING: lgw_version_info expects a return value.").as_c_str().as_ptr()
        }

    }
}
