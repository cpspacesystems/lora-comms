use std::{any::Any, cell::{Ref, RefCell, RefMut}, collections::VecDeque, net::{SocketAddr, SocketAddrV4}, num::ParseIntError, sync::{Arc, RwLock, atomic::AtomicUsize}, time};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{common::{Bandwidth, BufferType, LoraCodeRate, SpreadFactor}, errors::AnyError, network::{NetworkRadio, SendError}, packet::{OutgoingPacketConfig, OutgoingPacketModulation, PacketMetadata, ReceivedPacket}};


pub struct SimulatedRadio {
    self_address: String,
    peer_address: String,

    probability_of_loss: f32,
    
    send_finish: time::Instant,
    outgoing_byterate: VecDeque<(time::Instant, usize)>, // send start time, size

    tokio_rt: tokio::runtime::Runtime,
    root_task_handle: Option<tokio::task::AbortHandle>,
    shared: Arc<SharedData>
}

struct SharedData {
    receive_finish: RwLock<time::Instant>,

    peer_addrs: RwLock<Vec<SocketAddr>>,
    
    packet_buffer_len: AtomicUsize,
    packet_buffer: std::sync::Mutex<VecDeque<(time::Instant, ReceivedPacket)>>
}

impl SharedData {
    pub fn new() -> Self {
        SharedData { 
            receive_finish: time::Instant::now().into(),

            peer_addrs: Vec::new().into(),

            packet_buffer_len: 0.into(), 
            packet_buffer: VecDeque::new().into()  
        }
    }

    fn acquire_packet_buffer(&self) -> std::sync::MutexGuard<'_, VecDeque<(time::Instant, ReceivedPacket)>> {
        self.packet_buffer.lock().unwrap_or_else(|e| {
            let mut mutex = e.into_inner();
            *mutex = VecDeque::new();
            self.packet_buffer_len.store(0, std::sync::atomic::Ordering::Relaxed);

            mutex
        })
    }

    pub fn push_packets(&self, packets: Vec<(time::Instant, ReceivedPacket)>) {
        let mut mutex = self.acquire_packet_buffer();

        mutex.extend(packets);
        self.packet_buffer_len.store(mutex.len(), std::sync::atomic::Ordering::Relaxed);
    }

    pub fn get_packets(&self) -> Vec<ReceivedPacket> {
        let buf_len = self.packet_buffer_len.load(std::sync::atomic::Ordering::Relaxed);
        if buf_len == 0 {
            return Vec::new();
        }

        let mut mutex = self.acquire_packet_buffer();
        let buf = std::mem::take(&mut *mutex);

        let now = time::Instant::now();
        let (received, mut pending): (VecDeque<_>, VecDeque<_>) = buf.into_iter()
            .partition(|p| p.0 <= now);

        std::mem::swap(&mut *mutex, &mut pending);

        received.into_iter().map(|x| x.1).collect()
    }
}

impl SimulatedRadio {
    const SIM_HTTP_DTYPE_FORMAT: [&str; 5] = ["freq", "sf", "coderate", "payload_length", "payload_data"];

    pub fn new(self_address: String, peer_address: String) -> Self { 
        Self {
            self_address, peer_address,

            outgoing_byterate: VecDeque::new(),
            probability_of_loss: 0.0,

            send_finish: time::Instant::now(),

            tokio_rt: tokio::runtime::Runtime::new().unwrap(),
            root_task_handle: None,
            shared: SharedData::new().into()
        }
    }

    async fn tokio_root_task(shared: Arc<SharedData>, self_address: String, peer_address: String) -> Result<(), AnyError> {
        // addr resolution
        let listener = tokio::net::TcpListener::bind(self_address).await?;
        {
            let peer_addrs: Vec<SocketAddr> = tokio::net::lookup_host(peer_address).await?.collect();
            *shared.peer_addrs.write().unwrap() = peer_addrs;
        }
        
        // tokio primary io loop
        let mut id: usize = 0;
        loop {
            let (mut stream, inc_addr) = listener.accept().await?;
            id += 1;
            let shread_ref = shared.clone();
            tokio::spawn(async move {
                if let Err(e) = Self::tokio_task_receive(shread_ref, id, &mut stream, inc_addr).await {
                    println!("{}", e);
                };
            });
        }
    }

