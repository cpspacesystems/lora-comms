use std::os::raw::c_void;

use bitvec::ptr;


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

pub enum lr11xx_radio_lora_sf_t {
    LR11XX_RADIO_LORA_SF5  = 0x05,  // Spreading Factor 5
    LR11XX_RADIO_LORA_SF6  = 0x06,  // Spreading Factor 6
    LR11XX_RADIO_LORA_SF7  = 0x07,  // Spreading Factor 7
    LR11XX_RADIO_LORA_SF8  = 0x08,  // Spreading Factor 8
    LR11XX_RADIO_LORA_SF9  = 0x09,  // Spreading Factor 9
    LR11XX_RADIO_LORA_SF10 = 0x0A,  // Spreading Factor 10
    LR11XX_RADIO_LORA_SF11 = 0x0B,  // Spreading Factor 11
    LR11XX_RADIO_LORA_SF12 = 0x0C,  // Spreading Factor 12
}

pub enum lr11xx_radio_lora_bw_t{
    LR11XX_RADIO_LORA_BW_10  = 0x08,  // Bandwidth 10.42 kHz
    LR11XX_RADIO_LORA_BW_15  = 0x01,  // Bandwidth 15.63 kHz
    LR11XX_RADIO_LORA_BW_20  = 0x09,  // Bandwidth 20.83 kHz
    LR11XX_RADIO_LORA_BW_31  = 0x02,  // Bandwidth 31.25 kHz
    LR11XX_RADIO_LORA_BW_41  = 0x0A,  // Bandwidth 41.67 kHz
    LR11XX_RADIO_LORA_BW_62  = 0x03,  // Bandwidth 62.50 kHz
    LR11XX_RADIO_LORA_BW_125 = 0x04,  // Bandwidth 125.00 kHz
    LR11XX_RADIO_LORA_BW_250 = 0x05,  // Bandwidth 250.00 kHz
    LR11XX_RADIO_LORA_BW_500 = 0x06,  // Bandwidth 500.00 kHz
    LR11XX_RADIO_LORA_BW_200 = 0x0D,  // Bandwidth 203.00 kHz, 2G4 and compatible with LR112x chips only
    LR11XX_RADIO_LORA_BW_400 = 0x0E,  // Bandwidth 406.00 kHz, 2G4 and compatible with LR112x chips only
    LR11XX_RADIO_LORA_BW_800 = 0x0F,  // Bandwidth 812.00 kHz, 2G4 and compatible with LR112x chips only
}

pub enum lr11xx_radio_lora_cr_t {
    LR11XX_RADIO_LORA_NO_CR     = 0x00,  // No Coding Rate
    LR11XX_RADIO_LORA_CR_4_5    = 0x01,  // Coding Rate 4/5 Short Interleaver
    LR11XX_RADIO_LORA_CR_4_6    = 0x02,  // Coding Rate 4/6 Short Interleaver
    LR11XX_RADIO_LORA_CR_4_7    = 0x03,  // Coding Rate 4/7 Short Interleaver
    LR11XX_RADIO_LORA_CR_4_8    = 0x04,  // Coding Rate 4/8 Short Interleaver
    LR11XX_RADIO_LORA_CR_LI_4_5 = 0x05,  // Coding Rate 4/5 Long Interleaver
    LR11XX_RADIO_LORA_CR_LI_4_6 = 0x06,  // Coding Rate 4/6 Long Interleaver
    LR11XX_RADIO_LORA_CR_LI_4_8 = 0x07,  // Coding Rate 4/8 Long Interleaver
}

pub enum lr11xx_radio_lora_pkt_len_modes_t {
    LR11XX_RADIO_LORA_PKT_EXPLICIT = 0x00,  // Explicit header: transmitted over the air
    LR11XX_RADIO_LORA_PKT_IMPLICIT = 0x01,  // Implicit header: not transmitted over the air
}

pub enum lr11xx_radio_lora_crc_t {
    LR11XX_RADIO_LORA_CRC_OFF = 0x00,  // CRC deactivated
    LR11XX_RADIO_LORA_CRC_ON  = 0x01,  // CRC activated
}

pub enum lr11xx_radio_lora_iq_t {
    LR11XX_RADIO_LORA_IQ_STANDARD = 0x00,  // IQ standard
    LR11XX_RADIO_LORA_IQ_INVERTED = 0x01,  // IQ inverted
}

pub enum lr11xx_radio_fallback_modes_t {
    LR11XX_RADIO_FALLBACK_STDBY_RC   = 0x01,  // Standby RC (Default)
    LR11XX_RADIO_FALLBACK_STDBY_XOSC = 0x02,  // Standby XOSC
    LR11XX_RADIO_FALLBACK_FS         = 0x03   // FS
}

pub enum lr11xx_radio_rx_duty_cycle_mode_t {
    LR11XX_RADIO_RX_DUTY_CYCLE_MODE_RX  = 0x00,  // LoRa/GFSK: Uses Rx for listening to packets
    LR11XX_RADIO_RX_DUTY_CYCLE_MODE_CAD = 0x01,  // Only in LoRa: Uses CAD to listen for over-the-air activity
}

