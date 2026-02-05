use crate::sx1302::types::{FixedVec, OutgoingPacketConfig, RadioStatus, Radios};
use crate::sx1302::error::{ConfigureError, FailedToGetStatus, FailedToGetTemp, FailedToStart, FailedToStop, FailedToTryReceive, TrySendError};

pub mod radio;
pub mod bindings_loragw_hal;
pub mod conf;
pub mod types;
pub mod error;
pub mod backing;

pub const MAX_PAYLOAD_SIZE:usize = 256; // must match the payload arr size in loragw_hal
pub const MAX_RAW_PAYLOAD_HOLDER_SIZE: usize = 10; // must fit into u8

pub type Payload = FixedVec<u8, MAX_PAYLOAD_SIZE>;