    async fn tokio_task_receive(shared: Arc<SharedData>, id: usize, stream: &mut tokio::net::TcpStream, inc_addr: std::net::SocketAddr) -> Result<(), AnyError>{
        let mut inc_data = String::new();
        let bytes_read = stream.read_to_string(&mut inc_data).await?;
        // println!("NET #{:05} -> Read {} bytes from {}", id, bytes_read, inc_addr);
        // println!("{}", inc_data);
        
        let mut inc_data_iter = inc_data.split("\r\n");
        
        // Parse Header
        if let Some(s) = inc_data_iter.next() {
            let contents: Vec<&str> = s.split_whitespace().collect();

            if  contents.len() == 3 &&
                contents[0] == "POST" &&
                contents[1] == "/lora_comms_simluation/receive" &&
                contents[2].starts_with("HTTP/") &&
                inc_data_iter.next().is_some_and(|s| s.trim().is_empty())
            {
                return Self::simulator_parse_new_packets(&shared, &id, stream, &mut inc_data_iter).await;
            } else {
                stream.write_all(b"HTTP/1.1 400 Bad Request").await?;
                return Err(format!("NET #{} ERR: Bad header.", id).into());    
            }
        } else {
            stream.write_all(b"HTTP/1.1 400 Bad Request").await?;
            return Err(format!("NET #{} ERR: Missing header.", id).into());
        };
    }
    
    async fn simulator_parse_new_packets(shared: &Arc<SharedData>, id: &usize, stream: &mut tokio::net::TcpStream, lines: &mut std::str::Split<'_, &str>) -> Result<(), AnyError> {
        let now = time::Instant::now();

        // verify format
        if let Some(s) = lines.next() {
            let dformat: Vec<&str> = s.split_terminator(',').map(|s| s.trim()).collect();
            if dformat != Self::SIM_HTTP_DTYPE_FORMAT {
                stream.write_all(b"HTTP/1.1 400 Bad Request").await?;
                return Err(format!("NET #{} ERR: Data format declearation mismatch.", id).into());
            }
        } else {
            stream.write_all(b"HTTP/1.1 400 Bad Request").await?;
            return Err(format!("NET #{} ERR: Missing data format declearation.", id).into());
        }
        
        // parse all data (bascially csv parse)
        let parsed: Vec<(time::Instant, ReceivedPacket)> = {
            let mut receive_finish = shared.receive_finish.write().unwrap_or_else(|e| e.into_inner());
            
            lines
            .map(|s| s
                .split_terminator(',')
                .map(|s| s.trim())
                .collect::<Vec<&str>>()
            )
            // decode into received packets
            .filter_map(|l| 
                if l.len() == Self::SIM_HTTP_DTYPE_FORMAT.len() {
                    // decode dara
                    let p = Some(ReceivedPacket {
                        data: if let Ok(d) = hex::decode(l[4]) { d } else { return None },
                        meta: PacketMetadata {
                            length: if let Ok(n) = l[3].parse() { n } else { return None },
                            snr: 0.0,
                            frequency: if let Ok(n) = l[0].parse() { n } else { return None },
                            sf: if let Some(s) = l[1].parse::<u8>().ok().and_then(|n| n.try_into().ok() ) { s } else { return None },
                            coderate: if let Some(c) = l[2].parse::<u8>().ok().and_then(|n| parse_lora_coderate(n)) { c } else { return None },
                        },
                    });

                    // update receive finish time
                    if let Some(r) = p {
                        let toa = lora_packet_time_on_air(
                            Bandwidth::Low125khz, r.meta.sf, r.meta.coderate, 
                            8, false, false, r.data.len(), &mut 0.0, &mut 0, &mut 0);
                        
                        if let Some(t) = now.checked_add(toa) {
                            if t > *receive_finish {
                                *receive_finish = t;
                            }

                            return Some((t, r));
                        };
                    }

                    None
                } else { None })
            .collect()
        };
        
        // update received packets 
        shared.push_packets(parsed);

        stream.write_all(b"HTTP/1.1 200 Ok").await?;
        Ok(())
    }

