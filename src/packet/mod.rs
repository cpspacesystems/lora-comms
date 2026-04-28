

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
        let rc = if let Some(p) = self.producer_mg.get_producer_by_id(&id) { p }
        else {
            return Err(errors::GatherUnknownTypeError(id as u8).into());
        };

        let mut producer = rc.borrow_mut();
        let raw = if let Some(r) = producer.produce()? { r } else {
            return Ok(self);
        };
        if raw.len() != producer.get_size() {
            return Err(errors::GatherUnexpectedSize(producer.get_size(), raw.len()).into());
        }

        let data = create_data_section(id.into(), raw)?; 
        self.data_sections.push(data);
        Ok(self)
    }

    /// gather data sections from all avaliable producers
    /// <br> This function will silently skip over any producers that errored while trying to produce
    pub fn gather_all(&mut self) {
        for (id, rc) in self.producer_mg.iter_producers() {
            let mut p = rc.borrow_mut();
            let data = match p.produce() {
                Ok(Some(v)) => v,
                Ok(None) => continue,
                Err(e) => { 
                    println!("Encountered errors while producing id {}: {}", *id as u8, e);
                    continue;
                } 
            };

            let ds = match create_data_section(*id, data) {
                Ok(v) => v,
                Err(e) => { 
                    println!("Encountered errors while producing id {}: {}", *id as u8, e);
                    continue;
                }
            }; 

            self.data_sections.push(ds);
        };
    }

    #[inline]
    fn create_final_packet(tsm: &TSMCtrlInfo, packet: &mut BufferType) {
        println!("SEND: {}", tsm.get_packet_number());
        let data_sections = std::mem::take(packet); // gets an owned view of packet, which should only contain data sections right now
        let _ = std::mem::replace(packet, [
            tsm.to_wire((Self::PACKET_FIXED_SIZE + data_sections.len()) as u8), 
            data_sections
        ].concat());
    }

    /// builds new frame (into mutiple packets if needed), consumes all data in internal buffer
    ///
    /// this builder can be reused for a new frame 
    pub fn build(&mut self, last_tsm: &mut TSMCtrlInfo) -> Vec<BufferType> {        
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
            packets.push(BufferType::new());
            last_tsm.advance(true);
            Self::create_final_packet(&last_tsm, &mut packets[0]);
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
#[derive(Clone, Copy)]
#[derive(Default)]
pub struct PacketMetadata {
    pub length: usize,
    pub snr: f32,
    pub frequency: u32, 
    pub sf: SpreadFactor,
    pub coderate: LoraCodeRate
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct DecodedPacket {
    pub tsm_ctrl: TSMCtrlInfo,
    pub data_sections: Vec<DecodedDataSection>,
    pub meta: PacketMetadata,
}
impl DecodedPacket {
    pub fn sort_packets(packets: &mut Vec<DecodedPacket>, last_tsm: TSMCtrlInfo) {
        packets.sort_by(|a, b| {
            a.tsm_ctrl.num_packets_from_last(last_tsm).cmp(&b.tsm_ctrl.num_packets_from_last(last_tsm))
        });
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub struct ReceivedPacket {
    pub data: BufferType,
    pub meta: PacketMetadata
}

impl ReceivedPacket {
    pub fn decode(self, consumer_mg: &ConsumerManager) -> Result<DecodedPacket, AnyError> {
        let tsm_ctrl = TSMCtrlInfo::try_from_wire(&self.data[0..2], self.meta.length as u8)?;
    
        let ds = decode_data_sections(&consumer_mg, &self.data[2..])?;
        Ok(DecodedPacket { meta: self.meta, tsm_ctrl, data_sections: ds })
    } 
}

/// Outgoing/Transmit Packet Modulation configuration 
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutgoingPacketModulation {
    /// continous wave
    CW {
        /// frequency offset from Radio Tx frequency
        freq_offset_hz: i8
    },
    /// frequency shift keying
    FSK {
        /// frequency deviation in khz 
        freq_deviation_khz: u8,
        /// baudrate, valid between [500, 250000] bauds
        baudrate: u32,
        /// length of preamble, at least 3, normally 5
        preamble_length: u16,
        /// fixed length packet
        fixed_length: bool,
    },
    /// LoRa spread spectrum
    LoRa {
        /// LoRa modulation/transmit channel bandwidth
        bandwidth: Bandwidth,
        /// LoRa spread factor, valid between SF of [5,12]
        spread_factor: SpreadFactor,
        /// Error correcting level to use for the packet
        coderate: LoraCodeRate,
        /// Is implicit header enabled for this transmission
        no_header: bool,
        /// Invert signal polarity, for orthogonal downlinks (LoRa only) 
        invert_polarity: bool,
        /// length of preamble, at least 6, normally 8
        preamble_length: u16,
    },
}

/// When is the packet sent
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutgoingPacketTiming {
    /// send packet as soon as possible
    Immediate,
    /// send packet when timestamp is reached
    /// 
    /// timestamp or delay in microseconds for to trigger TX start
    Timestamped(u32),
    /// send packet on next GPS/PPS pluse
    GPSTriggered,
}

/// configuration of an packet to be trasmitted
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OutgoingPacketConfig {
    /// The center frequency that the packet will be transmitted at. 
    /// ex 907300000 for packet on 907.3 khz
    pub freq_hz: u32,
    /// the modulation mode used for the packet
    pub modulation: OutgoingPacketModulation,
    /// when the packet is going to be sent
    pub timing: OutgoingPacketTiming,
    /// TX power, in dBm.
    /// Must be match the rf_power of one of the TxGains in Tx Gains configuration
    pub rf_power: i8,
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

        assert_eq!(builder.build(&mut TSMCtrlInfo::default()), &[TSMCtrlInfo::default().advance(true).to_wire(2)]);

        assert!(matches!(builder.gather_by_id(TypeIDs::Test5), Err(e) if e.is::<errors::GatherUnknownTypeError>()));

        assert_eq!(
            builder
                .gather_by_id(TypeIDs::Test1).unwrap()
                .gather_by_id(TypeIDs::Test2).unwrap()
                .gather_by_id(TypeIDs::Test3).unwrap()
                .build(&mut TSMCtrlInfo::default()),
            [[TSMCtrlInfo::new(1_U7, true).to_wire(2+1+3+1+11+1+64),
                vec![0xFD], vec![0x00; 64], vec![0xFC], vec![0x00; 11], vec![0xFB], vec![0x00; 3]].concat()].to_vec() 
        );
    }

    #[test]
    fn test_consume_incoming_packet() {
        let consumers = ConsumerManager::new();

        assert!(matches!(ReceivedPacket { data: vec![common_config::LORA_REGONATION_CODE ^ 0x0, 0x0, 0xEE], meta: PacketMetadata::default() }.decode(&consumers), Err(e) if e.is::<errors::DecodeUnknownTypeError>()));
        assert!(ReceivedPacket { data: vec![common_config::LORA_REGONATION_CODE ^ 0x0, 0x0], meta: PacketMetadata::default() }.decode(&consumers).is_ok());

        assert!(
            ReceivedPacket { data: [
                TSMCtrlInfo::new(120_U7, true).to_wire(83),
                vec![0xFB], vec![0x00; 3], vec![0xFC], vec![0x00; 11], vec![0xFD], vec![0x00; 64]
                ].concat(),
            meta: PacketMetadata { length: 83, ..Default::default() } }
        .decode(&consumers).is_ok());

        
        assert!(
            ReceivedPacket { data: [
                TSMCtrlInfo::new(120_U7, true).to_wire(81),
                vec![0xFB], vec![0x00; 3], vec![0xFC], vec![0x00; 11], vec![0xFD], vec![0x00; 64]
                ].concat(),
            meta: PacketMetadata { length: 83, ..Default::default() } }
        .decode(&consumers).is_err());
    }
}
