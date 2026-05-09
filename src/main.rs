use std::time;
use std::{thread::sleep, time::Duration};

use crate::common::AsRc;
use crate::common_config::{DOWNLINK_SELECTED_CH, LORA_125KHZ_CH0};
use crate::lr1121::LR1121;
#[cfg(any(test, feature = "simulation"))]
use crate::network::simulated_radio::SimulatedRadio;
use crate::pubsub::Connection;
use crate::{common_config::{INITIAL_CODE_RATE, LORA_PREAMBLE_LENGTH}, data_handlers::{ConsumerManager, DataConsumer, ProducerManager}, network::{NetworkRadio, conn_mgr::RadioConnectionManager}, network_ids::{TypeID, TypeIDs}, packet::DecodedPacket, pubsub::{tism::TISMConnection, zenoh::ZenohConnection}};
use std::process::exit;

mod packet;
mod common;
mod errors;
mod pubsub;
mod data_handlers;
mod network_ids;
mod network;
mod common_config;
mod simulation;
mod config;
mod lr1121;
mod rocket_config;

fn main() {
    println!("CPSS - LoRa Rocket Communication Node");

    let config_path: String;

    // parse arguments
    #[cfg(any(feature = "simulation", feature = "hardware_attached_full_system"))]
    { // skip parsing if feat sim or hwas
        config_path = "./etc/hwas.toml".to_string();
    }
    #[cfg(not(any(feature = "simulation", feature = "hardware_attached_full_system")))]
    {
        let mut args = std::env::args();
        if args.len() != 2 {
            println!("Not Enough Arguments, Expected: <path to config.toml>");
            exit(1);
        } else {
            let _ = args.next().expect("Expected executable name!");
            config_path = args.next().expect("Config path should exist at this point.");
        }
    }
    

    let mut producer_mgmt = ProducerManager::new();
    
    let mut consumer_mgmt= ConsumerManager::new();

    #[cfg(any(not(feature = "simulation"), feature = "hardware_attached_full_system"))]
    let (mut _tism, mut _zenoh) = {
        let cfg = config::parse(config_path).expect("Unable to parse config!");
        let mut generator = config::generator::Generator::new(
            || TISMConnection, || ZenohConnection::new());

        generator.add_consuming_entries(&cfg.ground_to_rocket);
        generator.add_producing_entries(&cfg.rocket_to_ground);

        generator.finalize(&mut config::generator::IDProvider::new_rocket(), 
            &mut producer_mgmt, &mut consumer_mgmt)
    };
    
    #[cfg(all(feature = "simulation", not(feature = "hardware_attached_full_system")))]
    {
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
    }

    let mut connection_mgr = RadioConnectionManager::new_downlink(
        common_config::RADIO_ENABLE_UPLINK,
        &consumer_mgmt, &producer_mgmt,
    );

    // configure lr1121
    #[cfg(not(any(test, feature = "simulation")))]
    let mut radio = LR1121::new(rocket_config::LR1121_CONFIG);

    #[cfg(any(test, feature = "simulation"))] // LR1121 has no test provider for now, so simulation provider will act for test
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
    let mut last_loop_time = time::Instant::now();
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

        let outbound_packets = connection_mgr.update(radio.is_currently_receiving().unwrap_or(false), decoded_packets);

        if !outbound_packets.is_empty() {
            let pkt_config = packet::OutgoingPacketConfig {
                freq_hz: DOWNLINK_SELECTED_CH,
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

        let now = time::Instant::now();
        if now.saturating_duration_since(last_loop_time) > time::Duration::from_secs(1) {
            last_loop_time = now;
            dbg!(connection_mgr.get_statistics());
        }
        // sleep by what ever ms for new packets to appear
        // sleep(Duration::from_millis(360/1000));
    }; 
}


