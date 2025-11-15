use std::{error::Error, fmt, io::Read, u8};

use crate::packet::types::{self, DataSectionType, BufferType};
use crate::packet::common::*;
use crate::packet::error::*;

pub fn create_data_section(data_type: types::DataSectionType, mut data: Vec<u8>) -> Result<BufferType, ErrorType> {
    match data_type {
        i if types::type_allocations::RESERVED.contains(&i) => {
            Err(EncodeReservedError(data_type).into())
        },
        i if types::type_allocations::FLATBUFFERS.contains(&i) => {
            let mut buffer = BufferType::with_capacity(1 + data.len() + 2 + 1);

            buffer.push(data_type.to_le());
            buffer.append(&mut data);
            buffer.extend_from_slice(&compute_crc16(&buffer.as_slice()).to_le_bytes());
            add_boundary(&mut buffer);

            Ok(buffer)
        }, 
        _ => Err(EncodeUnknownTypeError(data_type).into())
    }
}

pub struct DecodedDataSection {
    pub dtype: DataSectionType, 
    pub bytes: BufferType
}
pub fn decode_data_section(data: Vec<u8>) -> Result<DecodedDataSection, ErrorType> {
    let l = data.len(); 
    
    debug_print_vec_bits(&data);

    if l < 5 {
        return Err(DecodeTooSmallError().into());
    }

    // boundary check
    if data[l-1] != DATA_BOUNDARY {
        return Err(DecodeBoundaryMissingError("end").into()); 
    }

    // CRC check
    let checksum = u16::from_le_bytes(data[l-3..l-1].try_into()?); 
    if checksum != compute_crc16(&data[0..l-3]) {
        return Err(DecodeCRCNoMatchError().into());
    }

    // data parse
    Ok(DecodedDataSection {
        dtype: u8::from_le(data[0]),
        bytes: data[1..l-3].to_vec()
    })
}

pub mod reserved {
    use crate::packet::types::GPSTime;
    use super::*;

    pub fn create_reset() -> BufferType {
        vec![0b00000000_u8.to_le(), DATA_BOUNDARY]
    }

    pub fn create_indicator_time_gps(time: GPSTime) -> BufferType {
        let mut buffer = BufferType::with_capacity(11); 
        buffer.push(0b00000001_u8.to_le());
        buffer.extend_from_slice(&time.to_le_bytes());
        buffer.push(compute_crc8(&buffer).to_le());
        buffer.push(DATA_BOUNDARY);
        buffer
    }

    pub fn create_indicator_eot() -> BufferType {
        vec![0b00001001_u8.to_le(), 0b00001001_u8.to_le(), 0b00001001_u8.to_le()]
    }

    

}

#[cfg(test)]
mod tests {
    use crate::packet;
    use super::*;

    #[test]
    fn test_data_section() {
        create_data_section(2, vec![]).expect_err("Err expected");
        create_data_section(255, vec![]).expect_err("Err expected");
        
        let data = b"hello world".to_vec(); 
        // assert_eq!(create_data_section(packet::types::flatbuffers::ALITITUDE, data).unwrap(), 
        //     Vec::from(0b000010100110100001100101011011000110110001101111001000000111011101101111011100100110110001100100100011110111001011011011_u128.to_be_bytes())
        // );
    }
}






