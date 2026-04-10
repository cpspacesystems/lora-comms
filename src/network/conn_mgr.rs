use std::{collections::VecDeque, io::SeekFrom, rc::Rc, time};

use crate::{common::{BufferType, LoraChannel, LoraCodeRate}, common_config::{PACKET_LOST_CALC_INTERVAL, UPLINK_TRANSMIT_BEGIN_PERIOD, UPLINK_TRANSMIT_TIMEOUT_PERIOD}, data_handlers::{self, ConsumerManager, ProducerManager, r_negotiate::NegotiatedState}, errors::AnyError, network_ids::{self, TypeIDs}, packet::{self, DecodedPacket, OutgoingFrameBuilder, transmission_ctrl::TSMCtrlInfo}};

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
    LOST,
    SEARCHING,
    UPLINKING,
    DOWNLINKING,
}

pub struct RadioConnectionManager<'a> {
    is_downlink: bool,

    consumer_mgmt: &'a ConsumerManager,
    producer_mgmt: &'a ProducerManager,
    frame_builder: OutgoingFrameBuilder<'a>,
    current_status: RadioConnectionStatus,
    
    negotiated_state: NegotiatedState,
    pending_state: Option<NegotiatedState>, 

    last_tsm: TSMCtrlInfo,
    last_transmit_end_time: time::Instant,

    negotiate_handler: Rc<data_handlers::r_negotiate::NegotiateHandler>,

    stats: ConnectionStatistics, 
    recent_packets_losts: VecDeque<TimestampedData<u8>>, // u8 respresent number of packets lost
    recent_packets_received: VecDeque<TimestampedData<usize>>, // u8 respresent packet sizes
}

