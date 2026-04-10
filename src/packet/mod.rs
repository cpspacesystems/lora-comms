

use std::default;

use crate::{common_config, data_handlers::{ConsumerManager, ProducerManager}, errors, network_ids::TypeID, packet::{data_section::{DecodedDataSection, create_data_section, decode_data_sections}, transmission_ctrl::{TSM_CTRL_SIZE, TSMCtrlInfo}}};
use crate::network_ids::TypeIDs;
use crate::common::*;
use crate::errors::*;
use thiserror::Error;

pub mod data_section;
pub mod transmission_ctrl;

pub struct OutgoingFrameBuilder<'a> {
    data_sections: Vec<BufferType>,
    producer_mg: &'a ProducerManager,
}

impl<'a> OutgoingFrameBuilder<'a> {
    pub const PACKET_FIXED_SIZE: usize = TSM_CTRL_SIZE;

    /// initializes a new OutgoingFrameBuilder
    pub fn new(producer_mg: &'a ProducerManager) -> Self {
        Self { data_sections: Vec::new(), producer_mg }
    }

    /// gather data section from producer with id
    /// <br> calling this function 2 times will create 2 data sections from the same producer
    pub fn gather_by_id(&mut self, id: TypeIDs) -> Result<&mut Self, AnyError> {
        let producer = if let Some(p) = self.producer_mg.get_producer_by_id(&id) { p }
        else {
            return Err(errors::GatherUnknownTypeError(id as u8).into());
        };

        let data = create_data_section(id, producer.produce()?)?; 
        self.data_sections.push(data);
        Ok(self)
    }

    /// gather data sections from all avaliable producers
    /// <br> This function will silently skip over any producers that errored while trying to produce
    pub fn gather_all(&mut self) {
        for (id, p) in self.producer_mg.iter_producers() {
            let data = match p.produce() {
                Ok(v) => v,
                Err(_) => continue,
            };

            let ds = match create_data_section(*id, data) {
                Ok(v) => v,
                Err(_) => continue,
            }; 

            self.data_sections.push(ds);
        };
    }

    #[inline]
    fn create_final_packet(tsm: &TSMCtrlInfo, packet: &mut BufferType) {
        let data_sections = std::mem::take(packet); // gets an owned view of packet, which should only contain data sections right now
        let _ = std::mem::replace(packet, [
            tsm.to_wire((Self::PACKET_FIXED_SIZE + data_sections.len()) as u8), 
            data_sections
        ].concat());
    }

    /// builds new frame (into mutiple packets if needed), consumes all data in internal buffer
    ///
    /// this builder can be reused for a new frame 
    pub fn build(&mut self, mut last_tsm: TSMCtrlInfo) -> Vec<BufferType> {        
        let mut packets: Vec<BufferType> = Vec::new();
        'outer: while let Some(mut ds) = self.data_sections.pop() {
            for packet in &mut packets {
                if Self::PACKET_FIXED_SIZE + packet.len() + ds.len() < common_config::MAX_PAYLOAD_SIZE {
                    packet.append(&mut ds);
                    continue 'outer;
                } 
            }

            // no packet big enough for ds found, make new packet
            packets.push(ds);
        }

        if packets.len() == 0 {
            return packets;
        }

        // add tansmission control info
        let end_idx  = packets.len() - 1;
        for packet in &mut packets[0..end_idx] {
            last_tsm.advance(false);
            Self::create_final_packet(&last_tsm, packet);
        }
        // last packet will have a tsm set to eot to end this frame
        last_tsm.advance(true);
        Self::create_final_packet(&last_tsm, &mut packets[end_idx]);

        packets
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct DecodedPacket {
    pub tsm_ctrl: TSMCtrlInfo,
    pub data_sections: Vec<DecodedDataSection>,
}
impl DecodedPacket {
    pub fn sort_packets(packets: &mut Vec<DecodedPacket>, last_tsm: TSMCtrlInfo) {
        packets.sort_by(|a, b| {
            a.tsm_ctrl.num_packets_from_last(last_tsm).cmp(&b.tsm_ctrl.num_packets_from_last(last_tsm))
        });
    }
}

// consumes packet data, calling all corrosponding consumers
pub fn decode_incoming_packet(consumer_mg: &ConsumerManager, header_packet_size: u8, data: BufferType) 
    -> Result<DecodedPacket, AnyError> {
    let tsm_ctrl = TSMCtrlInfo::try_from_wire(&data[0..2], header_packet_size)?;
    
    let ds = decode_data_sections(&consumer_mg, &data[2..])?;
    Ok(DecodedPacket { tsm_ctrl, data_sections: ds})
}

#[cfg(test)]
use bitint::bitint_literals;
#[cfg(test)]
#[bitint_literals]
mod tests {
    use crate::{common_config, network_ids::TypeIDs};

    use super::*;

    #[test]
    fn test_outgoing_packet_builder() {
        let producers = ProducerManager::new();
        let mut builder =  OutgoingFrameBuilder::new(&producers);

        assert!(builder.build(TSMCtrlInfo::default()).is_empty());

        assert!(matches!(builder.gather_by_id(TypeIDs::from_repr(255).unwrap()), Err(e) if e.is::<errors::GatherUnknownTypeError>()));

        assert_eq!(
            builder
                .gather_by_id(TypeIDs::Test1).unwrap()
                .gather_by_id(TypeIDs::from_repr(252).unwrap()).unwrap()
                .gather_by_id(TypeIDs::Test3).unwrap()
                .build(TSMCtrlInfo::default()),
            [[TSMCtrlInfo::new(1_U7, true).to_wire(2+1+3+1+11+1+64),
                vec![0xFD], vec![0x00; 64], vec![0xFC], vec![0x00; 11], vec![0xFB], vec![0x00; 3]].concat()].to_vec() 
        );
    }

    #[test]
    fn test_consume_incoming_packet() {
        let consumers = ConsumerManager::new();

        assert!(matches!(decode_incoming_packet(&consumers, 0, vec![common_config::LORA_REGONATION_CODE ^ 0x0, 0x0, 0xEE]), Err(e) if e.is::<errors::DecodeUnknownTypeError>()));
        assert!(decode_incoming_packet(&consumers, 0, vec![common_config::LORA_REGONATION_CODE ^ 0x0, 0x0]).is_ok());

        assert!(decode_incoming_packet(&consumers, 83, [
            TSMCtrlInfo::new(120_U7, true).to_wire(83),
            vec![0xFB], vec![0x00; 3], vec![0xFC], vec![0x00; 11], vec![0xFD], vec![0x00; 64]
        ].concat()).is_ok());

        assert!(decode_incoming_packet(&consumers, 83, [
            TSMCtrlInfo::new(120_U7, true).to_wire(81),
            vec![0xFB], vec![0x00; 3], vec![0xFC], vec![0x00; 11], vec![0xFD], vec![0x00; 64]
        ].concat()).is_err());
    }
}
