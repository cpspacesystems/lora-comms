use std::{collections::VecDeque, io::SeekFrom, rc::Rc, time};

use crate::{common::{BufferType, LoraChannel, LoraCodeRate}, common_config::{self, PACKET_LOST_CALC_INTERVAL, UPLINK_TRANSMIT_BEGIN_PERIOD, UPLINK_TRANSMIT_TIMEOUT_PERIOD}, data_handlers::{self, ConsumerManager, ProducerManager}, errors::AnyError, network_ids::{self, TypeIDs}, packet::{self, DecodedPacket, OutgoingFrameBuilder, transmission_ctrl::TSMCtrlInfo}};

pub struct TimestampedData<T> {
    time: time::Instant,
    data: T
}
impl<T> TimestampedData<T> {
    pub fn new(time: time::Instant, data: T) -> Self {
        Self { time, data }
    }

    pub fn now(data: T) -> Self {
        Self::new(time::Instant::now(), data)
    }

    pub const fn time(&self) -> time::Instant {
        self.time
    }

    pub const fn data(&self) -> &T {
        &self.data
    }

    pub const fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(PartialEq)]
#[derive(Default)]
pub struct ConnectionStatistics {
    /// total packets received
    pub packets_received: u64,
    /// total packets lost
    pub packets_lost: u64,
    /// percent packet lost over the last PACKET_LOST_CALC_INTERVAL seconds interval
    pub recent_packet_lost_rate: f32,
    /// bits per second, calculated over the last PACKET_LOST_CALC_INTERVAL seconds interval
    pub recent_data_rate: u64, 
}

#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(PartialEq, Eq)]
pub enum RadioConnectionStatus {
    /// Connection is completely lost
    LOST,
    /// Connection is lost to peer, but we are actively transmitting packets and trying to reach them
    SEARCHING,
    /// Connection Established, transmising packets to peer
    TRANSMITTING,
    /// Connection Established, listening for packets from peer
    LISTENING,
}

pub struct RadioConnectionManager<'a> {
    // configuration
    enable_uplink: bool,
    is_downlink: bool,

    consumer_mgmt: &'a ConsumerManager,
    producer_mgmt: &'a ProducerManager,
    frame_builder: OutgoingFrameBuilder<'a>,
    current_status: RadioConnectionStatus,
    
    // transmission control
    last_tsm: TSMCtrlInfo,
    last_transmit_end_time: time::Instant,
    last_packet_received_time: time::Instant,

    // stastistics 
    stats: ConnectionStatistics, 
    recent_packets_losts: VecDeque<TimestampedData<u8>>, // u8 respresent number of packets lost
    recent_packets_received: VecDeque<TimestampedData<usize>>, // u8 respresent packet sizes
}

