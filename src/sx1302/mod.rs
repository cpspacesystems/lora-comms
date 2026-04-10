use crate::sx1302::types::{OutgoingPacketConfig, RadioStatus, Radios};
use crate::sx1302::error::{ConfigureError, FailedToGetStatus, FailedToGetTemp, FailedToStart, FailedToStop, FailedToTryReceive, TrySendError};

// public modules
pub mod radio;
pub use radio::SX1302;
pub mod conf;
pub mod types;
pub mod error;
pub mod backing;

// internal modules
mod bindings_loragw_hal;
mod testing;
 
pub const MAX_RAW_PAYLOAD_HOLDER_SIZE: usize = 10; // must fit into u8

pub type Payload = Vec<u8>;