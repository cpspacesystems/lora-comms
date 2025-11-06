use std::{error::Error, fmt, io::Read, u8};

use crate::byte_ops::types::{self, DataSectionType, BufferType};
use crate::byte_ops::common::*;

pub fn create_data_section(data_type: types::DataSectionType, mut data: Vec<u8>) -> BufferType {
    match data_type {
        i if types::type_allocations::RESERVED.contains(&i) => {
            panic!("whelp {data_type} is a reserved type, please go call the apporiate functions for this type"); 
        },
        i if types::type_allocations::FLATBUFFERS.contains(&i) => {
            let mut buffer = BufferType::with_capacity(1 + data.len() + 2 + 1);

            buffer.push(data_type.to_le());
            buffer.append(&mut data);
            buffer.extend_from_slice(&compute_crc16(&buffer.as_slice()).to_le_bytes());
            add_boundary(&mut buffer);

            buffer
        }, 
        _ => panic!("whelp, u messed up")
    }
}

#[derive(Debug)]
struct DecodeError(String);
impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Data Section decode failed with: {}", self.0)
    }
}
impl Error for DecodeError {}

fn print_vec_bits(dat: &Vec<u8>) {
    for b in dat {
        print!("{:08b}", b); 
    }
    println!(""); 
}

pub struct DecodedDataSection {
    pub dtype: DataSectionType, 
    pub bytes: BufferType
}
pub fn decode_data_section(data: Vec<u8>) -> Result<DecodedDataSection, Box<dyn Error + Send + Sync>> {
    let l = data.len(); 
    
    print_vec_bits(&data);

    if l < 5 {
        return Err(Box::new(DecodeError("data section too small".to_string())));
    }

    // boundary check
    if data[l-1] != DATA_BOUNDARY {
        return Err(Box::new(DecodeError("No end boundary detected on data section".to_string()))); 
    }

    // CRC check
    let checksum = u16::from_le_bytes(data[l-3..l-1].try_into()?); 
    if checksum != compute_crc16(&data[0..l-3]) {
        return Err(Box::new(DecodeError("CRC no match".to_string())));
    }

    // data parse
    Ok(DecodedDataSection {
        dtype: u8::from_le(data[0]),
        bytes: data[1..l-3].to_vec()
    })
}

