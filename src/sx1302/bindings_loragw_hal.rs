#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
use std::ffi; 
/*
 / _____)             _              | |
( (____  _____ ____ _| |_ _____  ____| |__
 \____ \| ___ |    (_   _) ___ |/ ___)  _ \
 _____) ) ____| | | || |_| ____( (___| | | |
(______/|_____)_|_|_| \__)_____)\____)_| |_|
  (C)2019 Semtech

Description:
    LoRa concentrator Hardware Abstraction Layer

License: Revised BSD License, see LICENSE.TXT file include in the project
*/


// #ifndef _LORAGW_HAL_H
// #define _LORAGW_HAL_H

// /* -------------------------------------------------------------------------- */
// /* --- DEPENDANCIES --------------------------------------------------------- */

// #include <stdint.h>     /* C99 types */
// #include <stdbool.h>    /* bool type */

// #include "loragw_com.h"

// #include "config.h"     /* library configuration options (dynamically generated) */

// /* -------------------------------------------------------------------------- */
// /* --- PUBLIC MACROS -------------------------------------------------------- */

// #define IS_LORA_BW(bw)          ((bw == BW_125KHZ) || (bw == BW_250KHZ) || (bw == BW_500KHZ))
// #define IS_LORA_DR(dr)          ((dr == DR_LORA_SF5) || (dr == DR_LORA_SF6) || (dr == DR_LORA_SF7) || (dr == DR_LORA_SF8) || (dr == DR_LORA_SF9) || (dr == DR_LORA_SF10) || (dr == DR_LORA_SF11) || (dr == DR_LORA_SF12))
// #define IS_LORA_CR(cr)          ((cr == CR_LORA_4_5) || (cr == CR_LORA_4_6) || (cr == CR_LORA_4_7) || (cr == CR_LORA_4_8))

// #define IS_FSK_BW(bw)           ((bw >= 1) && (bw <= 7))
// #define IS_FSK_DR(dr)           ((dr >= DR_FSK_MIN) && (dr <= DR_FSK_MAX))

// #define IS_TX_MODE(mode)        ((mode == IMMEDIATE) || (mode == TIMESTAMPED) || (mode == ON_GPS))

/* -------------------------------------------------------------------------- */
/* --- PUBLIC CONSTANTS ----------------------------------------------------- */

/* return status code */
pub const LGW_HAL_SUCCESS: ffi::c_int =     0;
pub const LGW_HAL_ERROR: ffi::c_int =       -1;
pub const LGW_LBT_NOT_ALLOWED: ffi::c_int = 1;

/* radio-specific parameters */
pub const LGW_XTAL_FREQU: u32 =     32000000;            /* frequency of the RF reference oscillator */
pub const LGW_RF_CHAIN_NB: usize =    2;                  /* number of RF chains */
// no idea how to convert this to rust, doesn't seem to be used anyways, so
// const LGW_RF_RX_BANDWIDTH {1000000, 1000000}  /* bandwidth of the radios */

/* concentrator chipset-specific parameters */
pub const LGW_IF_CHAIN_NB: usize =    10;      /* number of IF+modem RX chains */
pub const LGW_REF_BW: u32 =         125000;  /* typical bandwidth of data channel */
pub const LGW_MULTI_NB: u16 =       8;       /* number of LoRa 'multi SF' chains */
pub const LGW_MULTI_SF_EN: u8 = 0xFF;    /* bitmask to enable/disable SF for multi-sf correlators  (12 11 10 9 8 7 6 5) */

/* values available for the 'modulation' parameters */
/* NOTE: arbitrary values */
pub const MOD_UNDEFINED: u8 =   0;
pub const MOD_CW: u8 = 0x08;
pub const MOD_LORA: u8 = 0x10;
pub const MOD_FSK: u8 = 0x20;

/* values available for the 'bandwidth' parameters (LoRa & FSK) */
/* NOTE: directly encode FSK RX bandwidth, do not change */
pub const BW_UNDEFINED: u8 = 0;
pub const BW_500KHZ: u8 = 0x06;
pub const BW_250KHZ: u8 = 0x05;
pub const BW_125KHZ: u8 = 0x04;

