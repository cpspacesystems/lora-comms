use std::{rc::Rc, thread::sleep, time::Duration};

use crate::{common::{Bandwidth, BufferType, LoraChannel, LoraCodeRate}, common_config::{ALLOW_CH_CHANGE, DOWNLINK_CH, INITIAL_CODE_RATE, LORA_PREAMBLE_LENGTH, UPLINK_CH}, data_handlers::{ConsumerManager, DataConsumer, ProducerManager, altimeter::Producer, r_negotiate::NegotiatedState}, network::conn_mgr::RadioConnectionManager, packet::{DecodedPacket, OutgoingFrameBuilder, data_section::{DecodedDataSection, decode_data_sections}, transmission_ctrl::TSMCtrlInfo}, sx1302::{SX1302, backing::{DeviceBackingAPI, PhysicalDevice}, conf::{DEFAULT_SX1302_CONFIG, SX1302Configuration}, error::TrySendError, types::{OutgoingPacketConfig, OutgoingPacketModulation, RadioStatus, Radios}}};
use crate::network_ids::TypeIDs;

#[cfg(test)]
use crate::sx1302::backing::unit_test_backing::UnitTestDevice;

mod sx1302;
mod packet;
mod common;
mod errors;
mod publisher;
mod subscriber;
mod data_handlers;
mod network_ids;
mod network;
mod common_config;
mod ground_config;

fn main() {
    println!("CPSS - LoRa Ground Communication Node");

    // configure zenoh
    // start zenoh

    let mut producer_mgmt = ProducerManager::new();
    
    let mut consumer_mgmt = ConsumerManager::new();
    
    let altimeter1 = Rc::new(data_handlers::prng_data_source::PRNG::new(20));
    consumer_mgmt.add(TypeIDs::Altimeter1, altimeter1.clone());

    let negotiate_handler = Rc::new(data_handlers::r_negotiate::NegotiateHandler::new(NegotiatedState {
        downlink_ch: DOWNLINK_CH,
        downlink_coderate: INITIAL_CODE_RATE,
        uplink_ch: UPLINK_CH,
        uplink_coderate: INITIAL_CODE_RATE,
    }));

    let mut connection_mgr = RadioConnectionManager::new_uplink(
        &consumer_mgmt, &producer_mgmt,
        negotiate_handler
    );

    // configure sx1302
    #[cfg(not(test))]
    let mut radio: SX1302<PhysicalDevice> = SX1302::default();
    #[cfg(test)]
    let mut utd = { UnitTestDevice::new() };
    #[cfg(test)]
    let mut radio: SX1302<UnitTestDevice> = SX1302::new(DEFAULT_SX1302_CONFIG, &mut utd);

    // start sx1302
    let mut f_exit = false;
    while !f_exit {
        let mut decoded_packets: Vec<DecodedPacket> = Vec::new();
        // fetch any new packets 
        match radio.try_receive() {
            Ok(packets) => {
                decoded_packets.reserve(packets.len());
                // if have packets, decode them 
                for (size, data) in packets {
                    match packet::decode_incoming_packet(&consumer_mgmt, size, data) {
                        Ok(v) => decoded_packets.push(v),
                        Err(e) => println!("Encountered error while decoding packets: {}", e)
                    };
                };
            },
            Err(_) => println!("Encountered error while trying to receive."),
        };

        let outbound_packets = connection_mgr.update(
        match radio.get_radio_status(Radios::Radio1RxOnly) {
            Ok(RadioStatus::Busy) => true,
            Err(e) => { println!("Encountered error while trying to get radio status: {}", e); false },
            _ => false
        }, decoded_packets);

        if !outbound_packets.is_empty() {
            let conn_state= connection_mgr.get_negotiated_state();
            let pkt_config = OutgoingPacketConfig {
                freq_hz: if ALLOW_CH_CHANGE { conn_state.uplink_ch.into() } else { UPLINK_CH.into() },
                modulation: OutgoingPacketModulation::LoRa { 
                    bandwidth: Bandwidth::Low125khz, 
                    spread_factor: 7, 
                    coderate: conn_state.uplink_coderate, 
                    no_header: false, 
                    invert_polarity: false, 
                    preamble_length: LORA_PREAMBLE_LENGTH
                },
                timing: sx1302::types::OutgoingPacketTiming::Immediate,
                rf_power: 27, // max rf power
            };
            
            for packet in outbound_packets {
                loop {
                    match radio.try_send(pkt_config, &packet) {
                        Ok(_) => break,
                        Err(TrySendError::RadioBusy) => { // retry slightly later if radio is still busy
                            sleep(Duration::from_micros(333));
                            continue;
                        },  
                        Err(e) => {
                            println!("Encountered error while trying to send a packet: {e}");
                            break;
                        }
                    };
                }
            }
        }        

        // sleep by what ever ms for new packets to appear
        sleep(Duration::from_millis(1000));
    }; 
}