impl<'a> RadioConnectionManager<'a> {
    pub fn new_uplink(enable_uplink: bool,
        consumer_mgmt: &'a ConsumerManager, producer_mgmt: &'a ProducerManager,
    ) -> RadioConnectionManager<'a> {
        Self::new(false, enable_uplink, consumer_mgmt, producer_mgmt)
    }

    pub fn new_downlink(enable_uplink: bool,
        consumer_mgmt: &'a ConsumerManager, producer_mgmt: &'a ProducerManager,
    ) -> RadioConnectionManager<'a> {
        Self::new(true, enable_uplink, consumer_mgmt, producer_mgmt)
    }

    pub fn new(is_downlink: bool, enable_uplink: bool,
        consumer_mgmt: &'a ConsumerManager, producer_mgmt: &'a ProducerManager,
    ) -> RadioConnectionManager<'a> {

        RadioConnectionManager {
            enable_uplink, 
            is_downlink, 

            consumer_mgmt, 
            producer_mgmt,
            frame_builder: OutgoingFrameBuilder::new(producer_mgmt),

            current_status: RadioConnectionStatus::LOST, 

            last_tsm: TSMCtrlInfo::default(),
            last_transmit_end_time: time::Instant::now(),
            last_packet_received_time: time::Instant::now(),

            stats: ConnectionStatistics::default(),
            recent_packets_losts: VecDeque::with_capacity((4 * PACKET_LOST_CALC_INTERVAL.as_secs()).try_into().unwrap_or(usize::MAX)),
            recent_packets_received: VecDeque::with_capacity((4 * PACKET_LOST_CALC_INTERVAL.as_secs()).try_into().unwrap_or(usize::MAX))
        }
    }

    pub fn get_statistics(&self) -> ConnectionStatistics {
        let recent_packets_received: usize = self.recent_packets_received.len();
        let recent_packets_lost: f64 = self.recent_packets_losts.iter()
            .map(|v| *v.data() as f64)
            .sum()
        ;
        let recent_data_received: usize = self.recent_packets_received.iter()
            .map(|v| v.data())
            .sum()
        ;

        ConnectionStatistics { 
            recent_packet_lost_rate: (recent_packets_lost / recent_packets_received as f64) as f32, 
            recent_data_rate: recent_data_received.try_into().unwrap_or(u64::MAX) / PACKET_LOST_CALC_INTERVAL.as_secs(), 
            .. self.stats 
        }
    }

    #[inline]
    /// compute and update statistics 
    fn update_statistics(&mut self, now: time::Instant, packets_lost: u8, data_size: usize) {
        while let Some(d) = self.recent_packets_losts.front() {
            if now.saturating_duration_since(d.time()) > PACKET_LOST_CALC_INTERVAL {
                self.recent_packets_losts.pop_front();
            } else { 
                break;
            }
        }
        while let Some(d) = self.recent_packets_received.front() {
            if now.saturating_duration_since(d.time()) > PACKET_LOST_CALC_INTERVAL {
                self.recent_packets_received.pop_front();
            } else {
                break;
            }
        }
        if packets_lost != 0 {
            self.stats.packets_lost += packets_lost as u64;
            self.recent_packets_losts.push_back(TimestampedData::new(now, packets_lost));
        }
        self.stats.packets_received += 1;
        self.recent_packets_received.push_back(TimestampedData::new(now, data_size));
    }

    fn construct_frame(&mut self) -> Vec<BufferType> {
        println!("fconstruct");
        self.frame_builder.gather_all();
        let packets = self.frame_builder.build(&mut self.last_tsm);
        packets
    }

    pub fn update_transmit_finish(&mut self) {
        self.last_transmit_end_time = time::Instant::now();
    }

    #[inline]
    fn receive_and_consume_packets(&mut self, mut received_packets: Vec<DecodedPacket>, now: time::Instant) {
        // sort decoded packets by frame number 
        DecodedPacket::sort_packets(&mut received_packets, self.last_tsm);
        for packet in received_packets {
            println!("GOT: {}, {}", packet.tsm_ctrl.get_packet_number(), packet.tsm_ctrl.is_eot());
            let mut data_size = 0;
            let packets_lost = packet.tsm_ctrl.num_packets_from_last(self.last_tsm);

            self.last_tsm = packet.tsm_ctrl;
            for ds in packet.data_sections {
                data_size += ds.size();
                if let Err(e) = ds.consume() {
                    println!("Encountered error while consuming data section: {}", e);
                }
            }

            self.update_statistics(now, packets_lost, data_size);
        }
    }

    // expects to be called as soon an inbound packet is received
    // expects to not be called before all outbound packets have been sent
    pub fn update(&mut self, busy_receive: bool, received_packets: Vec<DecodedPacket>) -> Vec<BufferType> {
        let now = time::Instant::now();
        // receive and process packets
        if received_packets.is_empty() {
            if now.saturating_duration_since(self.last_packet_received_time) >  common_config::CONNECTION_LOST_AFTER_PERIOD {
                self.current_status = RadioConnectionStatus::LOST;
            }
        } else {
            self.last_packet_received_time = now;
            self.receive_and_consume_packets(received_packets, now);

            // peer has ended transmission, we can begin transmitting our data
            if self.last_tsm.is_eot() {
                println!("transmit");
                self.current_status = RadioConnectionStatus::TRANSMITTING;
                return self.construct_frame();
            }
        }
        
        // if uplink not enabled, then downlink will just transmit and uplink will always listen
        if !self.enable_uplink && self.is_downlink {
            self.current_status = RadioConnectionStatus::TRANSMITTING;
            return self.construct_frame(); 
        }

        // we might have missed an EOT.
        // for uplink, we will just skip this transmit window
        // for downlink, we will assume it's our window if the frequency is not busy
        // or simply timeout the wait for uplink
        // 
        // this also means that the downlink will initialize connection first 
        if self.is_downlink {
            let tslt = now.saturating_duration_since(self.last_transmit_end_time);
            if 
                // UPLINK failed to start transmit within transmit begin period
                (!busy_receive && tslt > UPLINK_TRANSMIT_BEGIN_PERIOD)
                // UPLINK transmitted for too long or unrelated transmission taking up freq
                || tslt > UPLINK_TRANSMIT_TIMEOUT_PERIOD
            {
                if RadioConnectionStatus::LOST == self.current_status {
                    self.current_status = RadioConnectionStatus::SEARCHING; 
                } else {
                    self.current_status = RadioConnectionStatus::TRANSMITTING;
                }
                
                return self.construct_frame(); // assume downlink window open, start transmit and stop waiting for receive 
            }
        }

        self.current_status = RadioConnectionStatus::LISTENING;

        vec![] // returning nothing to transmit
    }

    pub const fn get_status(&self) -> RadioConnectionStatus {
        return self.current_status;
    }
}