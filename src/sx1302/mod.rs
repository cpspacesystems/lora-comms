use crate::sx1302::types::{FixedVec, OutgoingPacketConfig, RadioStatus, Radios};
use crate::sx1302::error::{ConfigureError, FailedToGetStatus, FailedToGetTemp, FailedToStart, FailedToStop, FailedToTryReceive, TrySendError};

pub mod radio;
pub mod bindings_loragw_hal;
pub mod conf;
pub mod types;
pub mod error;

pub const MAX_PAYLOAD_SIZE:usize = 256; // must match the payload arr size in loragw_hal
pub const MAX_RAW_PAYLOAD_HOLDER_SIZE: usize = 10; // must fit into u8

pub type Payload = FixedVec<u8, MAX_PAYLOAD_SIZE>;

/// SX1302 Interface
pub trait SX1302 {
    /// creates a new SX1302 radio with configuration
    fn new(config: conf::SX1302Configuration) -> Self;
    /// creates a new SX1302 with a default config using conf::DEFAULT_SX1302_CONFIG
    fn default() -> Self where Self: Sized {
        Self::new(conf::DEFAULT_SX1302_CONFIG)
    }
    /// configures the SX1302 radio
    fn configure(&mut self) -> Result<(), ConfigureError>;
    /// Start the SX1302 radio
    fn start(&mut self) -> Result<(), FailedToStart>;
    /// Stop the SX1302 radio
    fn stop(&mut self) -> Result<(), FailedToStop>;
    /// try receiving packets from sx1302, only valid packets are returned
    fn try_receive(&mut self) -> Result<FixedVec<Payload, MAX_RAW_PAYLOAD_HOLDER_SIZE>, FailedToTryReceive>;
    /// try sending a packet from sx1302, this will fail if Tx is diabled or busy
    fn try_send(&mut self, packet_config: OutgoingPacketConfig, payload: Payload) -> Result<(), TrySendError>;
    /// gets the current status of a radio on the SX1302
    fn get_radio_status(&mut self, radio: Radios) -> Result<RadioStatus, FailedToGetStatus>;
    /// Get the SX1302 temperature in degrees celcius
    fn get_temperature_celcius(&mut self) -> Result<f32, FailedToGetTemp>;
}
