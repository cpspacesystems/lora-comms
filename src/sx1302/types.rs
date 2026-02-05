use core::num;
use std::array;

use crate::sx1302::{self, bindings_loragw_hal, error::{AssertFailure, assert_np}, types};

///////////////////////////////////////////
/// All common data types and functions ///
///////////////////////////////////////////


/// Radios present on the SX1302
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Radios {
    /// The radio capable of both receive and transmit
    Radio0RxTx = 0,
    /// The radio capable of only receiving
    Radio1RxOnly = 1,
}

/// Status of a radio
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RadioStatus {
    /// Radio is off, both Rx and Tx modems are stopped
    Off,
    /// Radio is on, currently listening for packets.
    /// Tx operation/packet transmit is avaliable and ready.
    Avaliable,
    /// Radio is currently trasmitting a packet. 
    /// Unable to engage in Rx operation and any additional Tx operation
    /// untill the current packet has finished transmission.   
    Busy,
    /// Radio is on, currently listening for packets,
    /// but Tx modem is off/disabled. No Tx operations avaliable!
    RxOnly,
    /// Unable to get status of the radio or radio in unknown state.
    /// 
    /// This can be due to radio disconnect, or radio not yet configured and started.
    /// Or something has gone terriably wrong and we are all gonaa die.  
    Unknown,
}

/// bandwidth for radio channels
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bandwidth {
    Low125khz = bindings_loragw_hal::BW_125KHZ,
    Mid250khz = bindings_loragw_hal::BW_250KHZ,
    High500khz = bindings_loragw_hal::BW_500KHZ,
    // no BW_UNDEFINED here, the default is explicitly defined here as Low125khz
}

/// error correction level for LoRa packets
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoraCodeRate {
    /// 4 data bits and 1 parity bits for 5 total bits
    CR1 = bindings_loragw_hal::CR_LORA_4_5,
    /// 4 data bits and 2 parity bits for 6 total bits
    CR2 = bindings_loragw_hal::CR_LORA_4_6,
    /// 4 data bits and 3 parity bits for 7 total bits
    CR3 = bindings_loragw_hal::CR_LORA_4_7,
    /// 4 data bits and 4 parity bits for 8 total bits
    CR4 = bindings_loragw_hal::CR_LORA_4_8,
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
        spread_factor: u32,
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


/// a Vec-tor with a fixed capacity (anddd on the stack if it can fit)
#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct FixedVec<T, const N: usize> {
    data: [T; N],
    size: usize,
}
impl<T: std::marker::Copy, const N: usize> FixedVec<T, N> {
    /// creates a new FixedVec
    pub const fn new(default: T) -> Self {
        FixedVec { data: [default; N], size: 0 }
    }
    
    /// push a new T to the end of this FixedVec
    /// 
    /// assert fails when the FixedVec can not take more data
    pub fn push(&mut self, data: T) -> Result<(), AssertFailure> {
        assert_np!(self.size + 1 < self.data.len(), "Can not push data onto FixedVec as FixedVec of size {} is already full.", self.size);
        self.data[self.size] = data;
        self.size += 1;
        Ok(())
    }

    /// conacts a slice of T to the end of this FixedVec
    /// 
    /// assert fails when the FixedVec can not take more data
    pub fn concat_from_slice(&mut self, data: &[T]) -> Result<(), AssertFailure> {
        assert_np!(self.size + data.len() < self.data.len(), "Can not concat data of size {} as FixedVec is already of size {} and will overflow if this slice is pushed.", self.size, data.len());
        self.data[self.size..data.len()].copy_from_slice(data);
        self.size += data.len();
        Ok(())
    }

    /// gets the raw data of this FixedVec 
    pub const fn data(&self) -> [T; N] {
        self.data
    }

    /// gets the number of elenments in this FixedVec
    pub const fn len(&self) -> usize {
        self.size
    }

    /// creates a slice containing all data in this FixedVec
    pub fn as_slice(&self) -> &[T] {
        &self.data[..self.size]
    }
    
    /// creates a mutable slice containing all data in this FixedVec
    pub fn as_slice_mut(&mut self) -> &mut[T] {
        &mut self.data[..self.size]
    }
    
}
impl<'a, T, const N: usize> IntoIterator for &'a FixedVec<T, N> {
    type Item = &'a T;

    type IntoIter = FixedVecIter<'a, T, N>;

    fn into_iter(self) -> Self::IntoIter {
        FixedVecIter { payload: &self, cur: 0 }
    }
}
pub struct FixedVecIter<'a, T, const N: usize> {
    payload: &'a FixedVec<T, N>,
    cur: usize,
}
impl<'a, T, const N: usize> Iterator for FixedVecIter<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur + 1 > self.payload.size {
            None
        } else {
            let v = &self.payload.data[self.cur];
            self.cur += 1;
            Some(v)
        }
    }
}

#[cfg(test)]
mod test_payload {
    use std::{array, sync::atomic::AtomicUsize};

    use crate::sx1302::types::FixedVec;

    #[test]
    fn test_fixed_vec() {
        let mut p1 = FixedVec::new(0);
        assert_eq!(p1.data, [0; 256]);
        assert_eq!(p1.len(), 0);
        assert_eq!(p1.as_slice(), [0;0]);
        assert_eq!(p1.as_slice_mut(), [0;0]);

        p1.concat_from_slice(&[2; 250]).unwrap();
        assert_eq!(p1.len(), 250);
        assert_eq!(p1.as_slice(), [2; 250]);
        assert_eq!(p1.as_slice_mut(), [2; 250]);

        let mut s1: [u8; 256] = [0; 256];
        s1[0..250].copy_from_slice(&[2;250]);
        assert_eq!(p1.data(), s1);
        
        p1.concat_from_slice(&[2; 250]).unwrap_err();
        assert_eq!(p1.len(), 250);
        assert_eq!(p1.as_slice(), [2; 250]);

        let s2 = p1.as_slice_mut();
        s2[0] = 20;
        assert_eq!(p1.as_slice()[0], 20);
        assert_eq!(p1.len(), 250);

        p1.push(30).unwrap();
        assert_eq!(p1.as_slice()[250], 30);
    }

    #[test]
    fn test_fixed_vec_iter() {
        let mut p1: FixedVec<u8, 256> = FixedVec::new(0);
        assert_eq!(p1.into_iter().count(), 0);

        p1.concat_from_slice(&[3; 250]).unwrap();
        assert_eq!(p1.into_iter().count(), 250);
        assert_eq!(p1.into_iter().map(|v| *v as usize).sum::<usize>(), 250*3);
    }

}