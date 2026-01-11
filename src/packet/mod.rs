

use std::default;

use crate::packet::{data_section::{create_data_section, decode_data_sections}, data_types::{ConsumerManager, ProducerManager}};
use crate::common::*;
use crate::error::*;

pub mod data_section;
pub mod data_types;

pub struct OutgoingPacketBuilder<'a> {
    data_sections: Vec<BufferType>,
    producer_mg: &'a ProducerManager,
}

impl<'a> OutgoingPacketBuilder<'a> {
    /// initializes a new OutgoingPackerBuilder
    pub fn new(producer_mg: &'a ProducerManager) -> Self {
        Self { data_sections: Vec::new(), producer_mg }
    }

    /// gather data section from producer with id
    pub fn gather_by_id(&mut self, id: data_types::ID) -> Result<&mut Self, ErrorType> {
        let producer = if let Some(p) = self.producer_mg.get_producer_by_id(id) { p }
        else {
            return Err(ErrorType::GatherUnknownTypeError(id));
        };

        let data = create_data_section(id, producer.produce()?)?; 
        self.data_sections.push(data);
        Ok(self)
    }

    /// builds new packet, consumes all data in internal buffer
    ///
    /// this builder can be reused for a new packet 
    pub fn build(&mut self) -> BufferType {
        let data = self.data_sections.concat();
        self.data_sections.clear();
        data
    }
}

// consumes packet data, calling all corrosponding consumers
pub fn decode_and_consume_incoming_packet(consumer_mg: &ConsumerManager, data: BufferType) -> Result<(), ErrorType>{
    let ds = decode_data_sections(&consumer_mg, data)?;
    for i in ds {
        i.data_consumer.consume(i.bytes)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::packet::data_types::id_map;

    use super::*;

    #[test]
    fn test_outgoing_packet_builder() {
        let producers = ProducerManager::init();
        let mut builder =  OutgoingPacketBuilder::new(&producers);

        assert_eq!(builder.build(), BufferType::new());

        assert!(matches!(builder.gather_by_id(255), Err(ErrorType::GatherUnknownTypeError(255))));

        assert_eq!(builder
            .gather_by_id(id_map::__test1).unwrap()
            .gather_by_id(252).unwrap()
            .gather_by_id(id_map::__test3).unwrap()
            .build(),
            [vec![0xFB], vec![0x00; 3], vec![0xFC], vec![0x00; 11], vec![0xFD], vec![0x00; 64]].concat() 
        );
    }

    #[test]
    fn test_consume_incoming_packet() {
        let consumers = ConsumerManager::init();

        assert!(matches!(decode_and_consume_incoming_packet(&consumers, vec![0xEE]), Err(ErrorType::DecodeUnknownTypeError(_))));
        assert_eq!(decode_and_consume_incoming_packet(&consumers, vec![]), Ok(()));

        assert_eq!(decode_and_consume_incoming_packet(&consumers, [vec![0xFB], vec![0x00; 3], vec![0xFC], vec![0x00; 11], vec![0xFD], vec![0x00; 64]].concat()), Ok(()));
    }
}