/* values available for the 'datarate' parameters */
/* NOTE: LoRa values used directly to code SF bitmask in 'multi' modem, do not change */
pub const DR_UNDEFINED: u32 = 0;
pub const DR_LORA_SF5: u32 = 5;
pub const DR_LORA_SF6: u32 = 6;
pub const DR_LORA_SF7: u32 = 7;
pub const DR_LORA_SF8: u32 = 8;
pub const DR_LORA_SF9: u32 = 9;
pub const DR_LORA_SF10: u32 = 10;
pub const DR_LORA_SF11: u32 = 11;
pub const DR_LORA_SF12: u32 = 12;
/* NOTE: for FSK directly use baudrate between 500 bauds and 250 kbauds */
pub const DR_FSK_MIN: u32 = 500;
pub const DR_FSK_MAX: u32 = 250000;

/* values available for the 'coderate' parameters (LoRa only) */
/* NOTE: arbitrary values */

/* CR0 exists but is not recommended, so consider it as invalid */
pub const CR_UNDEFINED: u8 = 0;
pub const CR_LORA_4_5: u8 = 0x01;
pub const CR_LORA_4_6: u8 = 0x02;
pub const CR_LORA_4_7: u8 = 0x03;
pub const CR_LORA_4_8: u8 = 0x04;

/* values available for the 'status' parameter */
/* NOTE: values according to hardware specification */
pub const STAT_UNDEFINED: u8 = 0x00;
pub const STAT_NO_CRC: u8 = 0x01;
pub const STAT_CRC_BAD: u8 = 0x11;
pub const STAT_CRC_OK: u8 = 0x10;

/* values available for the 'tx_mode' parameter */
pub const IMMEDIATE: u8 = 0;
pub const TIMESTAMPED: u8 = 1;
pub const ON_GPS: u8 = 2;

/* values available for 'select' in the status function */
pub const TX_STATUS: u8 = 1;
pub const RX_STATUS: u8 = 2;

/* status code for TX_STATUS */
/* NOTE: arbitrary values */
pub const TX_STATUS_UNKNOWN: u8 = 0;
/* TX modem disabled, it will ignore commands */
pub const TX_OFF: u8 = 1;
/* TX modem is free, ready to receive a command */
pub const TX_FREE: u8 = 2;
/* TX modem is loaded, ready to send the packet after an event and/or delay */
pub const TX_SCHEDULED: u8 = 3;
/* TX modem is emitting */
pub const TX_EMITTING: u8 = 4;

/* status code for RX_STATUS */
/* NOTE: arbitrary values */
pub const RX_STATUS_UNKNOWN: u8 = 0;
/* RX modem is disabled, it will ignore commands  */
pub const RX_OFF: u8 = 1;
/* RX modem is receiving */
pub const RX_ON: u8 = 2;
/* RX is suspended while a TX is ongoing */
pub const RX_SUSPENDED: u8 = 3;

/* Maximum size of Tx gain LUT */
pub const TX_GAIN_LUT_SIZE_MAX: usize = 16;

/* Listen-Before-Talk */
/* Maximum number of LBT channels */
pub const LGW_LBT_CHANNEL_NB_MAX: usize = 16;

/* Spectral Scan */
/* The number of results returned by spectral scan function, to be used for memory allocation */
pub const LGW_SPECTRAL_SCAN_RESULT_SIZE: usize = 33;

/* -------------------------------------------------------------------------- */
/* --- PUBLIC TYPES --------------------------------------------------------- */

#[repr(C)]
// imported from loragw_com.h
pub enum lgw_com_type_t {
    LGW_COM_SPI,
    LGW_COM_USB,
    LGW_COM_UNKNOWN
}

#[repr(C)]
// imported from loragw_com.h
pub enum lgw_com_write_mode_t {
    LGW_COM_WRITE_MODE_SINGLE,
    LGW_COM_WRITE_MODE_BULK,
    LGW_COM_WRITE_MODE_UNKNOWN
}

