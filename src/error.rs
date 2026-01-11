use thiserror::Error;
use crate::packet::data_types::ID;

pub type ErrorType = LORAError;

#[derive(Error, Debug, PartialEq)]
pub enum LORAError {
    #[error("A NULL Producer can not produce")]
    NULLProducerError,
    #[error("A NULL Consumer can not consume")]
    NULLConsumerError,

    #[error("You have asked type id {0}, which is more not found than 404.")]
    GatherUnknownTypeError(ID),

    #[error("Packet decode failed with: {0}")]
    DecodeGenericError(String),
    #[error("Type id {0} does not exist in this world, the data is sent to the astroid belts.")]
    DecodeUnknownTypeError(ID),
    #[error("Packet encode failed with: {0}")]
    EncodeGenericError(String),

    #[error("the altimeter at Zenoh {0} seems to have returned an invalid flatbuffer!")]
    ParseFlatbufferAltimeterError(String),
}
