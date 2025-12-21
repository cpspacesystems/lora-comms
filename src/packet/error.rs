use thiserror::Error;
use crate::packet::{allocations::DataSectionType};

pub type ErrorType = LORAError;

#[derive(Error, Debug, PartialEq)]
pub enum LORAError {
    #[error("Packet decode failed with: {0}")]
    DecodeGenericError(String),
    #[error("type id {0} does not exist in this world, the data is sent to the astroid belts.")]
    DecodeUnknownTypeError(u8),
    #[error("Packet encode failed with: {0}")]
    EncodeGenericError(String),
    #[error("whelps, type id {} is a reserved type, please go call the apporiate functions for this type", .0.id)]
    EncodeReservedError(DataSectionType),
}
