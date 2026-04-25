use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use crate::{common::*, errors};
use crate::data_handlers::{ConsumerManager, DataConsumer};
use crate::errors::*;
use crate::{network_ids::{TypeID, TypeIDs}};

/// creates a new data section of type with data
pub fn create_data_section(type_id: TypeIDs, mut data: Vec<u8>) -> Result<BufferType, AnyError> {
    let mut buffer = BufferType::with_capacity(1 + data.len());

    buffer.push((type_id as u8).to_le());
    buffer.append(&mut data);

    Ok(buffer)
}

pub struct DecodedDataSection {
    id: TypeID,
    data_consumer: Rc<RefCell<dyn DataConsumer>>,
    bytes: BufferType
}
impl DecodedDataSection {
    pub fn consume(self) -> Result<(), AnyError>{
        self.data_consumer.borrow_mut().consume(self.bytes)
    }
    pub const fn size(&self) -> usize {
        self.bytes.len()
    }
}

impl Debug for DecodedDataSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DecodedDataSection").field("data_consumer", &format!("PTR: {:p}", self.data_consumer)).field("bytes", &self.bytes).finish()
    }
}
impl PartialEq for DecodedDataSection {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}


/// decode data sections into respective types and binary data of content
pub fn decode_data_sections<'a>(consumer_mg: &'a ConsumerManager, data: &[u8]) -> Result<Vec<DecodedDataSection>, errors::AnyError> {
    let mut res: Vec<DecodedDataSection> = Vec::new();
    let mut head = 0; 
    while head < data.len() {
        // parse and resolve id
        let id = data[head];   
        let data_consumer = if let Some(t) = consumer_mg.get_consumer_by_u8(data[head]) { t } 
            else { return Err(errors::DecodeUnknownTypeError(data[head]).into()); };
        head += 1;

        // get content
        let size = data_consumer.borrow().get_size();
        if head + size > data.len() {
            return Err(errors::InvalidData(format!("Expected data of size {}, but go data with size of {}!", head + size, size)).into());
        }
        let bytes = data[head..head + size].to_vec();
        res.push(DecodedDataSection {id, data_consumer, bytes});
        head += size;
    }

    Ok(res)
}

#[cfg(test)]
mod tests {
use crate::network_ids::TypeIDs;

    use super::*;

    #[test]
    fn test_create_data_section() {        
        let data = b"abc".to_vec();
        let correct: Vec<u8> = [0xFA, 0x61, 0x62, 0x63].to_vec();
        assert_eq!(create_data_section(TypeIDs::from_repr(250).unwrap(), data).unwrap(), correct); 
    }

    #[test]
    fn test_decode_data_sections() {
        let consumer_mg = ConsumerManager::new();
        assert!(decode_data_sections(&consumer_mg, vec![0xFF, 0x01].as_slice()).is_err());

        let d1 = create_data_section(TypeIDs::Test1, b"abc".to_vec()).unwrap();
        assert_eq!(
            decode_data_sections(&consumer_mg, d1.as_slice()).unwrap(),
            vec![DecodedDataSection {id: TypeIDs::Test1 as u8, bytes: b"abc".to_vec(), data_consumer: consumer_mg.get_consumer_by_id(&TypeIDs::Test1).unwrap()}]
        );

        let d1 = create_data_section(TypeIDs::Test1, b"abc".to_vec()).unwrap();
        let d2 = create_data_section(TypeIDs::Test2, b"hello world".to_vec()).unwrap();
        let d3 = [d1.clone(), d2.clone(), d1.clone(), d2.clone()].concat();
        assert_eq!(
            decode_data_sections(&consumer_mg, d3.as_slice()).unwrap(), 
            vec![DecodedDataSection {id: TypeIDs::Test1 as u8, bytes: b"abc".to_vec(), data_consumer: consumer_mg.get_consumer_by_id(&TypeIDs::Test1).unwrap()},
                DecodedDataSection {id: TypeIDs::Test2 as u8, bytes: b"hello world".to_vec(), data_consumer: consumer_mg.get_consumer_by_id(&TypeIDs::Test2).unwrap()},
                DecodedDataSection {id: TypeIDs::Test1 as u8, bytes: b"abc".to_vec(), data_consumer: consumer_mg.get_consumer_by_id(&TypeIDs::Test1).unwrap()},
                DecodedDataSection {id: TypeIDs::Test2 as u8, bytes: b"hello world".to_vec(), data_consumer: consumer_mg.get_consumer_by_id(&TypeIDs::Test2).unwrap()},
            ]
        )
        
    }


}






