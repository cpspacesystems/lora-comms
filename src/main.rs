use std::os::raw::c_void;


mod publisher;
mod subscriber;

pub const LR11XX_RADIO_GFSK_SYNC_WORD_LENGTH: usize = 0; //change to what is needed /////

#[repr(C)]
pub enum lr11xx_hal_status_t {
    LR11XX_HAL_STATUS_OK = 0,
    LR11XX_HAL_STATUS_ERROR = 3,
}
#[repr(C)]
pub enum lr11xx_status_t {
    LR11XX_HAL_STATUS_OK = 0,
    LR11XX_HAL_STATUS_ERROR = 3,
}
#[repr(C)]
pub enum lr11xx_radio_lora_network_type_t {
    LR11XX_RADIO_LORA_PRIVATE_NETWORK = 0,
    LR11XX_RADIO_LORA_PUBLIC_NETWORK  = 1
}
#[repr(C)]
pub enum lr11xx_radio_lna_mode_t {
    LR11XX_RADIO_LNA_MODE_SINGLE_RFI_N_LF0 = 1,  // Use only RFI_N_LF0 antenna
    LR11XX_RADIO_LNA_MODE_SINGLE_RFI_P_LF0 = 2,  // Use only RFI_P_LF0 antenna
    LR11XX_RADIO_LNA_MODE_DIFFERENTIAL_LF0 = 3   // Configure LNA LF0 in differential mode (default)
}
#[repr(C)]
pub enum lr11xx_radio_intermediary_mode_t {
    LR11XX_RADIO_MODE_SLEEP = 0x00,  // Sleep / Not recommended with LR1110 FW from 0x0303 to 0x0307 and LR1120 FW
                                     // 0x0101 in case of transition from Rx to Tx in LoRa
    LR11XX_RADIO_MODE_STANDBY_RC   = 0x01,  // Standby RC
    LR11XX_RADIO_MODE_STANDBY_XOSC = 0x02,  // Standby XOSC
    LR11XX_RADIO_MODE_FS           = 0x03   // Frequency Synthesis
}
pub struct  context {}

#[link(name = "lr11xx_driver", kind = "static")]
unsafe extern "C" {
    fn adfadfa();
   fn lr11xx_hal_write(context: *const std::ffi::c_void, command : *const i8, command_length : u16, data: *const u8, 
                                data_length : u16) -> lr11xx_hal_status_t;


    fn lr11xx_radio_set_pkt_type(context : *const std::ffi::c_void, pkt_type: u8) -> lr11xx_status_t;
    //lr11xx_status_t lr11xx_radio_set_gfsk_sync_word( const void*   context,
    //                                             const uint8_t gfsk_sync_word[LR11XX_RADIO_GFSK_SYNC_WORD_LENGTH] );
    fn lr11xx_radio_set_lora_sync_word(context : *const std::ffi::c_void, sync_word : u8) -> lr11xx_status_t;
    //lr11xx_status_t lr11xx_radio_set_lr_fhss_sync_word( const void*   context,
    //                                                const uint8_t sync_word[LR11XX_RADIO_LR_FHSS_SYNC_WORD_LENGTH] );
    fn fnlr11xx_radio_set_lora_public_network(context : *const std::ffi::c_void, network_type : lr11xx_radio_lora_network_type_t  ) -> lr11xx_status_t;
    fn  lr11xx_radio_set_rx(context : *const std::ffi::c_void, timeout_in_ms : u32) -> lr11xx_status_t;
    fn  lr11xx_radio_set_rx_and_lna_mode(context : *const std::ffi::c_void, timeout_in_ms : u32,
                                                lna_mode : lr11xx_radio_lna_mode_t) -> lr11xx_status_t;
    fn lr11xx_radio_set_pkt_address(context : *const std::ffi::c_void, node_address : u8,
                                              broadcast_address : u8) -> lr11xx_status_t;
    fn lr11xx_radio_set_cad(context : *const std::ffi::c_void) -> lr11xx_status_t;
    fn lr11xx_radio_set_tx_cw(context : *const std::ffi::c_void) -> lr11xx_status_t;
    fn lr11xx_radio_set_tx_infinite_preamble(context : *const std::ffi::c_void) -> lr11xx_status_t; //check firmware version
    fn lr11xx_radio_set_lora_sync_timeout(context : *const std::ffi::c_void, nb_symbol : u16) -> lr11xx_status_t; //depends on firmware
    fn lr11xx_radio_set_rx_with_timeout_in_rtc_step(context : *const std::ffi::c_void, timeout_in_rtc_step : u32) -> lr11xx_status_t;
    fn  lr11xx_radio_set_rx_with_timeout_in_rtc_step_and_lna_mode(context : *const std::ffi::c_void, timeout_in_rtc_step : u32,
                                                                           lna_mode : lr11xx_radio_lna_mode_t) -> lr11xx_status_t;
    fn lr11xx_radio_set_tx(context : *const std::ffi::c_void, timeout_in_ms : u32) -> lr11xx_status_t;
    fn lr11xx_radio_set_tx_with_timeout_in_rtc_step(context : *const std::ffi::c_void, timeout_in_rtc_step : u32) -> lr11xx_status_t;
    fn  lr11xx_radio_set_rf_freq(context : *const std::ffi::c_void, freq_in_hz : u32) -> lr11xx_status_t;
    fn  lr11xx_radio_auto_tx_rx(context : *const std::ffi::c_void, delay : u32,
                                        intermediary_mode : lr11xx_radio_intermediary_mode_t , timeout : u32) -> lr11xx_status_t;
}
fn main() {
    // let mut a = Vec::<u8>::new().as_ptr();
    // unsafe {
    //     lr11xx_hal_write(context, a, command_length, data, data_length)
    // }
    unsafe { adfadfa(); }
}




// 8 bit enum instead of string key

//config function
//send function

