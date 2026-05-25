use std::{process::exit, rc::Rc, thread::sleep, time::{self, Duration}};

use log::{error, info};

#[cfg(all(not(test), feature = "simulation"))]
use crate::network::simulated_radio::SimulatedRadio;
use crate::{common::{AsRc, Bandwidth, BufferType, LoraCodeRate}, common_config::{ALLOW_CH_CHANGE, DOWNLINK_SELECTED_CH, INITIAL_CODE_RATE, LORA_PREAMBLE_LENGTH, UPLINK_SELECTED_CH}, data_handlers::{ConsumerManager, DataConsumer, ProducerManager, altimeter::Producer}, network::{NetworkRadio, conn_mgr::RadioConnectionManager}, packet::{DecodedPacket, OutgoingFrameBuilder, data_section::{DecodedDataSection, decode_data_sections}, transmission_ctrl::TSMCtrlInfo}, pubsub::{Connection, tism::TISMConnection, zenoh::{ZenohConnection, ZenohPublisher}}, sx1302::{SX1302, backing::{DeviceBackingAPI, PhysicalDevice}, conf::{DEFAULT_SX1302_CONFIG, SX1302Configuration}, error::TrySendError, types::{RadioStatus, Radios}}};
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
mod simulation;
mod config;

fn main() {
    // initialize logging
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Trace)
        .with_colors(true)
        .with_utc_timestamps()
        .env()
        .init().unwrap()
    ;


    info!(target: "ground", "CPSS - LoRa Ground Communication Node");

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
            error!(target: "ground", "Not Enough Arguments, Expected: <path to config.toml>");
            exit(1);
        } else {
            let _ = args.next().expect("Expected executable name!");
            config_path = args.next().expect("Config path should exist at this point.");
        }
    }
    
    // start zenoh

    let mut producer_mgmt = ProducerManager::new();
    
    let mut consumer_mgmt = ConsumerManager::new();
    
    #[cfg(any(not(feature = "simulation"), feature = "hardware_attached_full_system"))]
    let (mut _tism, mut _zenoh) = {
        info!(target: "ground", "Using config toml at {}", config_path);
        let cfg =  config::parse(config_path).unwrap();
        let mut generator = config::generator::Generator::new(
            || TISMConnection, || ZenohConnection::new());

        generator.add_consuming_entries(&cfg.rocket_to_ground);
        generator.add_producing_entries(&cfg.ground_to_rocket);

        generator.finalize(&mut config::generator::IDProvider::new_ground(), 
            &mut producer_mgmt, &mut consumer_mgmt)
    };

    #[cfg(all(feature = "simulation", not(feature = "hardware_attached_full_system")))]
    {
        let altimeter1 = data_handlers::prng_data_source::PRNG::new(100).as_rc();
        let altimeter2 = data_handlers::prng_data_source::PRNG::new(100).as_rc();
        let altimeter3 = data_handlers::prng_data_source::PRNG::new(100).as_rc();
        let cmd1 = data_handlers::prng_data_source::PRNG::new(10).as_rc();
        // let altimeter1 = data_handlers::altimeter::Consumer::<5, ZenohPublisher<5>>::new(zenoh.publish("/test/alt1".to_string())).as_rc();
        // producer_mgmt.add(TypeIDs::Altimeter1, altimeter1.clone());
        consumer_mgmt.add(TypeIDs::Altimeter1, altimeter1.clone());
        consumer_mgmt.add(TypeIDs::Altimeter2, altimeter2.clone());
        consumer_mgmt.add(TypeIDs::Altimeter3, altimeter3.clone());
        producer_mgmt.add(TypeIDs::SignalDeployParachute, cmd1.clone());
    }

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
        error!(target: "ground", "Encountered error while trying to configure the radio: {e}");
        return;
    };
    if let Err(e) = radio.start() {
        error!(target: "ground", "Encountered error while trying to start the radio: {e}");
        return;
    }

    // start sx1302
    let mut last_loop_time = time::Instant::now();
    loop {
        let mut decoded_packets: Vec<DecodedPacket> = Vec::new();
        // fetch any new packets 
        match radio.try_receive() {
            Ok(packets) => {
                decoded_packets.reserve(packets.len());
                // if have packets, decode them 
                for p in packets {
                    match p.decode(&consumer_mgmt) {
                        Ok(v) => decoded_packets.push(v),
                        Err(e) => error!(target: "ground", "Encountered error while decoding packets: {}", e)
                    };
                };
            },
            Err(_) => error!(target: "ground", "Encountered error while trying to receive."),
        };

        let outbound_packets = connection_mgr.update(radio.is_currently_receiving().unwrap_or(true), decoded_packets);

        if !outbound_packets.is_empty() {
            let pkt_config = packet::OutgoingPacketConfig {
                freq_hz: UPLINK_SELECTED_CH,
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
                            error!(target:"ground", "Encountered error while trying to send a packet: {e}");
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
            let stats = connection_mgr.get_statistics();
            info!(target: "stats", "Receive Kbps: {:.3}, PLR: {:.3}, RECEIVED: {}, LOST: {}", stats.recent_data_rate as f64 / 1000.0, stats.recent_packet_lost_rate, stats.packets_received, stats.packets_lost);
        }

        // sleep by what ever ms for new packets to appear
        // sleep(Duration::from_millis(360/1000));
    }; 
}