/**
@enum lgw_radio_type_t
@brief Radio types that can be found on the LoRa Gateway
*/
#[repr(C)]
pub enum lgw_radio_type_t {
    LGW_RADIO_TYPE_NONE,
    LGW_RADIO_TYPE_SX1255,
    LGW_RADIO_TYPE_SX1257,
    LGW_RADIO_TYPE_SX1272,
    LGW_RADIO_TYPE_SX1276,
    LGW_RADIO_TYPE_SX1250
}

/**
@struct lgw_conf_board_s
@brief Configuration structure for board specificities
*/
#[repr(C)]
pub struct lgw_conf_board_s {
    /// Enable ONLY for *public* networks using the LoRa MAC protocol 
    pub lorawan_public: bool,
    /// Index of RF chain which provides clock to concentrator 
    pub clksrc: u8, 
    /// Indicates if the gateway operates in full duplex mode or not 
    pub full_duplex: bool,    
    /// The COMmunication interface (SPI/USB) to connect to the SX1302
    pub com_type: lgw_com_type_t, 
    /// Path to access the COM device to connect to the SX1302
    pub com_path: [ffi::c_char; 64],    
}

/**
@struct lgw_rssi_tcomp_s
@brief Structure containing all coefficients necessary to compute the offset to be applied on RSSI for current temperature
*/
#[repr(C)]
pub struct lgw_rssi_tcomp_s {
    pub coeff_a: ffi::c_float,
    pub coeff_b: ffi::c_float,
    pub coeff_c: ffi::c_float,
    pub coeff_d: ffi::c_float,
    pub coeff_e: ffi::c_float
}

/**
@struct lgw_conf_rxrf_s
@brief Configuration structure for a RF chain
*/
#[repr(C)]
pub struct lgw_conf_rxrf_s {
    /// enable or disable that RF chain
    pub enable: bool,
    /// center frequency of the radio in Hz 
    pub freq_hz: u32, 
    /// Board-specific RSSI correction factor 
    pub rssi_offset: ffi::c_float,
    /// Board-specific RSSI temperature compensation coefficients 
    pub rssi_tcomp: lgw_rssi_tcomp_s,
    /// Radio type for that RF chain (SX1255, SX1257....) 
    pub r#type: lgw_radio_type_t,
    /// enable or disable TX on that RF chain
    pub tx_enable: bool,
    /// Configure the radio in single or differential input mode (SX1250 only)
    pub single_input_mode: bool 
}

/**
@struct lgw_conf_rxif_s
@brief Configuration structure for an IF chain
*/
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct lgw_conf_rxif_s {
    /// enable or disable that IF chain
    pub enable: bool, 
    /// to which RF chain is that IF chain associated 
    pub rf_chain: u8,       
    /// center frequ of the IF chain, relative to RF chain frequency 
    pub freq_hz: i32,       
    /// RX bandwidth, 0 for default  
    pub bandwidth: u8,      
    /// RX datarate, 0 for default 
    pub datarate: u32,      
    /// size of FSK sync word (number of bytes, 0 for default)  
    pub sync_word_size: u8, 
    /// FSK sync word (ALIGN RIGHT, eg. 0xC194C1) 
    pub sync_word: u64,      
    /// LoRa Service implicit header 
    pub implicit_hdr: bool,             
    /// LoRa Service implicit header payload length (number of bytes, 0 for default)   
    pub implicit_payload_length: u8,    
    /// LoRa Service implicit header CRC enable 
    pub implicit_crc_en: bool,         
    /// LoRa Service implicit header coding rate     
    pub implicit_coderate: u8
}

/**
@struct lgw_conf_demod_s
@brief Configuration structure for LoRa/FSK demodulators
*/
#[repr(C)]
pub struct lgw_conf_demod_s {
    /// bitmask to enable spreading-factors for correlators (SF12 - SF5) 
    pub multisf_datarate: u8
}

