use crate::{common::{BufferType, assert_np}, common_config, errors::{self, AnyError}};

use bitint::{U7, bitint_literals};

pub const TSM_CTRL_SIZE: usize = 2; 

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone, Copy)]
pub struct TSMCtrlInfo {
    packet_number: U7,
    is_eot: bool,
}

#[bitint_literals]
impl Default for TSMCtrlInfo {
    fn default() -> Self {
        Self::new(0_U7, false)
    }
}

#[bitint_literals]
impl TSMCtrlInfo {
    pub fn new(packet_number: U7, is_eot: bool) -> Self {
        TSMCtrlInfo { packet_number, is_eot }
    }

    pub fn advance(&mut self, is_eot: bool) -> &mut Self {
        self.packet_number = self.packet_number.wrapping_add(1_U7);
        self.is_eot = is_eot;
        self
    }
    #[inline]
    pub const fn get_packet_number(&self) -> u8 {
        self.packet_number.to_primitive()
    }
    #[inline]
    pub const fn is_eot(&self) -> bool {
        self.is_eot
    }

    pub const fn num_packets_from_last(&self, last: TSMCtrlInfo) -> u8 {
        let lpn = last.packet_number.to_primitive();
        let cpn = self.packet_number.to_primitive(); 
        if lpn > cpn {
            U7::MAX.to_primitive() - lpn + cpn
        } else {
            cpn - lpn
        }
    }

    pub fn to_wire(&self, packet_size: u8) -> BufferType {
        // encode eot info into info byte
        let info: u8 = (self.packet_number.to_primitive() << 1) | self.is_eot as u8;
        
        let mut buf = BufferType::with_capacity(2);
        buf.push(packet_size ^ common_config::LORA_REGONATION_CODE ^ info);
        buf.push(info);

        buf
    }

    pub fn try_from_wire(data: &[u8], packet_size: u8) -> Result<TSMCtrlInfo, AnyError> {
        assert_np!(data.len() == TSM_CTRL_SIZE);

        // verify packet signature
        let sig = data[0] ^ packet_size ^ data[1];
        if sig == common_config::LORA_REGONATION_CODE {
            // decode TSM CTRL
            Ok(Self::new(
                U7::new_masked(data[1] >> 1), 
                data[1] & 0x1 == 0x1,
            ))
        } else {
            Err(errors::UnrecognizedPacket.into())
        }
    }
}

#[cfg(test)]
#[bitint_literals]
mod tests {
    use super::*;

    #[test]
    fn test_dist() {
        let d1 = TSMCtrlInfo::new(120_U7, true);
        let d2 = TSMCtrlInfo::new(119_U7, true);
        let d3 = TSMCtrlInfo::new(126_U7, true);

        assert_eq!(d1.num_packets_from_last(d1), 0);
        assert_eq!(d1.num_packets_from_last(d2), 1);
        assert_eq!(d1.num_packets_from_last(d3), 121);
    }

    #[test]
    fn test_advance() {
        let mut d1 = TSMCtrlInfo::new(120_U7, true);
        d1.advance(false);
        assert_eq!(d1.is_eot, false);
        assert_eq!(d1.packet_number, 121_U7);

        for _ in 121..U7::MAX.into() { d1.advance(false); };
        assert_eq!(d1.packet_number, U7::MAX);

        d1.advance(false);
        assert_eq!(d1.packet_number, 0_U7);
    }

    #[test]
    fn test_parse() {
        let d1 = TSMCtrlInfo::new(120_U7, true);
        assert_eq!(TSMCtrlInfo::try_from_wire(&d1.to_wire(20), 20).unwrap(), d1);

        let d2 = TSMCtrlInfo::new(120_U7, false);
        assert_ne!(TSMCtrlInfo::try_from_wire(&d2.to_wire(20), 20).unwrap(), d1);

        let d3 = TSMCtrlInfo::new(0_U7, false);
        assert_ne!(TSMCtrlInfo::try_from_wire(&d3.to_wire(20), 20).unwrap(), d1);
        assert_ne!(TSMCtrlInfo::try_from_wire(&d3.to_wire(20), 20).unwrap(), d2);

        assert!(TSMCtrlInfo::try_from_wire(&d1.to_wire(20), 40).is_err());
        assert!(TSMCtrlInfo::try_from_wire(&vec![0x0, 0x0, 0x0], 10).is_err());
        assert!(TSMCtrlInfo::try_from_wire(&vec![0xF4, 0xC3], 10).is_err());
        
        assert!(TSMCtrlInfo::try_from_wire(&vec![common_config::LORA_REGONATION_CODE ^ 0x0 ^ 0x10, 0x1], 0x10).is_err());
        assert_eq!(TSMCtrlInfo::try_from_wire(&vec![common_config::LORA_REGONATION_CODE ^ 0xF1 ^ 0x10, 0xF1], 0x10).unwrap(), TSMCtrlInfo::new(0x78_U7, true));
    }

}