    async fn tokio_task_send_packets(shared: Arc<SharedData>, packets: Vec<(OutgoingPacketConfig, BufferType)>) -> Result<(), AnyError> {
        // create http packet
        let mut lines: Vec<String> = vec![
            "POST /lora_comms_simluation/receive HTTP/1.1".into(),
            "".into(),
            Self::SIM_HTTP_DTYPE_FORMAT.join(","),
        ];
        
        lines.reserve(packets.len());
        for (cfg, data) in packets {
            if let OutgoingPacketModulation::LoRa { spread_factor, coderate, ..} = cfg.modulation {
                lines.push([
                    cfg.freq_hz.to_string(),
                    Into::<u8>::into(spread_factor).to_string(), 
                    (coderate as u8).to_string(),
                    data.len().to_string(),
                    hex::encode_upper(data)
                ].join(","));
            };
        }

        // send http packet
        let peer_addrs = {
            shared.peer_addrs.read().unwrap_or_else(|e| e.into_inner()).clone()
        };

        let mut stream = tokio::net::TcpStream::connect(peer_addrs.as_slice()).await?;
        for l in lines {
            stream.write_all(l.as_bytes()).await?;
            stream.write_all(b"\r\n").await?;
        }
        stream.flush().await?;
        stream.shutdown().await?;

        Ok(())
    }
}

impl NetworkRadio for SimulatedRadio {
    type ConfigureError = AnyError;

    fn configure(&mut self) -> Result<(), Self::ConfigureError> {
        Ok(())
    }

    type ReceiveError = AnyError;
    fn try_receive(&mut self) -> Result<Vec<crate::packet::ReceivedPacket>, Self::ReceiveError> {
        let now = time::Instant::now();
        if now < self.send_finish {
            return Err("Last packet not yet fininished sending!".into());
        }

        Ok(self.shared.get_packets())
    }

    type CustomSendError = AnyError;
    fn try_send(&mut self, packet_config: OutgoingPacketConfig, payload: &crate::common::BufferType) -> Result<std::time::Duration, SendError<Self::CustomSendError>> {
        let now = time::Instant::now();
        if now < self.send_finish {
            return Err(SendError::RadioBusy);
        }

        if let OutgoingPacketModulation::LoRa { bandwidth, spread_factor, coderate, preamble_length, no_header, .. } = packet_config.modulation { 
            let toa = lora_packet_time_on_air(bandwidth, spread_factor, coderate, 
                preamble_length, no_header, false, payload.len(), 
                &mut 0_f64, &mut 0, &mut 0
            );
            
            // calculate and set statstics 
            self.outgoing_byterate.push_back((now, payload.len()));
            self.send_finish = now.checked_add(toa).expect("Simulation ran too long to point of time overflow."); 

            // send packet
            self.tokio_rt.spawn(Self::tokio_task_send_packets(self.shared.clone(), vec![(packet_config, payload.clone())]));

            // update bitrate figure
            while self.outgoing_byterate.pop_front_if(|stat| 
                self.send_finish.saturating_duration_since(stat.0) > time::Duration::from_secs(10)
                ).is_some() 
            {}
            
            let bytes: usize = self.outgoing_byterate.iter()
                .map(|stat| stat.1)
                .sum();
            println!("Outbound bitrate: {:.2} kbps per second.", (bytes * 8) as f64/1000.0/10.0);

            Ok(toa)
        } else {
            Err("Can not send packet! Packet modulation is not set to LoRa.".into())
        }
    }

    fn start(&mut self) -> Result<(), crate::errors::AnyError> {
        self.stop()?;

        self.root_task_handle = Some(self.tokio_rt.spawn(
            Self::tokio_root_task(self.shared.clone(), self.self_address.clone(), self.peer_address.clone())
        ).abort_handle());

        Ok(())
    }