/**
@struct lgw_pkt_rx_s
@brief Structure containing the metadata of a packet that was received and a pointer to the payload
*/
#[repr(C)]
#[derive(Debug)]
#[derive(Clone, Copy)]
pub struct lgw_pkt_rx_s {
    /// central frequency of the IF chain 
    pub freq_hz: u32,        
    pub freq_offset: i32,
    /// by which IF chain was packet received 
    pub if_chain: u8,       
    /// status of the received packet 
    pub status: u8,         
    /// internal concentrator counter for timestamping, 1 microsecond resolution 
    pub count_us: u32,       
    /// through which RF chain the packet was received 
    pub rf_chain: u8,       
    pub modem_id: u8,
    /// modulation used by the packet 
    pub modulation: u8,     
    /// modulation bandwidth (LoRa only) 
    pub bandwidth: u8,      
    /// RX datarate of the packet (SF for LoRa) 
    pub datarate: u32,      
    /// error-correcting code of the packet (LoRa only)  
    pub coderate: u8,       
    /// average RSSI of the channel in dB 
    pub rssic: ffi::c_float,         
    /// average RSSI of the signal in dB 
    pub rssis: ffi::c_float,          
    /// average packet SNR, in dB (LoRa only) 
    pub snr: ffi::c_float,            
    /// minimum packet SNR, in dB (LoRa only) 
    pub snr_min: ffi::c_float,        
    /// maximum packet SNR, in dB (LoRa only) 
    pub snr_max: ffi::c_float,
    /// CRC that was received in the payload         
    pub crc: u16,            
    /// payload size in bytes 
    pub size: u16,           
    /// buffer containing the payload 
    pub payload: [u8;256],   
    /// a fine timestamp has been received 
    pub ftime_received: bool,
    /// packet fine timestamp (nanoseconds since last PPS)  
    pub ftime: u32
}

/**
@struct lgw_pkt_tx_s
@brief Structure containing the configuration of a packet to send and a pointer to the payload
*/
#[repr(C)]
pub struct lgw_pkt_tx_s {
    /// center frequency of TX 
    pub freq_hz: u32,     
    /// select on what event/time the TX is triggered 
    pub tx_mode: u8,        
    /// timestamp or delay in microseconds for TX trigger 
    pub count_us: u32,      
    /// through which RF chain will the packet be sent  
    pub rf_chain: u8,       
    /// TX power, in dBm 
    pub rf_power: i8,      
    /// modulation to use for the packet 
    pub modulation: u8,    
    /// frequency offset from Radio Tx frequency (CW mode) 
    pub freq_offset: i8,   
    /// modulation bandwidth (LoRa only) 
    pub bandwidth: u8,     
    /// TX datarate (baudrate for FSK, SF for LoRa) 
    pub datarate: u32,      
    /// error-correcting code of the packet (LoRa only) 
    pub coderate: u8,      
    /// invert signal polarity, for orthogonal downlinks (LoRa only) 
    pub invert_pol: bool,    
    /// frequency deviation, in kHz (FSK only) 
    pub f_dev: u8,         
    /// set the preamble length, 0 for default 
    pub preamble: u16,      
    /// if true, do not send a CRC in the packet 
    pub no_crc: bool,        
    /// if true, enable implicit header mode (LoRa), fixed length (FSK) 
    pub no_header: bool,     
    /// payload size in bytes 
    pub size: u16,          
    /// buffer containing the payload 
    pub payload: [u8; 256]  
}

