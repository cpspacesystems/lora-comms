

use std::default;

use crate::packet::{data_section::{create_data_section, decode_data_sections}};
use crate::common::*;
use crate::error::*;

pub mod data_section;
pub mod record;

// ddon't wanna type 5 more words reexport
pub use crate::packet::record::{by_id, by_name};

#[derive(Default)]
pub struct OutgoingPacketBuilder {
    data_sections: Vec<BufferType>
}
impl OutgoingPacketBuilder {
    // gather data section from producer with id
    pub fn gather_by_id(&mut self, record_id: record::ID) -> Result<&mut Self, ErrorType> {
        let dtype = by_id(&record_id);
        let data = create_data_section(&dtype, dtype.produce()?)?; 
        self.data_sections.push(data);
        Ok(self)
    }
    // gather data section from producer with name
    pub fn gather_by_name(&mut self, record_name: &'static str) -> Result<&mut Self, ErrorType> {
        let dtype = by_name(&record_name);
        let data = create_data_section(&dtype, dtype.produce()?)?; 
        self.data_sections.push(data);
        Ok(self)
    }

    // builds new packet, consumes all data in internal buffer
    // this builder can be reused for a new packet 
    pub fn build(&mut self) -> BufferType {
        let data = self.data_sections.concat();
        self.data_sections.clear();
        data
    }
}

// consumes packet data, calling all corrosponding consumers
pub fn consume_incoming_packet(data: BufferType) -> Result<(), ErrorType>{
    let ds = decode_data_sections(data)?;
    for i in ds {
        i.dtype.consume(i.bytes)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outgoing_packet_builder() {
        let mut builder =  OutgoingPacketBuilder::default();

        assert!(matches!(builder.gather_by_id(0), Err(ErrorType::EncodeReservedError(_))));
        assert!(matches!(builder.gather_by_name("ack"), Err(ErrorType::EncodeReservedError(_))));

        assert_eq!(builder.build(), BufferType::new());

        assert_eq!(builder
            .gather_by_name("test1").unwrap()
            .gather_by_id(252).unwrap()
            .gather_by_name("test3").unwrap()
            .build(),
            [vec![0xFB], vec![0x00; 3], vec![0xFC], vec![0x00; 11], vec![0xFD], vec![0x00; 64]].concat() 
        );
    }

    #[test]
    fn test_consume_incoming_packet() {
        assert!(matches!(consume_incoming_packet(vec![0xEE]), Err(ErrorType::DecodeUnknownTypeError(_))));
        assert_eq!(consume_incoming_packet(vec![]), Ok(()));

        assert_eq!(consume_incoming_packet([vec![0xFB], vec![0x00; 3], vec![0xFC], vec![0x00; 11], vec![0xFD], vec![0x00; 64]].concat()), Ok(()));
    }
}
