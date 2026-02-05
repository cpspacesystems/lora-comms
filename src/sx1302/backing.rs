use std::ffi;

use crate::sx1302::bindings_loragw_hal::{self, LGW_SPECTRAL_SCAN_RESULT_SIZE, lgw_conf_board_s, lgw_conf_debug_s, lgw_conf_demod_s, lgw_conf_ftime_s, lgw_conf_rxif_s, lgw_conf_rxrf_s, lgw_conf_sx1261_s, lgw_pkt_rx_s, lgw_pkt_tx_s, lgw_spectral_scan_status_t, lgw_tx_gain_lut_s};


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
    unsafe fn lgw_board_setconf(conf: *mut lgw_conf_board_s) -> ffi::c_int;

    /**
    @brief Configure an RF chain (must configure before start)
    @param rf_chain number of the RF chain to configure [0, LGW_RF_CHAIN_NB - 1]
    @param conf structure containing the configuration parameters
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_rxrf_setconf(rf_chain: u8, conf: *mut lgw_conf_rxrf_s) -> ffi::c_int;

    /**
    @brief Configure an IF chain + modem (must configure before start)
    @param if_chain number of the IF chain + modem to configure [0, LGW_IF_CHAIN_NB - 1]
    @param conf structure containing the configuration parameters
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_rxif_setconf(if_chain: u8, conf: *mut lgw_conf_rxif_s) -> ffi::c_int;

    /**
    @brief Configure LoRa/FSK demodulators
    @param conf structure containing the configuration parameters
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_demod_setconf(conf: *mut lgw_conf_demod_s) -> ffi::c_int;

    /**
    @brief Configure the Tx gain LUT
    @param conf pointer to structure defining the LUT
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_txgain_setconf(rf_chain: u8, conf: *mut lgw_tx_gain_lut_s) -> ffi::c_int;

    /**
    @brief Configure the fine timestamping
    @param conf pointer to structure defining the config to be applied
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_ftime_setconf(conf: *mut lgw_conf_ftime_s) -> ffi::c_int;

    /*
    @brief Configure the SX1261 radio for LBT/Spectral Scan
    @param pointer to structure defining the config to be applied
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_sx1261_setconf(conf: *mut lgw_conf_sx1261_s) -> ffi::c_int { unimplemented!() }

    /**
    @brief Configure the debug context
    @param conf pointer to structure defining the config to be applied
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_debug_setconf(conf: *mut lgw_conf_debug_s) -> ffi::c_int { unimplemented!() }

    /**
    @brief Connect to the LoRa concentrator, reset it and configure it according to previously set parameters
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_start() -> ffi::c_int;

    /**
    @brief Stop the LoRa concentrator and disconnect it
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_stop() -> ffi::c_int;

    /**
    @brief A non-blocking function that will fetch up to 'max_pkt' packets from the LoRa concentrator FIFO and data buffer
    @param max_pkt maximum number of packet that must be retrieved (equal to the size of the array of struct)
    @param pkt_data pointer to an array of struct that will receive the packet metadata and payload pointers
    @return LGW_HAL_ERROR id the operation failed, else the number of packets retrieved
    */
    unsafe fn lgw_receive(max_pkt: u8, pkt_data: *mut lgw_pkt_rx_s) -> ffi::c_int;

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
    unsafe fn lgw_send(pkt_data: *mut lgw_pkt_tx_s) -> ffi::c_int;

    /**
    @brief Give the the status of different part of the LoRa concentrator
    @param select is used to select what status we want to know
    @param code is used to return the status code
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_status(rf_chain: u8, select: u8, code: *mut u8) -> ffi::c_int;

    /**
    @brief Abort a currently scheduled or ongoing TX
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_abort_tx(rf_chain: u8) -> ffi::c_int { unimplemented!() }

    /**
    @brief Return value of internal counter when latest event (eg GPS pulse) was captured
    @param trig_cnt_us pointer to receive timestamp value
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_get_trigcnt(trig_cnt_us: *mut u32) -> ffi::c_int { unimplemented!() }

    /**
    @brief Return instateneous value of internal counter
    @param inst_cnt_us pointer to receive timestamp value
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_get_instcnt(inst_cnt_us: *mut u32) -> ffi::c_int { unimplemented!() }

    /**
    @brief Return the LoRa concentrator EUI (Extended Unique Identifier)
    @param eui pointer to receive eui
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_get_eui(eui: *mut u64) -> ffi::c_int;

    /**
    @brief Return the temperature measured by the LoRa concentrator sensor
    @param temperature The temperature measured, in degree celcius
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_get_temperature(temperature: *mut ffi::c_float) -> ffi::c_int;

    /**
    @brief Allow user to check the version/options of the library once compiled
    @return pointer on a human-readable null terminated string
    */
    unsafe fn lgw_version_info() -> *const ffi::c_char;

    /**
    @brief Return time on air of given packet, in milliseconds
    @param packet is a pointer to the packet structure
    @return the packet time on air in milliseconds
    */
    unsafe fn lgw_time_on_air(packet: *const lgw_pkt_tx_s) -> u32 { unimplemented!() }

    /**
    @brief Start scaning the channel centered on the given frequency
    @param freq_hz channel center frequency
    @param nb_scan number of measures to be done for the scan
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_spectral_scan_start(freq_hz: u32, nb_scan: u16) -> ffi::c_int { unimplemented!() }

    /**
    @brief Get the current scan status
    @param status a pointer to the returned status
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_spectral_scan_get_status(status: *mut lgw_spectral_scan_status_t) -> ffi::c_int { unimplemented!() }

    /**
    @brief Get the channel scan results
    @param levels an array containing the power levels for which the scan results are given
    @param values ar array containing the results of the scan for each power levels
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_spectral_scan_get_results(levels_dbm: *mut [i16; LGW_SPECTRAL_SCAN_RESULT_SIZE], 
        results: *mut [u16; LGW_SPECTRAL_SCAN_RESULT_SIZE]) -> ffi::c_int { unimplemented!() }

    /**
    @brief Abort the current scan
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    unsafe fn lgw_spectral_scan_abort() -> ffi::c_int { unimplemented!() }
}

/// using the physical device to back this build (aka hardware attached)
pub struct PhysicalDevice;  
impl DeviceBackingAPI for PhysicalDevice {
    unsafe fn lgw_board_setconf(conf: *mut lgw_conf_board_s) -> ffi::c_int {
        unsafe { bindings_loragw_hal::lgw_board_setconf(conf) }
    }

    unsafe fn lgw_rxrf_setconf(rf_chain: u8, conf: *mut lgw_conf_rxrf_s) -> ffi::c_int {
        unsafe { bindings_loragw_hal::lgw_rxrf_setconf(rf_chain, conf) }
    }

    unsafe fn lgw_rxif_setconf(if_chain: u8, conf: *mut lgw_conf_rxif_s) -> ffi::c_int {
        unsafe { bindings_loragw_hal::lgw_rxif_setconf(if_chain, conf) }
    }

    unsafe fn lgw_demod_setconf(conf: *mut lgw_conf_demod_s) -> ffi::c_int {
        unsafe { bindings_loragw_hal::lgw_demod_setconf(conf) }
    }

    unsafe fn lgw_txgain_setconf(rf_chain: u8, conf: *mut lgw_tx_gain_lut_s) -> ffi::c_int {
        unsafe { bindings_loragw_hal::lgw_txgain_setconf(rf_chain, conf) }
    }

    unsafe fn lgw_ftime_setconf(conf: *mut lgw_conf_ftime_s) -> ffi::c_int {
        unsafe { bindings_loragw_hal::lgw_ftime_setconf(conf) }
    }

    unsafe fn lgw_start() -> ffi::c_int {
        unsafe { bindings_loragw_hal::lgw_start() }
    }

    unsafe fn lgw_stop() -> ffi::c_int {
        unsafe { bindings_loragw_hal:: lgw_stop() }
    }

    unsafe fn lgw_receive(max_pkt: u8, pkt_data: *mut lgw_pkt_rx_s) -> ffi::c_int {
        unsafe { bindings_loragw_hal::lgw_receive(max_pkt, pkt_data) }
    }

    unsafe fn lgw_send(pkt_data: *mut lgw_pkt_tx_s) -> ffi::c_int {
        unsafe { bindings_loragw_hal::lgw_send(pkt_data) }
    }

    unsafe fn lgw_status(rf_chain: u8, select: u8, code: *mut u8) -> ffi::c_int {
        unsafe { bindings_loragw_hal::lgw_status(rf_chain, select, code) }
    }

    unsafe fn lgw_get_eui(eui: *mut u64) -> ffi::c_int {
        unsafe { bindings_loragw_hal::lgw_get_eui(eui) }
    }

    unsafe fn lgw_get_temperature(temperature: *mut ffi::c_float) -> ffi::c_int {
        unsafe { bindings_loragw_hal::lgw_get_temperature(temperature) }
    }

    unsafe fn lgw_version_info() -> *const ffi::c_char {
        unsafe { bindings_loragw_hal::lgw_version_info() }
    }
}

/// backing and test harness used for unit tests (non thread safe)
#[allow(non_upper_case_globals)]
pub mod unit_test_backing {
    use std::mem::MaybeUninit;
    use paste::paste;

    use super::*;

    /// Creates test harness for a function
    /// 
    /// make_test_harness! {<br>
    /// &emsp;&emsp; name `function name`,<br>
    /// &emsp;&emsp; ret_type `return type`,<br>
    /// &emsp;&emsp; arg `function param1 name` type `function param1 type` <br>
    /// &emsp;&emsp; arg `function param2 name` type `function param2 type` <br>
    /// &emsp;&emsp; `... for any more args, or none for zero args`<br>
    /// } <br>
    /// 
    /// Ex: 
    /// 
    /// make_test_harness! { <br>
    /// &emsp;&emsp; name lgw_board_setconf, <br>
    /// &emsp;&emsp; ret_type ffi::c_int, <br>
    /// &emsp;&emsp; arg conf type lgw_conf_board_s <br>
    /// } <br>
    macro_rules! make_test_harness {
        (   name $func_name:ident, 
            ret_type $return_type:ty,
            $(arg $arg_name:ident type $arg_type:ty),*
            $(,)?          
        ) => {paste! {
            // static mut lgw_board_setconf_ret: ffi::c_int = unsafe { MaybeUninit::zeroed().assume_init() };   
            static mut [<$func_name _expected_ret>]: $return_type = unsafe { MaybeUninit::zeroed().assume_init() };
            // static mut lgw_board_setconf_good: bool = false;
            static mut [<$func_name _good>]: bool = false;
            // static mut lgw_board_expected_conf: *lgw_conf_board_s = unsafe { MaybeUninit::zeroed().assume_init() };
            // ... if there is more arguments/parameters, everything is zero inited
            $(
                static mut [<$func_name _expected_ $arg_name>]: $arg_type = unsafe { MaybeUninit::zeroed().assume_init() };
            )*
            // pub fn set_harness_lgw_board_setconf(ret: ffi::c_int, conf: *lgw_conf_board_s) {
            #[doc = "sets test harness for `" $func_name "`, after this is called, `" $func_name "` can be safely called"]
            pub fn [<set_harness_ $func_name>](ret: $return_type $(, $arg_name: $arg_type)*) {
                unsafe {
                    [<$func_name _expected_ret>] = ret; // lgw_board_setconf_ret = ret;
                    [<$func_name _good>] = false; // lgw_board_setconf_good = false;
                    $([<$func_name _expected_ $arg_name>] = $arg_name;)* // lgw_board_setconf_expected_conf = conf; // ... and repeats for more arguments
                }
            }
            // pub fn check_harness_result_lgw_board_setconf() -> bool {
            #[doc = "checks if the test result matches the what is set in the test harness for `" $func_name "`."]
            pub fn [<check_harness_result_ $func_name>]() -> bool {
                // unsafe { lgw_board_setconf_good }
                unsafe { [<$func_name _good>] }
            }
        }};
    }
    /// resolve pointers first, then compare
    macro_rules! handle_equals {
        (*mut $arg_type:ty, $left:expr, $right:expr) => {
            *$left == *$right
        };
        (*const $arg_type:ty, $left:expr, $right:expr) => {
            *$left == *$right
        };
        ($arg_type:ty, $left:expr, $right:expr) => {
            $left == $right
        };
    }


    /// Creates hooks for custom backing impl for test harness
    /// 
    /// Very similar syntax to make_test_harness, it also creates all the actually trait impls for DeviceBackingAPI
    /// 
    /// scroll down for 20 lines to see the examples
    macro_rules! create_UnitTestDevice {
        (   $(harness 
                name $func_name:ident, 
                ret_type $return_type:ty,
                $(arg $arg_name:ident type $arg_type:ty),*
                $(,)?
            )*
        ) => {paste! {
            $(
                make_test_harness! { // output of make_test_harness, for every single harness
                    name $func_name,
                    ret_type $return_type,
                    $(arg $arg_name type $arg_type),*
                }
            )*

            // struct defination
            /// this is a specific backing implenmentation so that tests can hook onto the lora module driver API calls
            /// and check if the upstream code is passing in correct arguments 
            pub struct UnitTestDevice;
            impl DeviceBackingAPI for UnitTestDevice {
                $( // repeadtely creates all the functions for every single harness 
                // unsage fn lgw_board_setconf(conf: *mut lgw_conf_board_s) -> ffi::c_int {
                unsafe fn $func_name($($arg_name: $arg_type),*) -> $return_type {
                    // unsafe { lgw_board_setconf_good = true   // if there is no arguments to check, _good is auto set to true
                    unsafe { [<$func_name _good>] = true
                        // && conf == lgw_conf_board_setconf_conf
                        $(&& handle_equals!($arg_type, $arg_name, [<$func_name _expected_ $arg_name>]))*;
                    }
                    return unsafe { [<$func_name _expected_ret>] };
                }
                )*
            }
        }};
    }

    create_UnitTestDevice! {
        harness
            name lgw_board_setconf,
            ret_type ffi::c_int,
            arg conf type *mut lgw_conf_board_s,
        harness
            name lgw_rxrf_setconf,
            ret_type ffi::c_int,
            arg rf_chain type u8,
            arg conf type *mut lgw_conf_rxrf_s,
        harness
            name lgw_rxif_setconf,
            ret_type ffi::c_int,
            arg if_chain type u8,
            arg conf type *mut lgw_conf_rxif_s,
        harness
            name lgw_demod_setconf, 
            ret_type ffi::c_int,
            arg conf type *mut lgw_conf_demod_s,
        harness
            name lgw_txgain_setconf, 
            ret_type ffi::c_int,
            arg rf_chain type u8,
            arg conf type *mut lgw_tx_gain_lut_s
        harness
            name lgw_ftime_setconf, 
            ret_type ffi::c_int,
            arg conf type *mut lgw_conf_ftime_s,
        harness
            name lgw_start, 
            ret_type ffi::c_int,
        harness
            name lgw_stop, 
            ret_type ffi::c_int,
        harness
            name lgw_receive, 
            ret_type ffi::c_int,
            arg max_pkt type u8,
            arg pkt_data type *mut lgw_pkt_rx_s,
        harness
            name lgw_send, 
            ret_type ffi::c_int,
            arg pkt_data type *mut lgw_pkt_tx_s,
        harness
            name lgw_status, 
            ret_type ffi::c_int,
            arg rf_chain type u8,
            arg select type u8,
            arg code type *mut u8,
        harness
            name lgw_get_eui, 
            ret_type ffi::c_int,
            arg eui type *mut u64,
        harness
            name lgw_get_temperature, 
            ret_type ffi::c_int,
            arg temperature type *mut ffi::c_float,
        harness
            name lgw_version_info,
            ret_type *const ffi::c_char,
    }


    
}
