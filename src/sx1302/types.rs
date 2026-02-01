use crate::sx1302::bindings_loragw_hal;

/// Radios present on the SX1302
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Radios {
    /// The radio capable of both receive and transmit
    Radio0RxTx = 0,
    /// The radio capable of only receiving
    Radio1RxOnly = 1,
}

/// Status of a radio
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RadioStatus {
    /// Radio is off, both Rx and Tx modems are stopped
    Off,
    /// Radio is on, currently listening for packets.
    /// Tx operation/packet transmit is avaliable and ready.
    Avaliable,
    /// Radio is currently trasmitting a packet. 
    /// Unable to engage in Rx operation and any additional Tx operation
    /// untill the current packet has finished transmission.   
    Busy,
    /// Radio is on, currently listening for packets,
    /// but Tx modem is off/disabled. No Tx operations avaliable!
    RxOnly,
    /// Unable to get status of the radio or radio in unknown state.
    /// 
    /// This can be due to radio disconnect, or radio not yet configured and started.
    /// Or something has gone terriably wrong and we are all gonaa die.  
    Unknown,
}

/// bandwidth for radio channels
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bandwidth {
    Low125khz = bindings_loragw_hal::BW_125KHZ,
    Mid250khz = bindings_loragw_hal::BW_250KHZ,
    High500khz = bindings_loragw_hal::BW_500KHZ,
    // no BW_UNDEFINED here, the default is explicitly defined here as Low125khz
}

/// error correction level for LoRa packets
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoraCodeRate {
    /// 4 data bits and 1 parity bits for 5 total bits
    CR1 = bindings_loragw_hal::CR_LORA_4_5,
    /// 4 data bits and 2 parity bits for 6 total bits
    CR2 = bindings_loragw_hal::CR_LORA_4_6,
    /// 4 data bits and 3 parity bits for 7 total bits
    CR3 = bindings_loragw_hal::CR_LORA_4_7,
    /// 4 data bits and 4 parity bits for 8 total bits
    CR4 = bindings_loragw_hal::CR_LORA_4_8,
}

/// Outgoing/Transmit Packet Modulation configuration 
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutgoingPacketModulation {
    /// continous wave
    CW {
        /// frequency offset from Radio Tx frequency
        freq_offset_hz: i8
    },
    /// frequency shift keying
    FSK {
        /// frequency deviation in khz 
        freq_deviation_khz: u8,
        /// baudrate, valid between [500, 250000] bauds
        baudrate: u32,
        /// length of preamble, at least 3, normally 5
        preamble_length: u16,
        /// fixed length packet
        fixed_length: bool,
    },
    /// LoRa spread spectrum
    LoRa {
        /// LoRa modulation/transmit channel bandwidth
        bandwidth: Bandwidth,
        /// LoRa spread factor, valid between SF of [5,12]
        spread_factor: u32,
        /// Error correcting level to use for the packet
        coderate: LoraCodeRate,
        /// Is implicit header enabled for this transmission
        no_header: bool,
        /// Invert signal polarity, for orthogonal downlinks (LoRa only) 
        invert_polarity: bool,
        /// length of preamble, at least 6, normally 8
        preamble_length: u16,
    },
}

/// When is the packet sent
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutgoingPacketTiming {
    /// send packet as soon as possible
    Immediate,
    /// send packet when timestamp is reached
    /// 
    /// timestamp or delay in microseconds for to trigger TX start
    Timestamped(u32),
    /// send packet on next GPS/PPS pluse
    GPSTriggered,
}

/// configuration of an packet to be trasmitted
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OutgoingPacketConfig {
    /// The center frequency that the packet will be transmitted at. 
    /// ex 907300000 for packet on 907.3 khz
    pub freq_hz: u32,
    /// the modulation mode used for the packet
    pub modulation: OutgoingPacketModulation,
    /// when the packet is going to be sent
    pub timing: OutgoingPacketTiming,
    /// TX power, in dBm.
    /// Must be match the rf_power of one of the TxGains in Tx Gains configuration
    pub rf_power: i8,
}

