use std::{thread::sleep, time::Duration};

use crate::{common::BufferType, data_handlers::DataConsumer, packet::{data_section::decode_data_sections, data_types::ConsumerManager}};

mod sx1302;
mod packet;
mod common;
mod error;
mod publisher;
mod subscriber;
mod data_handlers;

fn build_packet() {

}

fn main() {
    println!("Program starting");

    let consumers = ConsumerManager::init();

    // configure zenoh
    // start zenoh
    // configure sx1302
    // start sx1302
    let mut f_exit = false;
    while !f_exit {
        // try fetch packets from sx1302
        let data = BufferType::new();

        // if have packets, decode (parse into flatbufs) and consume (send to zenoh) them 
        if let Err(e) = packet::decode_and_consume_incoming_packet(&consumers, data) {
            println!("Encountered error while parsing packets: {}", e);
        };
        // sleep by what ever ms for new packets to appear
        sleep(Duration::from_millis(1000));
    }; 
}



