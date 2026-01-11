use std::fmt::Debug;

use crate::common::*;
use crate::data_handlers::DataConsumer;
use crate::error::*;
use crate::packet::data_types::ConsumerManager;
use crate::packet::data_types::ID;

/// creates a new data section of type with data
pub fn create_data_section(type_id: ID, mut data: Vec<u8>) -> Result<BufferType, ErrorType> {
    let mut buffer = BufferType::with_capacity(1 + data.len());

    buffer.push(type_id.to_le());
    buffer.append(&mut data);

    Ok(buffer)
}

pub struct DecodedDataSection<'a> {
    pub data_consumer: &'a dyn DataConsumer, 
    pub bytes: BufferType
}
impl Debug for DecodedDataSection<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DecodedDataSection").field("data_consumer", &format!("PTR: {:p}", self.data_consumer)).field("bytes", &self.bytes).finish()
    }
}
impl PartialEq for DecodedDataSection<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}


/// decode data sections into respective types and binary data of content
pub fn decode_data_sections<'a>(consumer_mg: &'a ConsumerManager, data: Vec<u8>) -> Result<Vec<DecodedDataSection<'a>>, ErrorType> {
    let mut res: Vec<DecodedDataSection> = Vec::new();
    let mut head = 0; 
    while head < data.len() {
        // parse and resolve id   
        let data_consumer = if let Some(t) = consumer_mg.get_consumer_by_id(data[head]) { t } 
            else { return Err(LORAError::DecodeUnknownTypeError(data[head])); };
        head += 1;

        // get content
        let size = data_consumer.get_size();
        let bytes = data[head..head + size].to_vec();
        res.push(DecodedDataSection {data_consumer, bytes});
        head += size;
    }

    Ok(res)
}

pub mod reserved {
    use crate::common::GPSTime;
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
    use crate::packet::data_types::id_map::{self};

    use super::*;

    #[test]
    fn test_create_data_section() {        
        let data = b"abc".to_vec();
        let correct: Vec<u8> = [0x14, 0x61, 0x62, 0x63].to_vec();
        assert_eq!(create_data_section(20, data).unwrap(), correct); 
    }

    #[test]
    fn test_decode_data_sections() {
        let consumer_mg = ConsumerManager::init();
        assert!(matches!(decode_data_sections(&consumer_mg, vec![0xFF, 0x01]), Err(LORAError::DecodeUnknownTypeError(_))));

        let d1 = create_data_section(id_map::__test1, b"abc".to_vec()).unwrap();
        assert_eq!(
            decode_data_sections(&consumer_mg, d1).unwrap(),
            vec![DecodedDataSection {bytes: b"abc".to_vec(), data_consumer: consumer_mg.get_consumer_by_id(id_map::__test1).unwrap()}]
        );

        let d1 = create_data_section(id_map::__test1, b"abc".to_vec()).unwrap();
        let d2 = create_data_section(id_map::__test2, b"hello world".to_vec()).unwrap();
        let d3 = [d1.clone(), d2.clone(), d1.clone(), d2.clone()].concat();
        assert_eq!(
            decode_data_sections(&consumer_mg, d3).unwrap(), 
            vec![DecodedDataSection {bytes: b"abc".to_vec(), data_consumer: consumer_mg.get_consumer_by_id(id_map::__test1).unwrap()},
                DecodedDataSection {bytes: b"hello world".to_vec(), data_consumer: consumer_mg.get_consumer_by_id(id_map::__test2).unwrap()},
                DecodedDataSection {bytes: b"abc".to_vec(), data_consumer: consumer_mg.get_consumer_by_id(id_map::__test1).unwrap()},
                DecodedDataSection {bytes: b"hello world".to_vec(), data_consumer: consumer_mg.get_consumer_by_id(id_map::__test2).unwrap()},
            ]
        )
        
    }


}






