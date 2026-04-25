use std::{rc::Rc, thread::sleep, time::Duration};

#[cfg(all(not(test), feature = "simulation"))]
use crate::network::simulated_radio::SimulatedRadio;
use crate::{common::{Bandwidth, BufferType, LoraChannel, LoraCodeRate}, common_config::{ALLOW_CH_CHANGE, DOWNLINK_CH, INITIAL_CODE_RATE, LORA_PREAMBLE_LENGTH, UPLINK_CH}, data_handlers::{ConsumerManager, DataConsumer, ProducerManager, altimeter::Producer}, network::{NetworkRadio, conn_mgr::RadioConnectionManager}, packet::{DecodedPacket, OutgoingFrameBuilder, data_section::{DecodedDataSection, decode_data_sections}, transmission_ctrl::TSMCtrlInfo}, pubsub::{Connection, tism::TISMConnection, zenoh::{ZenohConnection, ZenohPublisher}}, sx1302::{SX1302, backing::{DeviceBackingAPI, PhysicalDevice}, conf::{DEFAULT_SX1302_CONFIG, SX1302Configuration}, error::TrySendError, types::{RadioStatus, Radios}}};
use crate::network_ids::TypeIDs;

#[cfg(test)]
use crate::sx1302::backing::unit_test_backing::UnitTestDevice;

mod sx1302;
mod packet;
mod common;
mod errors;
mod pubsub;
mod data_handlers;
mod network_ids;
mod network;
mod common_config;
mod ground_config;

fn main() {
    println!("CPSS - LoRa Ground Communication Node");

    // configure zenoh
    let mut zenoh = ZenohConnection::new();
    let mut tism = TISMConnection;
    // start zenoh

    let mut producer_mgmt = ProducerManager::new();
    
    let mut consumer_mgmt = ConsumerManager::new();
    
    let altimeter1 = data_handlers::prng_data_source::PRNG::new(100).as_rc();

    // let altimeter1 = data_handlers::altimeter::Consumer::<5, ZenohPublisher<5>>::new(zenoh.publish("/test/alt1".to_string())).as_rc();
    // producer_mgmt.add(TypeIDs::Altimeter1, altimeter1.clone());
    consumer_mgmt.add(TypeIDs::Altimeter1, altimeter1.clone());

    let mut connection_mgr = RadioConnectionManager::new_uplink(
        common_config::RADIO_ENABLE_UPLINK,
        &consumer_mgmt, &producer_mgmt,
    );

    // configure sx1302
    #[cfg(not(any(test, feature = "simulation")))]
    let mut device = sx1302::backing::PhysicalDevice;
    #[cfg(not(any(test, feature = "simulation")))]
    let mut radio: SX1302<PhysicalDevice> = SX1302::new(ground_config::SX1302_CONFIG, &mut device);
    #[cfg(test)]
    let mut utd = { UnitTestDevice::new() };
    #[cfg(test)]
    let mut radio: SX1302<UnitTestDevice> = SX1302::new(DEFAULT_SX1302_CONFIG, &mut utd);
    #[cfg(all(not(test), feature = "simulation"))]
    let mut radio = SimulatedRadio::new(common_config::SIMULATION_GROUND_ADDR.to_string(), common_config::SIMULATION_ROCKET_ADDR.to_string());

    if let Err(e) = radio.configure() {
        println!("Encountered error while trying to configure the radio: {e}");
        return;
    };
    if let Err(e) = radio.start() {
        println!("Encountered error while trying to start the radio: {e}");
        return;
    }

    // start sx1302
    let mut f_exit = false;
    while !f_exit {
        let mut decoded_packets: Vec<DecodedPacket> = Vec::new();
        // fetch any new packets 
        match radio.try_receive() {
            Ok(packets) => {
                decoded_packets.reserve(packets.len());
                // if have packets, decode them 
                for p in packets {
                    match p.decode(&consumer_mgmt) {
                        Ok(v) => decoded_packets.push(v),
                        Err(e) => println!("Encountered error while decoding packets: {}", e)
                    };
                };
            },
            Err(_) => println!("Encountered error while trying to receive."),
        };

        let outbound_packets = connection_mgr.update(radio.is_currently_receiving().unwrap_or(true), decoded_packets);

        if !outbound_packets.is_empty() {
            let pkt_config = packet::OutgoingPacketConfig {
                freq_hz: UPLINK_CH.into(),
                modulation: packet::OutgoingPacketModulation::LoRa { 
                    bandwidth: Bandwidth::Low125khz, 
                    spread_factor: common::SpreadFactor::SF7, 
                    coderate: INITIAL_CODE_RATE, 
                    no_header: false, 
                    invert_polarity: false, 
                    preamble_length: LORA_PREAMBLE_LENGTH
                },
                timing: packet::OutgoingPacketTiming::Immediate,
                rf_power: 27, // max rf power
            };
            
            for packet in outbound_packets {
                loop {
                    match radio.try_send(pkt_config, &packet) {
                        Ok(t) => {
                            sleep(t); // sleep for packets to finish transmit
                            break;
                        }
                        Err(network::SendError::RadioBusy) => { // retry slightly later if radio is still busy
                            sleep(Duration::from_micros(333));
                            continue;
                        },  
                        Err(e) => {
                            println!("Encountered error while trying to send a packet: {e}");
                            break;
                        }
                    };
                }
                connection_mgr.update_transmit_finish();
            }
        }

        // sleep by what ever ms for new packets to appear
        sleep(Duration::from_millis(360/1000));
    }; 
}



