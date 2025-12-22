use thiserror::Error;
use crate::packet::record::ID;

pub type ErrorType = LORAError;

#[derive(Error, Debug, PartialEq)]
pub enum LORAError {
    #[error("Packet decode failed with: {0}")]
    DecodeGenericError(String),
    #[error("type id {0} does not exist in this world, the data is sent to the astroid belts.")]
    DecodeUnknownTypeError(ID),
    #[error("Packet encode failed with: {0}")]
    EncodeGenericError(String),
    #[error("whelps, type id {0} is a reserved type, please go call the apporiate functions for this type")]
    EncodeReservedError(ID),
}