/**
@struct lgw_tx_gain_s
@brief Structure containing all gains of Tx chain
*/
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct lgw_tx_gain_s {
    /// measured TX power at the board connector, in dBm 
    pub rf_power: i8,  
    /// (sx125x) 2 bits: control of the digital gain of SX1302 
    pub dig_gain: u8,  
    /// (sx125x) 2 bits: control of the external PA (SX1302 I/O)
    /// 
    /// (sx1250) 1 bits: enable/disable the external PA (SX1302 I/O)
    pub pa_gain: u8, 
    /// (sx125x) 2 bits: control of the radio DAC 
    pub dac_gain: u8,
    /// (sx125x) 4 bits: control of the radio mixer 
    pub mix_gain: u8,
    /// (sx125x) calibrated I offset 
    pub offset_i: i8,
    /// (sx125x) calibrated Q offset 
    pub offset_q: i8,
    /// (sx1250) 6 bits: control the radio power index to be used for configuration 
    pub pwr_idx: u8
}

/**
@struct lgw_tx_gain_lut_s
@brief Structure defining the Tx gain LUT
*/
#[repr(C)]
pub struct lgw_tx_gain_lut_s {
    /// Array of Tx gain struct 
    pub lut: [lgw_tx_gain_s; TX_GAIN_LUT_SIZE_MAX], 
    /// Number of LUT indexes 
    pub size: u8,                      
}

/**
@struct lgw_conf_debug_s
@brief Configuration structure for debug
*/
#[repr(C)]
pub struct conf_ref_payload_s {
    pub id: u32,
    pub d: [u8;255],
    pub prev_cnt: u32,
}
#[repr(C)]
pub struct lgw_conf_debug_s {
    pub nb_ref_payload: u8,
    pub d: [conf_ref_payload_s;16],
    pub e: [ffi::c_char;128],
}

/**
@enum lgw_ftime_mode_t
@brief Fine timestamping modes
*/
#[repr(C)]
pub enum lgw_ftime_mode_t {
    /// fine timestamps for SF5 -> SF10 
    LGW_FTIME_MODE_HIGH_CAPACITY,  
    /// fine timestamps for SF5 -> SF12 
    LGW_FTIME_MODE_ALL_SF          
}

/**
@struct lgw_conf_ftime_s
@brief Configuration structure for fine timestamping
*/
#[repr(C)]
pub struct lgw_conf_ftime_s {
    /// Enable / Disable fine timestamping 
    pub enable: bool,             
    /// Fine timestamping mode 
    pub mode: lgw_ftime_mode_t,   
}

/**
@enum lgw_lbt_scan_time_t
@brief Radio types that can be found on the LoRa Gateway
*/
#[repr(C)]
pub enum lgw_lbt_scan_time_t {
    LGW_LBT_SCAN_TIME_128_US    = 128,
    LGW_LBT_SCAN_TIME_5000_US   = 5000,
}

/**
@brief Structure containing a Listen-Before-Talk channel configuration
*/
#[repr(C)]
pub struct lgw_conf_chan_lbt_s{
    /// LBT channel frequency 
    pub freq_hz: u32,          
    /// LBT channel bandwidth 
    pub bandwidth: u8,        
    /// LBT channel carrier sense time 
    pub scan_time_us: lgw_lbt_scan_time_t,     
    /// LBT channel transmission duration when allowed 
    pub transmit_time_ms: u16, 
}

/**
@struct lgw_conf_lbt_s
@brief Configuration structure for listen-before-talk
*/
#[repr(C)]
pub struct lgw_conf_lbt_s {
    /// enable or disable LBT 
    pub enable: bool,            
    /// RSSI threshold to detect if channel is busy or not (dBm) 
    pub rssi_target: i8,       
    /// number of LBT channels 
    pub nb_channel: u8,        
    /// LBT channels configuration 
    pub s: [lgw_conf_chan_lbt_s;LGW_LBT_CHANNEL_NB_MAX], 
}

/**
@struct lgw_conf_sx1261_s
@brief Configuration structure for additional SX1261 radio used for LBT and Spectral Scan
*/
#[repr(C)]
pub struct lgw_conf_sx1261_s {
    /// enable or disable SX1261 radio 
    pub enable: bool,            
    /// Path to access the SPI device to connect to the SX1261 (not used for USB com type) 
    pub h: [ffi::c_char;64],      
    /// value to be applied to the sx1261 RSSI value (dBm) 
    pub rssi_offset: i8,       
    /// listen-before-talk configuration 
    pub lbt_conf: lgw_conf_lbt_s,          
}