    fn stop(&mut self) -> Result<(), crate::errors::AnyError> {
        if let Some(h) = &self.root_task_handle {
            h.abort();
        }

        Ok(())
    }

    fn is_currently_receiving(&mut self) -> Result<bool, crate::errors::AnyError> {
        let ft = self.shared.receive_finish.read().unwrap_or_else(|e| e.into_inner());
        
        Ok(time::Instant::now() < *ft)
    }
}

fn parse_lora_coderate(cr: u8) -> Option<LoraCodeRate> {
    match cr {
        1 => Some(LoraCodeRate::CR1),
        2 => Some(LoraCodeRate::CR2),
        3 => Some(LoraCodeRate::CR3),
        4 => Some(LoraCodeRate::CR4),
        _ => None
    }
}

/// Calculate the time on air of a LoRa packet
/// # Parameters 
/// - `band_width` packet bandwidth
/// - `spread_factor` packet spreading factor
/// - `coderate` packet coding rate
/// - `n_symbol_preamble` packet preamble length (number of symbols)
/// - `no_header` true if packet has no header
/// - `no_crc` true if packet has no CRC
/// - `size` packet size in bytes
/// - `nb_symbols` pointer to return the total number of symbols in packet
/// - `nb_symbols_payload` pointer to return the number of symbols in packet payload
/// - `t_symbol_us` pointer to return the duration of a symbol in microseconds
// translation of C function lora_packet_time_on_air from SX1302 HAl. File: /libloragw/src/loragw_hal.c 
// At git https://github.com/Lora-net/sx1302_hal; commit 4b42025d1751e04632c0b04160e0d29dbbb222a5; tag V2.1.0
fn lora_packet_time_on_air(
    band_width: Bandwidth, spread_factor: SpreadFactor, coderate: LoraCodeRate, n_symbol_preamble: u16,
    no_header: bool, no_crc: bool, size: usize,
    out_nb_symbols: &mut f64, out_nb_symbols_payload: &mut u32, out_t_symbol_us: &mut u16
) -> std::time::Duration {
    let (h, de, n_bit_crc);
    let t_symbol_us;
    let n_symbol;
    let (toa_us, n_symbol_payload );

    let sf = spread_factor as u8 as usize;
    let cr = coderate as u8;

    /* Get bandwidth 125KHz divider*/
    let bw_pow= match band_width {
        Bandwidth::Low125khz => 1,
        Bandwidth::Mid250khz => 2,
        Bandwidth::High500khz => 3,
    };

    /* Duration of 1 symbol */
    t_symbol_us = (1 << sf) * 8 / bw_pow; /* 2^SF / BW , in microseconds */

    /* Packet parameters */
    h = if no_header == false { 1 } else { 0 }; /* header is always enabled, except for beacons */
    de = if sf >= 11 { 1 } else { 0 }; /* Low datarate optimization enabled for SF11 and SF12 */
    n_bit_crc = if no_crc == false { 16 } else { 0 };

    /* Number of symbols in the payload */
    n_symbol_payload = f64::ceil(
    f64::max(( 8 * size + n_bit_crc - 4*sf + (if sf >= 7 { 8 } else { 0 }) + 20*h ) as f64, 0.0) 
        /
        ( 4 * (sf - 2*de)) as f64 /* Explicitely cast to double to keep precision of the division */ 
    ) * ( cr + 4 ) as f64; 

    /* number of symbols in packet */
    n_symbol = n_symbol_preamble as f64 + if sf >= 7 { 4.25 } else { 6.25 } + 8.0 + n_symbol_payload;

    /* Duration of packet in microseconds */
    toa_us = n_symbol * t_symbol_us as f64;

    /* Return details if required */
    *out_nb_symbols = n_symbol;
    *out_nb_symbols_payload = n_symbol_payload as u32;
    *out_t_symbol_us = t_symbol_us;

    return std::time::Duration::from_micros(toa_us as u64);
}