use std::{cell::RefCell};

use bitvec::vec;

use crate::{common::{BufferType, LoraChannel, LoraCodeRate, SpreadFactor, assert_np}, data_handlers::{DataConsumer, DataProducer}, errors::{self, AnyError}};

pub const NEGOTIATE_SIZE: usize = 2; 
pub const QOS_LINK_STATE_ECC_INC_THRESH: f32 = 0.3;

#[derive(Debug)]
pub struct QOSHandler {
    self_spread_factor: RefCell<SpreadFactor>,
    self_coderate: RefCell<LoraCodeRate>,
    need_sending: RefCell<bool>,
}

impl QOSHandler {
    pub fn new(initial_sf: SpreadFactor, initial_coderate: LoraCodeRate) -> QOSHandler {
        QOSHandler { self_spread_factor: initial_sf.into(), self_coderate: initial_coderate.into(), need_sending: false.into() }
    }

    pub fn compute_and_send(&self, packet_lost_rate: f32) {
        if packet_lost_rate > QOS_LINK_STATE_ECC_INC_THRESH {
            let mut cr = self.self_coderate.borrow_mut();
            if *cr != LoraCodeRate::MAX { 
                *cr = cr.increment();
                self.need_sending.replace(true);
                return;
            }

            let mut sf = self.self_spread_factor.borrow_mut();
            if *sf != SpreadFactor::MAX {
                *sf = sf.increment();
                self.need_sending.replace(true);
            }
            // other wise we are already at max allowed code rate && SF, so nothing we can do
        }
    }

    pub fn get_sf(&self) -> SpreadFactor {
        *self.self_spread_factor.borrow()
    }

    pub fn get_datarate(&self) -> LoraCodeRate {
        *self.self_coderate.borrow()
    }
}

impl DataProducer for QOSHandler {
    fn produce(&self) -> Result<crate::common::BufferType, crate::errors::AnyError> {
        
        let mut data = BufferType::with_capacity(NEGOTIATE_SIZE);
        
        let cur = self.current_state.borrow();
        data.push(to_wire_lora_map(cur.downlink_ch, cur.downlink_coderate));
        data.push(to_wire_lora_map(cur.uplink_ch, cur.uplink_coderate));
        drop(cur);

        self.need_sending.replace(false);
        Ok(data)
    }
    
    fn has_data(&self) -> Result<bool, crate::errors::AnyError> {
        Ok(*self.need_sending.borrow())
    }
}

impl DataConsumer for QOSHandler {
    fn consume(&self, buffer: crate::common::BufferType) -> Result<(), crate::errors::AnyError> {
        assert_np!(buffer.len() == NEGOTIATE_SIZE);

        let (dl_ch, dl_cr) = from_wire_lora_map(buffer[0])?;
        let (up_ch, up_cr) = from_wire_lora_map(buffer[1])?;

        let mut cur = self.current_state.borrow_mut(); 
        cur.downlink_ch = dl_ch;
        cur.downlink_coderate = dl_cr;
        cur.uplink_ch = up_ch;
        cur.uplink_coderate = up_cr;
        drop(cur);

        self.has_new_state.replace(true);
        Ok(())
    }

    fn get_size(&self) -> usize {
        NEGOTIATE_SIZE
    }
}

pub fn to_wire_lora_map(channel: LoraChannel, coderate: LoraCodeRate) -> u8 {
    // ch uses one nybble
    let ch: u8 = match channel {
        LoraChannel::CH0 => 0,
        LoraChannel::CH1 => 1,
        LoraChannel::CH2 => 2,
        LoraChannel::CH3 => 3,
        LoraChannel::CH4 => 4,
        LoraChannel::CH5 => 5,
        LoraChannel::CH6 => 6,
        LoraChannel::CH7 => 7,
        LoraChannel::CH8HBW => 8,
    };

    // cr uses one nybble (technically can save 2 bits if needed)
    // todo: use those 2 bytes to signal weather it's an uplink or downlink change
    let cr: u8 = match coderate {
        LoraCodeRate::CR1 => 1,
        LoraCodeRate::CR2 => 2,
        LoraCodeRate::CR3 => 3,
        LoraCodeRate::CR4 => 4,
    };

    // 0000 0000
    // -ch- -cr-
    (ch << 4) | cr
}

pub fn from_wire_lora_map(data: u8) -> Result<(LoraChannel, LoraCodeRate), errors::InvalidData> {
    let ch = data >> 4; // ch one nybble
    let channel = match ch {
        0 => LoraChannel::CH0,
        1 => LoraChannel::CH1,
        2 => LoraChannel::CH2,
        3 => LoraChannel::CH3,
        4 => LoraChannel::CH4,
        5 => LoraChannel::CH5,
        6 => LoraChannel::CH6,
        7 => LoraChannel::CH7,
        8 => LoraChannel::CH8HBW,
        n => return Err(errors::InvalidData(format!("`{n}` is an invalid lora channel.")))
    };
    
    let cr = data & 0x0F; // cr one nybble
    let coderate = match cr {
        1 => LoraCodeRate::CR1,
        2 => LoraCodeRate::CR2,
        3 => LoraCodeRate::CR3,
        4 => LoraCodeRate::CR4,
        n => return Err(errors::InvalidData(format!("`{n}` is an invalid lora coderate.")))
    };

    Ok((channel, coderate))
}
