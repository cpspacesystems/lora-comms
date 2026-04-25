use std::{thread::sleep, time::Duration};

// #[cfg(all(not(test), feature = "simulation"))]
use crate::network::simulated_radio::SimulatedRadio;
use crate::{common_config::{DOWNLINK_CH, INITIAL_CODE_RATE, LORA_PREAMBLE_LENGTH}, data_handlers::{ConsumerManager, DataConsumer, ProducerManager}, network::{NetworkRadio, conn_mgr::RadioConnectionManager}, network_ids::{TypeID, TypeIDs}, packet::DecodedPacket, pubsub::{tism::TISMConnection, zenoh::ZenohConnection}};

mod packet;
mod common;
mod errors;
mod pubsub;
mod data_handlers;
mod network_ids;
mod network;
mod common_config;

//lora radio code: tell zenoh when to get new data
//goes through everything it needs to get (newest version of data) and puts it into flatbuffers
//put flatbuffers into packets
//send packets into lora
//schedule packet to be sent over lora
// fn main() {
//     println!("Program starting");

//     let mut f_exit = false;
//     while !f_exit {
//         // grab stuff from zenoh and convert it to data section
//         // send said packet to LR1100 interface 
//         // configure LR1100 
//         // send bytes
//         //
//     }; 
// }


fn main() {
    println!("CPSS - LoRa Ground Communication Node");

    // configure zenoh
    // let mut zenoh = ZenohConnection::new();
    let mut tism = TISMConnection;
    // start zenoh

    let mut producer_mgmt = ProducerManager::new();
    
    let mut consumer_mgmt = ConsumerManager::new();
    
    let altimeter1 = data_handlers::prng_data_source::PRNG::new(100).as_rc();
    let altimeter2 = data_handlers::prng_data_source::PRNG::new(100).as_rc();
    let cmd1 = data_handlers::prng_data_source::PRNG::new(10).as_rc();
    // let altimeter3 = data_handlers::prng_data_source::PRNG::new(100).as_rc();

    // let altimeter1 = data_handlers::altimeter::Consumer::<5, ZenohPublisher<5>>::new(zenoh.publish("/test/alt1".to_string())).as_rc();
    producer_mgmt.add(TypeIDs::Altimeter1, altimeter1.clone());
    producer_mgmt.add(TypeIDs::Altimeter2, altimeter2.clone());
    // producer_mgmt.add(TypeIDs::Altimeter3, altimeter3.clone());
    consumer_mgmt.add(TypeIDs::Altimeter1, altimeter1.clone());
    consumer_mgmt.add(TypeIDs::SignalDeployParachute, cmd1.clone());


    let mut connection_mgr = RadioConnectionManager::new_downlink(
        common_config::RADIO_ENABLE_UPLINK,
        &consumer_mgmt, &producer_mgmt,
    );

    // configure sx1302
    #[cfg(not(any(test, feature = "simulation")))]
    let mut device = sx1302::backing::PhysicalDevice;
    #[cfg(not(any(test, feature = "simulation")))]
    let mut radio: SX1302<PhysicalDevice> = SX1302::new(ground_config::SX1302_CONFIG, &mut device);
    // #[cfg(all(not(test), feature = "simulation"))]
    let mut radio = SimulatedRadio::new(common_config::SIMULATION_ROCKET_ADDR.to_string(), common_config::SIMULATION_GROUND_ADDR.to_string());

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
                println!("IN  {} packets.", packets.len());
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

        let outbound_packets = connection_mgr.update(radio.is_currently_receiving().unwrap_or(false), decoded_packets);

        if !outbound_packets.is_empty() {
            let pkt_config = packet::OutgoingPacketConfig {
                freq_hz: DOWNLINK_CH.into(),
                modulation: packet::OutgoingPacketModulation::LoRa { 
                    bandwidth: common::Bandwidth::Low125khz, 
                    spread_factor: common::SpreadFactor::SF7, 
                    coderate: INITIAL_CODE_RATE, 
                    no_header: false, 
                    invert_polarity: false, 
                    preamble_length: LORA_PREAMBLE_LENGTH
                },
                timing: packet::OutgoingPacketTiming::Immediate,
                rf_power: 27, // max rf power
            };

            println!("OUT {} packets.", outbound_packets.len());            
            for packet in outbound_packets {
                loop {
                    match radio.try_send(pkt_config, &packet) {
                        Ok(t) => { 
                            sleep(t); // sleep for packets to finish transmit
                            break;
                        },
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

        dbg!(connection_mgr.get_statistics());
        // sleep by what ever ms for new packets to appear
        // sleep(Duration::from_millis(360/1000));
    }; 
}


