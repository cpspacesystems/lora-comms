use std::alloc::System;
use std::time::{Duration, Instant};

use zenoh;
use zenoh::Wait;
use zenoh::bytes::ZBytes;
//use zenoh::pubsub::Subscriber;
use zenoh::handlers::FifoChannelHandler;
use zenoh::sample::Sample;

pub struct Sub {
    subscriber : zenoh::pubsub::Subscriber<FifoChannelHandler<Sample>>,
    key : String,
    start_time : Instant,
    timed_bytes : Vec<TimeStamp>
}
struct TimeStamp {
    bytes : u8,
    time : Duration
}

pub fn start_session() -> zenoh::Session {
    return zenoh::open(zenoh::Config::default()).wait().unwrap();
}


impl Sub {
    pub fn new(k : String, session : &zenoh::Session) -> Sub {
        let start = Instant::now();
        let subscriber = session.declare_subscriber(&k).wait().unwrap();
        Sub { 
            subscriber : subscriber,
            key : k,
            start_time : start,
            timed_bytes : Vec::new()
        }
    }
    pub fn get_test(&self) {
        while let Ok(sample) = self.subscriber.recv() {
            println!("Received: {:?}", sample.payload());
        }
    }
    pub fn receive(&mut self) {
        while let Ok(sample) = self.subscriber.recv() {
            let time : Duration = self.start_time.elapsed();
            for i in sample.payload().slices() {
                self.timed_bytes.push(
                TimeStamp {
                    bytes: i[0], 
                    time: time
                }
            );
            }

            println!("Received: {:?}", sample.payload());
        }
    }
    
}