/**
@struct lgw_context_s
@brief Configuration context shared across modules
*/
#[repr(C)]
pub struct lgw_context_s {
    /* Global context */
    pub is_started: bool,
    pub board_cfg: lgw_conf_board_s,
    /* RX context */
    pub rf_chain_cfg: [lgw_conf_rxrf_s; LGW_RF_CHAIN_NB],
    pub if_chain_cfg: [lgw_conf_rxif_s; LGW_IF_CHAIN_NB],
    pub demod_cfg: lgw_conf_demod_s,
    pub lora_service_cfg: lgw_conf_rxif_s,                       /* LoRa service channel config parameters */
    pub fsk_cfg: lgw_conf_rxif_s,                                /* FSK channel config parameters */
    /* TX context */
    pub t: [lgw_tx_gain_lut_s; LGW_RF_CHAIN_NB],
    /* Misc */
    pub ftime_cfg: lgw_conf_ftime_s,
    pub sx1261_cfg: lgw_conf_sx1261_s,
    /* Debug */
    pub debug_cfg: lgw_conf_debug_s,
}
pub type lgw_context_t = lgw_context_s;

/**
@struct lgw_spectral_scan_status_t
@brief Spectral Scan status
*/
#[repr(C)]
pub enum lgw_spectral_scan_status_t {
    LGW_SPECTRAL_SCAN_STATUS_NONE,
    LGW_SPECTRAL_SCAN_STATUS_ON_GOING,
    LGW_SPECTRAL_SCAN_STATUS_ABORTED,
    LGW_SPECTRAL_SCAN_STATUS_COMPLETED,
    LGW_SPECTRAL_SCAN_STATUS_UNKNOWN
}