pub enum lr11xx_radio_pa_selection_t {
    LR11XX_RADIO_PA_SEL_LP = 0x00,  // Low-power Power Amplifier
    LR11XX_RADIO_PA_SEL_HP = 0x01,  // High-power Power Amplifier
    LR11XX_RADIO_PA_SEL_HF = 0x02,  // High-frequency Power Amplifier
}

pub enum lr11xx_radio_pa_reg_supply_t {
    LR11XX_RADIO_PA_REG_SUPPLY_VREG = 0x00,  // Power amplifier supplied by the main regulator
    LR11XX_RADIO_PA_REG_SUPPLY_VBAT = 0x01   // Power amplifier supplied by the battery
}







#[repr(C)]
pub struct lr11xx_radio_mod_params_lora_t {
    sf : lr11xx_radio_lora_sf_t,
    bw : lr11xx_radio_lora_bw_t,
    cr : lr11xx_radio_lora_cr_t,
    ldro : u8
}

pub struct lr11xx_radio_pkt_params_lora_t
{
    preamble_len_in_symb : u16,  // LoRa Preamble length [symbols]
    header_type : lr11xx_radio_lora_pkt_len_modes_t,           // LoRa Header type configuration
    pld_len_in_bytes : u8,      // LoRa Payload length [bytes]
    crc : lr11xx_radio_lora_crc_t,                 // LoRa CRC configuration
    iq : lr11xx_radio_lora_iq_t                    // LoRa IQ configuration
}


#[repr(C)]
pub struct Lr11xxHalContext {
    spi_fd: i32,
    set_nss: Option<extern "C" fn(bool)>,
    set_reset: Option<extern "C" fn(bool)>,
    delay_ms: Option<extern "C" fn(u32)>,
}

pub struct lr11xx_radio_pa_cfg_t{
    pa_sel : lr11xx_radio_pa_selection_t,         // Power Amplifier selection
    pa_reg_supply : lr11xx_radio_pa_reg_supply_t,  // Power Amplifier regulator supply source
    pa_duty_cycle : u8,  // Power Amplifier duty cycle (Default 0x04)
    pa_hp_sel : u8      // Number of slices for HPA (Default 0x07)
}

struct rssi_gain_tune_t {
        g4 : u8,
        g5 : u8,
        g6 : u8,
        g7 : u8,
        g8 : u8,
        g9 : u8,
        g10 : u8,
        g11 : u8,
        g12 : u8,
        g13 : u8,
        g13hp1 : u8,
        g13hp2 : u8,
        g13hp3 : u8,
        g13hp4 : u8,
        g13hp5 : u8,
        g13hp6 : u8,
        g13hp7 : u8
    }  // Used to set gain tune value for RSSI calibration

pub struct lr11xx_radio_rssi_calibration_table_t
{
    
    gain_tune : rssi_gain_tune_t,
    gain_offset : u16  // Used to set gain offset value for RSSI calibration
}




pub struct  context {}