impl<'a> RadioConnectionManager<'a> {
    pub fn new_uplink(
        consumer_mgmt: &'a ConsumerManager, producer_mgmt: &'a ProducerManager,
        negotiate_handler: Rc<data_handlers::r_negotiate::NegotiateHandler>,
    ) -> RadioConnectionManager<'a> {
        Self::new(false, consumer_mgmt, producer_mgmt, negotiate_handler)
    }

    pub fn new_downlink(
        consumer_mgmt: &'a ConsumerManager, producer_mgmt: &'a ProducerManager,
        negotiate_handler: Rc<data_handlers::r_negotiate::NegotiateHandler>,
    ) -> RadioConnectionManager<'a> {
        Self::new(true, consumer_mgmt, producer_mgmt, negotiate_handler)
    }

    pub fn new(is_downlink: bool,
        consumer_mgmt: &'a ConsumerManager, producer_mgmt: &'a ProducerManager,
        negotiate_handler: Rc<data_handlers::r_negotiate::NegotiateHandler>,
    ) -> RadioConnectionManager<'a> {

        RadioConnectionManager { 
            is_downlink, 

            consumer_mgmt, 
            producer_mgmt,
            frame_builder: OutgoingFrameBuilder::new(producer_mgmt),

            current_status: RadioConnectionStatus::LOST, 
            negotiated_state: negotiate_handler.get_state(), 
            pending_state: None,

            last_tsm: TSMCtrlInfo::default(),
            last_transmit_end_time: time::Instant::now(),

            negotiate_handler: negotiate_handler,

            stats: ConnectionStatistics::default(),
            recent_packets_losts: VecDeque::with_capacity((4 * PACKET_LOST_CALC_INTERVAL.as_secs()).try_into().unwrap_or(usize::MAX)),
            recent_packets_received: VecDeque::with_capacity((4 * PACKET_LOST_CALC_INTERVAL.as_secs()).try_into().unwrap_or(usize::MAX))
        }
    }

    pub fn get_statistics(&self) -> ConnectionStatistics {
        let recent_packets_received: usize = self.recent_packets_received.len();
        let recent_packets_lost: usize = self.recent_packets_losts.iter()
            .map(|v| *v.data() as usize)
            .sum()
        ;
        let recent_data_received: usize = self.recent_packets_received.iter()
            .map(|v| *v.data() as usize)
            .sum()
        ;

        ConnectionStatistics { 
            recent_packet_lost_rate: (recent_packets_lost as f64 / recent_packets_received as f64) as f32, 
            recent_data_rate: recent_data_received.try_into().unwrap_or(u64::MAX) / PACKET_LOST_CALC_INTERVAL.as_secs(), 
            .. self.stats 
        }
    }

    pub fn negotiate(&mut self, 
        downlink_ch: Option<LoraChannel>, downlink_cr: Option<LoraCodeRate>, 
        uplink_ch: Option<LoraChannel>, uplink_cr: Option<LoraCodeRate>,
        _effective_immediately: Option<bool>
    ) {
        let state = NegotiatedState {
            downlink_ch: downlink_ch.unwrap_or(self.negotiated_state.downlink_ch),
            downlink_coderate: downlink_cr.unwrap_or(self.negotiated_state.downlink_coderate),
            uplink_ch: uplink_ch.unwrap_or(self.negotiated_state.uplink_ch),
            uplink_coderate: uplink_cr.unwrap_or(self.negotiated_state.uplink_coderate),
        };

        self.negotiate_handler.send_negotiate(state);
        self.pending_state = Some(state);
        // TODO implenment effective immediate negotiate 
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

    fn update_protocol_state(&mut self, packets_lost: u8) {
        if packets_lost != 0 && self.pending_state.is_some() {
            // reset pending state if we experienced a packet lost since we last sent out a pending state
            self.pending_state = None; 
        }

        // handle negotiations if there are negotiations 
        if self.negotiate_handler.has_new_state() {
            let s = self.negotiate_handler.get_state();
            // we have a pending state waiting to be confirmed and it's confirmed successfully
            if let Some(p) = self.pending_state && s == p {
                self.negotiated_state = s;  // use new negotiated state
                self.pending_state = None;
            } else { // new negotiate request or failed to confirm due to requesting a different negotiate 
                self.negotiate_handler.send_negotiate(s); // send confirm negotiate responding to request 
                self.pending_state = Some(s); // expect next packet to contain confirm negotiate
            }
        // didn't receive a negotiate when we expected one
        } else if self.pending_state.is_some() {
            self.pending_state = None;
        }
    }

    fn construct_frame(&mut self) -> Vec<BufferType> {
        self.frame_builder.gather_all();
        self.frame_builder.build(self.last_tsm)
    }

    pub fn update_transmit_finish(&mut self) {
        self.last_transmit_end_time = time::Instant::now();
    }

    // expects to be called as soon an inbound packet is received
    // expects to not be called before all outbound packets have been sent
    pub fn update(&mut self, busy_receive: bool, mut received_packets: Vec<DecodedPacket>) -> Vec<BufferType> {
        // sort decoded packets by frame number 
        DecodedPacket::sort_packets(&mut received_packets, self.last_tsm);

        let now = time::Instant::now();
        for packet in received_packets {
            let mut data_size = 0;
            let packets_lost = packet.tsm_ctrl.num_packets_from_last(self.last_tsm);
            
            self.last_tsm = packet.tsm_ctrl;
            for ds in packet.data_sections {
                data_size += ds.size();
                if let Err(e) = ds.consume() {
                    println!("Encountered error while consuming data section: {}", e);
                }
            }
            self.update_protocol_state(packets_lost);

            self.update_statistics(now, packets_lost, data_size);
        }

        // peer has ended transmission, we can begin transmitting our data
        if self.last_tsm.is_eot() {
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
                // UPLINK transmitted for too long
                || tslt > UPLINK_TRANSMIT_TIMEOUT_PERIOD
            { 
                return self.construct_frame(); // assume downlink window open, start transmit and stop waiting for receive 
            }
        }

        vec![]
    }

    pub const fn get_status(&self) -> RadioConnectionStatus {
        return self.current_status;
    }

    pub const fn get_negotiated_state(&self) -> NegotiatedState {
        self.negotiated_state
    }

}