/* -------------------------------------------------------------------------- */
/* --- PUBLIC FUNCTIONS PROTOTYPES ------------------------------------------ */
#[cfg(target_family = "unix")]
#[link(name = "loragw", kind="static")]
#[link(name = "tinymt32", kind="static")]
unsafe extern "C" {
    /**
    @brief Configure the gateway board
    @param conf structure containing the configuration parameters
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    pub fn lgw_board_setconf(conf: *mut lgw_conf_board_s) -> ffi::c_int;

    /**
    @brief Configure an RF chain (must configure before start)
    @param rf_chain number of the RF chain to configure [0, LGW_RF_CHAIN_NB - 1]
    @param conf structure containing the configuration parameters
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    pub fn lgw_rxrf_setconf(rf_chain: u8, conf: *mut lgw_conf_rxrf_s) -> ffi::c_int;

    /**
    @brief Configure an IF chain + modem (must configure before start)
    @param if_chain number of the IF chain + modem to configure [0, LGW_IF_CHAIN_NB - 1]
    @param conf structure containing the configuration parameters
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    pub fn lgw_rxif_setconf(if_chain: u8, conf: *mut lgw_conf_rxif_s) -> ffi::c_int;

    /**
    @brief Configure LoRa/FSK demodulators
    @param conf structure containing the configuration parameters
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    pub fn lgw_demod_setconf(conf: *mut lgw_conf_demod_s) -> ffi::c_int;

    /**
    @brief Configure the Tx gain LUT
    @param conf pointer to structure defining the LUT
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    pub fn lgw_txgain_setconf(rf_chain: u8, conf: *mut lgw_tx_gain_lut_s) -> ffi::c_int;

    /**
    @brief Configure the fine timestamping
    @param conf pointer to structure defining the config to be applied
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    pub fn lgw_ftime_setconf(conf: *mut lgw_conf_ftime_s) -> ffi::c_int;

    /*
    @brief Configure the SX1261 radio for LBT/Spectral Scan
    @param pointer to structure defining the config to be applied
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    pub fn lgw_sx1261_setconf(conf: *mut lgw_conf_sx1261_s) -> ffi::c_int;

    /**
    @brief Configure the debug context
    @param conf pointer to structure defining the config to be applied
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    pub fn lgw_debug_setconf(conf: *mut lgw_conf_debug_s) -> ffi::c_int;

    /**
    @brief Connect to the LoRa concentrator, reset it and configure it according to previously set parameters
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    pub fn lgw_start() -> ffi::c_int;

    /**
    @brief Stop the LoRa concentrator and disconnect it
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    pub fn lgw_stop() -> ffi::c_int;

    /**
    @brief A non-blocking function that will fetch up to 'max_pkt' packets from the LoRa concentrator FIFO and data buffer
    @param max_pkt maximum number of packet that must be retrieved (equal to the size of the array of struct)
    @param pkt_data pointer to an array of struct that will receive the packet metadata and payload pointers
    @return LGW_HAL_ERROR id the operation failed, else the number of packets retrieved
    */
    pub fn lgw_receive(max_pkt: u8, pkt_data: *mut lgw_pkt_rx_s) -> ffi::c_int;

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
    pub fn lgw_send(pkt_data: *mut lgw_pkt_tx_s) -> ffi::c_int;

    /**
    @brief Give the the status of different part of the LoRa concentrator
    @param select is used to select what status we want to know
    @param code is used to return the status code
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    pub fn lgw_status(rf_chain: u8, select: u8, code: *mut u8) -> ffi::c_int;

    /**
    @brief Abort a currently scheduled or ongoing TX
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    pub fn lgw_abort_tx(rf_chain: u8) -> ffi::c_int;

    /**
    @brief Return value of internal counter when latest event (eg GPS pulse) was captured
    @param trig_cnt_us pointer to receive timestamp value
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    pub fn lgw_get_trigcnt(trig_cnt_us: *mut u32) -> ffi::c_int;

    /**
    @brief Return instateneous value of internal counter
    @param inst_cnt_us pointer to receive timestamp value
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    pub fn lgw_get_instcnt(inst_cnt_us: *mut u32) -> ffi::c_int;

    /**
    @brief Return the LoRa concentrator EUI
    @param eui pointer to receive eui
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    pub fn lgw_get_eui(eui: *mut u64) -> ffi::c_int;

    /**
    @brief Return the temperature measured by the LoRa concentrator sensor
    @param temperature The temperature measured, in degree celcius
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    pub fn lgw_get_temperature(temperature: *mut ffi::c_float) -> ffi::c_int;

    /**
    @brief Allow user to check the version/options of the library once compiled
    @return pointer on a human-readable null terminated string
    */
    pub fn lgw_version_info() -> *const ffi::c_char;

    /**
    @brief Return time on air of given packet, in milliseconds
    @param packet is a pointer to the packet structure
    @return the packet time on air in milliseconds
    */
    pub fn lgw_time_on_air(packet: *const lgw_pkt_tx_s) -> u32;

    /**
    @brief Start scaning the channel centered on the given frequency
    @param freq_hz channel center frequency
    @param nb_scan number of measures to be done for the scan
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    pub fn lgw_spectral_scan_start(freq_hz: u32, nb_scan: u16) -> ffi::c_int;

    /**
    @brief Get the current scan status
    @param status a pointer to the returned status
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    pub fn lgw_spectral_scan_get_status(status: *mut lgw_spectral_scan_status_t) -> ffi::c_int;

    /**
    @brief Get the channel scan results
    @param levels an array containing the power levels for which the scan results are given
    @param values ar array containing the results of the scan for each power levels
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    pub fn lgw_spectral_scan_get_results(levels_dbm: *mut [i16; LGW_SPECTRAL_SCAN_RESULT_SIZE], results: *mut [u16; LGW_SPECTRAL_SCAN_RESULT_SIZE]) -> ffi::c_int;

    /**
    @brief Abort the current scan
    @return LGW_HAL_ERROR id the operation failed, LGW_HAL_SUCCESS else
    */
    pub fn lgw_spectral_scan_abort() -> ffi::c_int;

}

/* --- EOF ------------------------------------------------------------------ */