#[link(name = "lr11xx_driver", kind = "static")]
unsafe extern "C" {
    fn adfadfa();
    fn lr11xx_hal_write(context: *const std::ffi::c_void, command : *const u8, command_length : u16, data: *const u8, 
                                data_length : u16) -> lr11xx_hal_status_t;
    fn lr11xx_hal_read(context: *const std::ffi::c_void, command : *const u8, command_length : u16,
                                     data : *mut u8, data_length : u16) -> lr11xx_hal_status_t;
    fn lr11xx_radio_set_pkt_type(context : *const std::ffi::c_void, pkt_type: u8) -> lr11xx_status_t;

    fn lr11xx_radio_set_lora_sync_word(context : *const std::ffi::c_void, sync_word : u8) -> lr11xx_status_t;
    fn fnlr11xx_radio_set_lora_public_network(context : *const std::ffi::c_void, network_type : lr11xx_radio_lora_network_type_t  ) -> lr11xx_status_t;
    fn  lr11xx_radio_set_rx(context : *const std::ffi::c_void, timeout_in_ms : u32) -> lr11xx_status_t;
    fn  lr11xx_radio_set_rx_and_lna_mode(context : *const std::ffi::c_void, timeout_in_ms : u32,
                                                lna_mode : lr11xx_radio_lna_mode_t) -> lr11xx_status_t;
    fn lr11xx_radio_set_rx_with_timeout_in_rtc_step(context : *const std::ffi::c_void, timeout_in_rtc_step : u32) -> lr11xx_status_t;
    fn  lr11xx_radio_set_rx_with_timeout_in_rtc_step_and_lna_mode(context : *const std::ffi::c_void, timeout_in_rtc_step : u32,
                                                                           lna_mode : lr11xx_radio_lna_mode_t) -> lr11xx_status_t;
    fn lr11xx_radio_set_tx(context : *const std::ffi::c_void, timeout_in_ms : u32) -> lr11xx_status_t;
    fn lr11xx_radio_set_tx_with_timeout_in_rtc_step(context : *const std::ffi::c_void, timeout_in_rtc_step : u32) -> lr11xx_status_t;
    fn lr11xx_radio_set_rf_freq(context : *const std::ffi::c_void, freq_in_hz : u32) -> lr11xx_status_t;
    fn lr11xx_radio_auto_tx_rx(context : *const std::ffi::c_void, delay : u32,
                                        intermediary_mode : lr11xx_radio_intermediary_mode_t , timeout : u32) -> lr11xx_status_t;
    fn lr11xx_radio_set_lora_mod_params(context : *const std::ffi::c_void,
                                                 mod_params : *const lr11xx_radio_mod_params_lora_t) -> lr11xx_status_t;
    fn lr11xx_radio_set_lora_pkt_params(context : *const std::ffi::c_void, pkt_params : *const lr11xx_radio_pkt_params_lora_t) -> lr11xx_status_t;
    fn lr11xx_radio_set_pkt_address(context : *const std::ffi::c_void, node_address : u8, broadcast_address : u8) -> lr11xx_status_t;
    fn lr11xx_radio_set_rx_tx_fallback_mode(context : *const std::ffi::c_void, fallback_mode : lr11xx_radio_fallback_modes_t) -> lr11xx_status_t;
    fn lr11xx_radio_set_rx_duty_cycle(context : *const std::ffi::c_void, rx_period_in_ms : u32, sleep_period_in_ms : u32, mode : lr11xx_radio_rx_duty_cycle_mode_t) -> lr11xx_status_t;
    fn lr11xx_radio_set_rx_duty_cycle_with_timings_in_rtc_step(context : *const std::ffi::c_void, rx_period_in_rtc_step : u32,
                                                                         sleep_period_in_rtc_step : u32, mode : lr11xx_radio_rx_duty_cycle_mode_t) -> lr11xx_status_t;
    fn lr11xx_radio_set_pa_cfg(context : *const std::ffi::c_void, pa_cfg : *const lr11xx_radio_pa_cfg_t) -> lr11xx_status_t;
    fn  lr11xx_radio_stop_timeout_on_preamble(context : *const std::ffi::c_void, stop_timeout_on_preamble : bool) -> lr11xx_status_t;
    fn lr11xx_radio_set_cad(context : *const std::ffi::c_void) -> lr11xx_status_t;
    fn lr11xx_radio_set_tx_cw(context : *const std::ffi::c_void) -> lr11xx_status_t;
    fn lr11xx_radio_set_tx_infinite_preamble(context : *const std::ffi::c_void) -> lr11xx_status_t; //check firmware version
    fn lr11xx_radio_set_lora_sync_timeout(context : *const std::ffi::c_void, nb_symbol : u16) -> lr11xx_status_t; //depends on firmware
    fn lr11xx_radio_cfg_rx_boosted(context : *const std::ffi::c_void, enable_boost_mode : bool) -> lr11xx_status_t;
    fn lr11xx_radio_set_rssi_calibration(context : *const std::ffi::c_void, rssi_cal_table : *const lr11xx_radio_rssi_calibration_table_t) -> lr11xx_status_t;
    fn lr11xx_radio_get_lora_time_on_air_numerator(pkt_p : *const lr11xx_radio_pkt_params_lora_t,
                                                     mod_p : *const lr11xx_radio_mod_params_lora_t) -> u32;
    fn lr11xx_radio_get_lora_bw_in_hz(bw : lr11xx_radio_lora_bw_t) -> u32;
    fn lr11xx_radio_get_lora_time_on_air_in_ms(pkt_p : *const lr11xx_radio_pkt_params_lora_t,
                                                  mod_p : *const lr11xx_radio_mod_params_lora_t) -> u32;
    fn lr11xx_radio_convert_nb_symb_to_mant_exp(nb_symbol : u16, mant : *mut u8, exp : *mut u8) -> u16;
    fn lr11xx_radio_set_lna_mode(context : *const std::ffi::c_void, lna_mode : lr11xx_radio_lna_mode_t) -> lr11xx_status_t;



}

use std::{thread, time::Duration};

extern "C" fn set_nss(level: bool) {
    println!("[HAL] NSS -> {}", if level { "HIGH" } else { "LOW" });
}

extern "C" fn set_reset(level: bool) {
    println!("[HAL] RESET -> {}", if level { "HIGH" } else { "LOW" });
}

extern "C" fn delay_ms(ms: u32) {
    thread::sleep(Duration::from_millis(ms as u64));
}


fn main() {
    // let mut a = Vec::<u8>::new().as_ptr();
    // unsafe {
    //     lr11xx_hal_write(context, a, command_length, data, data_length)
    // }
    let ctx = Lr11xxHalContext {
        spi_fd: 0, // dummy for WSL
        set_nss: Some(set_nss),
        set_reset: Some(set_reset),
        delay_ms: Some(delay_ms),
    };
    unsafe {
        let status = lr11xx_radio_set_tx(&ctx as *const _ as *const c_void, 0);
        println!("Status: {:?}", status as i32);
    }
}




// 8 bit enum instead of string key

//config function
//send function

