
    use std::time;
    use thiserror::Error;
    use crate::{common::BufferType, errors::AnyError, packet::{OutgoingPacketConfig, ReceivedPacket}};



    pub mod conn_mgr;
    pub mod simulated_radio;

    #[derive(Error, Debug, PartialEq)]
    pub enum SendError<O> {
        #[error("Radio is currently busy and unable to send any more packets.")]
        RadioBusy,
        #[error("Send resulted in error: {0}")]
        Other(O),
    }
    impl<O> From<O> for SendError<O> {
        fn from(value: O) -> Self {
            Self::Other(value.into())
        }
    }
    impl From<&str> for SendError<AnyError> {
        fn from(value: &str) -> Self {
            Self::Other(value.into())
        }
    }

    /// Generic Radio trait
    pub trait NetworkRadio {
        type ConfigureError;
        /// configure the raido
        fn configure(&mut self) -> Result<(), Self::ConfigureError>;
        
        type ReceiveError;
        /// receive packets from radio
        fn try_receive(&mut self) -> Result<Vec<ReceivedPacket>, Self::ReceiveError>;
        
        type CustomSendError;
        /// send packets from radio
        fn try_send(&mut self, packet_config: OutgoingPacketConfig, payload: &BufferType) -> Result<time::Duration, SendError<Self::CustomSendError>>;

        /// start the radio
        fn start(&mut self) -> Result<(), AnyError>;
        /// stop the radio
        fn stop(&mut self) -> Result<(), AnyError>;
        /// check if the radio is currently receiving
        fn is_currently_receiving(&mut self) -> Result<bool, AnyError>;
    }