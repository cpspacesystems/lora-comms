use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::write;

use thiserror::Error;
use crate::network_ids::TypeID;
use crate::network_ids::TypeIDs;

pub type AnyError = Box<dyn std::error::Error + Send + Sync>;


#[derive(Error, Debug, PartialEq)]
#[error("AssertFailure: {0}")]
/// Generic assertion failure to respersent some program condition where we can't continue
pub struct AssertFailure(pub String);

#[derive(Error, Debug, PartialEq)]
#[error("You have asked type id {0}, which is more not found than 404.")]
pub struct GatherUnknownTypeError(pub TypeID);

#[derive(Error, Debug, PartialEq)]
#[error("Expected size of {0} data to be produced, but got data size of {1}!")]
pub struct GatherUnexpectedSize(pub usize, pub usize);

#[derive(Error, Debug, PartialEq)]
#[error("Type id {0} does not exist in this world, the data is sent to the astroid belts.")]
pub struct DecodeUnknownTypeError(pub TypeID);

#[derive(Error, Debug, PartialEq)]
#[error("No Reset was requested.")]
pub struct NoResetRequested;

#[derive(Error, Debug, PartialEq)]
#[error("Received Reset with incorrect confirm code.")]
pub struct InvalidReset;

#[derive(Error, Debug, PartialEq)]
#[error("Unable to parse, invalid data: {0}")]
pub struct InvalidData(pub String);

#[derive(Error, Debug, PartialEq)]
#[error("The received data does not match the expected data.")]
pub struct PRNGConsumerUnexpected;

#[derive(Error, Debug, PartialEq)]
#[error("A NULL Producer can not produce.")]
pub struct NULLProducerError;
#[derive(Error, Debug, PartialEq)]
#[error("A NULL Consumer can not consume.")]
pub struct NULLConsumerError;

#[derive(Error, Debug, PartialEq)]
#[error("The altimeter at Zenoh {0} seems to have returned an invalid flatbuffer!")]
pub struct ParseFlatbufferAltimeterError(pub String);

#[derive(Error, Debug, PartialEq)]
#[error("The packet received is not regonized as a packet broadcasted by a CPSS system!")]
pub struct UnrecognizedPacket;