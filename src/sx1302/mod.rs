use core::error;

use thiserror::Error;

use crate::sx1302::types::{OutgoingPacketConfig, RadioStatus, Radios};

pub mod radio;
pub mod bindings_loragw_hal;
pub mod conf;
pub mod types;

const MAX_PAYLOAD_SIZE:usize = 256;

/// SX1302 Interface
pub trait SX1302 {
    /// creates a new SX1302 radio with configuration
    fn new(config: conf::SX1302Configuration) -> Self;
    /// creates a new SX1302 with a default config using conf::DEFAULT_SX1302_CONFIG
    fn default() -> Self where Self: Sized {
        Self::new(conf::DEFAULT_SX1302_CONFIG)
    }
    /// configures the SX1302 radio
    fn configure(&mut self) -> Result<(), SX1302Error>;
    /// Start the SX1302 radio
    fn start(&mut self) -> Result<(), SX1302Error>;
    /// Stop the SX1302 radio
    fn stop(&mut self) -> Result<(), SX1302Error>;
    /// try receiving packets from sx1302, only valid packets are returned
    fn try_receive(&mut self) -> Result<Vec<Vec<u8>>, SX1302Error>;
    /// try sending a packet from sx1302, this will fail if Tx is diabled or busy
    fn try_send(&mut self, packet_config: OutgoingPacketConfig, payload: Vec<u8>) -> Result<(), SX1302Error>;
    /// gets the current status of a radio on the SX1302
    fn get_radio_status(&mut self, radio: Radios) -> Result<RadioStatus, SX1302Error>;
    /// Get the SX1302 temperature in degrees celcius
    fn get_temperature_celcius(&mut self) -> Result<f32, SX1302Error>;
}

/// all application errors
#[derive(Error, Debug, PartialEq)]
pub enum SX1302Error {
    #[error("RX modem is disabled, unable to receive!")]
    RXDisabled,
    #[error("TX is currently going, unable to perform RX operations.")]
    RXSuspended,
    
    #[error("Payload of size {0} is too large to be send. The max payload size is {1}")]
    PayloadTooLarge(usize, usize),
    #[error("The provided rf_power level of {0} for the outgoing packet was not one of the preconfigured power levels with in the Tx Gains configuration!")]
    PacketRfPowerUndefined(i8),
    #[error("The provided preamble length of {0} is too short. The minminum is {1}")]
    PacketPreambleLengthTooShort(u16, u16),
    #[error("The provided spread factor {0} is not within the supported range of [5, 12]")]
    PacketLoraSFUnsupported(u32),
    #[error("The provided baudrate {0} is not within the valid range of [500,250_000] bauds!")]
    PacketFSKInvalidBaudrate(u32),

    #[error("Encountered an error while trying to receive new packets from the SX1302!")]
    TryReceiveFailed,
    #[error("Encountered an error while trying to send packet from the SX1302!")]
    TrySendFailed,
    #[error("Radio1RxTx is currently unable to send any more packets.")]
    RadioBusy,

    #[error("Encountered an error while trying to start the SX1302!")]
    FailedToStart,
    #[error("Encountered an error while trying to stop the SX1302!")]
    FailedToStop,
    #[error("Encountered an error while trying to get SX1302 temperature.")]
    FailedToGetTemp,
    #[error("Encountered an error while trying to get status for SX1302's radio {0} !")]
    FailedToGetStatus(u8),

    #[error("The provided device_com_path `{0}` is too long. The max size of device_com_path supported is {1} bytes, but you passed a string that is {2} bytes!")]
    ConfigCOMPathTooLong(String, usize, usize),
    #[error("The provided device_com_path `{0}` contains a 0-byte(Null Terminator) and can not be parsed.")]
    ConfigUnparsableCOMPath(String),
    #[error("The configuration provided for board configuration is invalid. Check the device section of conf.rs")]
    ConfigBoardSetConfError,
    #[error("The provided demodulator configuration is invalid. Check the demodulator section in conf.rs")]
    ConfigDemodSetConfError,
    #[error("The provided fine timestamp configuration is invalid. Check the fine timestamp section in conf.rs")]
    ConfigFineTimestampSetConfError, /// how tf can someone mess this up, I shrimply do not know, but we do everything to avoid the unwraps from destroying a rocket, right :) 
    #[error("The provided Rx channel configuration for channel {0} is invalid. Check the Rx channel configuration section in conf.rs")]
    ConfigRxIFSetConfError(u8),
    #[error("The provided radio configuration for radio {0} is invalid. Check the radio {0} section in conf.rs")]
    ConfigRxRFSetConfError(u8),
    #[error("The provided Tx Gains configuration for radio {0} is invalid. Check the Tx Gains section for radio {0} in conf.rs")]
    ConfigTxGainSetConfError(u8),
    
    // #[error("The provided configuration is invalid. Check the  section in conf.rs")]
    // Config,
}
