use std::{error::Error, fmt};

use crate::packet::types;
pub type ErrorType = Box<dyn Error + Send + Sync>;

#[derive(Debug)]
pub struct DecodeError(pub String);
impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Packet decode failed with: {}", self.0)
    }
}
impl Error for DecodeError {}

#[derive(Debug)]
pub struct EncodeError(pub String);
impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Packet encode failed with: {}", self.0)
    }
}
impl Error for EncodeError {}


#[derive(Debug)]
pub struct EncodeReservedError(pub types::DataSectionType);
impl fmt::Display for EncodeReservedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "whelps, type id {} is a reserved type, please go call the apporiate functions for this type", self.0)
    }
}
impl Error for EncodeReservedError {}


#[derive(Debug)]
pub struct EncodeUnknownTypeError(pub types::DataSectionType);
impl fmt::Display for EncodeUnknownTypeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "type id {} does not exist in this world, the data is sent to the astroid belts.", self.0)
    }
}
impl Error for EncodeUnknownTypeError {}


#[derive(Debug)]
pub struct DecodeTooSmallError();
impl fmt::Display for DecodeTooSmallError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "data section too small, there's nothing to work with here")
    }
}
impl Error for DecodeTooSmallError {}

#[derive(Debug)]
pub struct DecodeBoundaryMissingError<'a>(pub &'a str);
impl fmt::Display for DecodeBoundaryMissingError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "No {} boundary detected on data section", self.0)
    }
}
impl Error for DecodeBoundaryMissingError<'_> {}

#[derive(Debug)]
pub struct DecodeCRCNoMatchError();
impl fmt::Display for DecodeCRCNoMatchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CRC no match, data discarded")
    }
}
impl Error for DecodeCRCNoMatchError {}

// debug functions
pub fn debug_print_vec_bits(dat: &Vec<u8>) {
    for b in dat {
        print!("{:08b}", b); 
    }
    println!(""); 
}
