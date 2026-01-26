use core::error;

use thiserror::Error;

pub mod radio;
pub mod bindings_loragw_hal;
pub mod conf;


/// SX1302 Interface
pub trait SX1302 {
    fn configure(config: conf::SX1302Configuration) -> Result<(), SX1302Error>;
    fn start() -> Result<(), SX1302Error>;
    fn stop() -> Result<(), SX1302Error>;
    fn try_receive() -> Result<Vec<Vec<u8>>, SX1302Error>;
    fn try_send() -> Result<(), SX1302Error>;
    fn get_temperature_celcius() -> Result<f32, SX1302Error>;
}

/// Common application error type
#[derive(Error, Debug, PartialEq)]
pub enum SX1302Error {
    #[error("RX modem is disabled, unable to receive!")]
    RXDisabled,
    #[error("TX is currently going, unable to perform RX operations.")]
    RXSuspended,
    
    #[error("Encountered an error while trying to receive new packets from the SX1302!")]
    TryReceiveFailed,

    #[error("Encountered an error while trying to start the SX1302!")]
    FailedToStart,
    #[error("Encountered an error while trying to stop the SX1302!")]
    FailedToStop,
    #[error("Encountered an error while trying to get SX1302 temperature.")]
    FailedToGetTemp,

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
    
    #[error("The provided configuration is invalid. Check the  section in conf.rs")]
    Config,
}
