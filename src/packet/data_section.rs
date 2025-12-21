use std::{error::Error, fmt, io::Read, u8};
use zenoh::bytes;

use crate::packet::allocations::{self, DSAllocRecord};
use crate::packet::common::*;
use crate::packet::error::*;

pub fn create_data_section(data_type: DSAllocRecord, mut data: Vec<u8>) -> Result<BufferType, ErrorType> {
    match data_type {
        i if allocations::type_allocations::RESERVED.contains(&i.id) => {
            Err(LORAError::EncodeReservedError(data_type))
        },
        _ => { // everything else is unreserved and thus can be created using this func
            let mut buffer = BufferType::with_capacity(1 + data.len() + 2 + 1);

            buffer.push(data_type.id.to_le());
            buffer.append(&mut data);

            Ok(buffer)
        }, 
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct DecodedDataSection {
    pub dtype: DSAllocRecord, 
    pub bytes: BufferType
}
pub fn decode_data_sections(data: Vec<u8>) -> Result<Vec<DecodedDataSection>, ErrorType> {
    let mut res: Vec<DecodedDataSection> = Vec::new();
    let mut head = 0; 
    while head < data.len() {
        let dtype = if let Some(t) = allocations::try_id(&data[head]) { t } 
            else { return Err(LORAError::DecodeUnknownTypeError(data[head])); };
        head += 1;
        let bytes = data[head..head + dtype.size].to_vec();
        res.push(DecodedDataSection {bytes, dtype});
        head += dtype.size;
    }

    Ok(res)
}

pub mod reserved {
    use crate::packet::common::GPSTime;
    use super::*;

    pub fn create_reset() -> BufferType {
        vec![0x00_u8.to_le()]
    }

    pub fn create_indicator_time_gps(time: GPSTime) -> BufferType {
        let mut buffer = BufferType::with_capacity(9); 
        buffer.push(0x01_u8.to_le());
        buffer.extend_from_slice(&time.to_le_bytes());
        buffer
    }

    pub fn create_indicator_eot() -> BufferType {
        vec![0x09_u8.to_le(), 0x09_u8.to_le(), 0x09_u8.to_le()]
    }
}

#[cfg(test)]
mod tests {
    use crate::packet::{self, allocations::{by_id, by_name}};
    use super::*;

    #[test]
    fn test_create_data_section() {
        assert!(matches!(create_data_section(by_id(&0), vec![]), Err(LORAError::EncodeReservedError(_))));
        
        let data = b"abc".to_vec();
        let correct: Vec<u8> = [0x14, 0x61, 0x62, 0x63].to_vec();
        assert_eq!(create_data_section(DSAllocRecord { id: 20, name: "()", size: 20 }, data).unwrap(), correct); 
    }

    #[test]
    fn test_decode_data_sections() {
        assert!(matches!(decode_data_sections(vec![0xFF, 0x01]), Err(LORAError::DecodeUnknownTypeError(_))));

        let d1 = create_data_section(by_name("test1"), b"abc".to_vec()).unwrap();
        assert_eq!(
            decode_data_sections(d1).unwrap(),
            vec![DecodedDataSection {bytes: b"abc".to_vec(), dtype: by_name("test1")}]
        );

        let d1 = create_data_section(by_name("test1"), b"abc".to_vec()).unwrap();
        let d2 = create_data_section(by_name("test2"), b"hello world".to_vec()).unwrap();
        let d3 = [d1.clone(), d2.clone(), d1.clone(), d2.clone()].concat();
        assert_eq!(
            decode_data_sections(d3).unwrap(), 
            vec![DecodedDataSection {bytes: b"abc".to_vec(), dtype: by_name("test1")},
                DecodedDataSection {bytes: b"hello world".to_vec(), dtype: by_name("test2")},
                DecodedDataSection {bytes: b"abc".to_vec(), dtype: by_name("test1")},
                DecodedDataSection {bytes: b"hello world".to_vec(), dtype: by_name("test2")},
            ]
        )
        
    }


}






