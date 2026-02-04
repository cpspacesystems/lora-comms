use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
#[error("AssertFailure: {0}")]
/// Generic assertion failure due to most likely programmer error.
pub struct AssertFailure(pub String);
/// an assert macro similar to assert!, except an Err(AssertFailure) is returned as an result instead of panic.
macro_rules! assert_no_panic {
    ($cond:expr $(,)?) => {{
        if !($cond) {
            return Err(crate::sx1302::error::AssertFailure(format!("Assertion of {} failed at {}:{}:{}", stringify!($cond), file!(), line!(), column!())));
        }
    }};
    ($cond:expr, $($arg:tt)+) => {{ 
        if !($cond) {
            return Err(crate::sx1302::error::AssertFailure(format!("Assertion of {} failed at {}:{}:{} with message: {}", stringify!($cond), file!(), line!(), column!(), format!($($arg)*))));
        }
    }};
}
pub(super) use assert_no_panic as assert_np;

/// all configuration errors
#[derive(Error, Debug, PartialEq)]
pub enum ConfigureError {
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

#[derive(Error, Debug, PartialEq)]
#[error("Encountered an error while trying to start the SX1302!")]
pub struct FailedToStart;
#[derive(Error, Debug, PartialEq)]
#[error("Encountered an error while trying to stop the SX1302!")]
pub struct FailedToStop;
#[derive(Error, Debug, PartialEq)]
#[error("Encountered an error while trying to get SX1302 temperature.")]
pub struct FailedToGetTemp;
#[derive(Error, Debug, PartialEq)]
#[error("Encountered an error while trying to get status for SX1302's radio {0} !")]
pub struct FailedToGetStatus(pub u8);
#[derive(Error, Debug, PartialEq)]
#[error("Encountered an error while trying to receive new packets from the SX1302!")]
pub struct FailedToTryReceive;

/// all errors for try_send
#[derive(Error, Debug, PartialEq)]
pub enum TrySendError {   
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

    #[error("Radio1RxTx is currently unable to send any more packets.")]
    RadioBusy,
    
    #[error("Encountered an error while trying to send packet from the SX1302!")]
    FailedToTrySend,
}
