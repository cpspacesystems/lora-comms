use std::alloc::System;
use std::time::SystemTime;

use zenoh;
use zenoh::Wait;
//use zenoh::pubsub::Subscriber;
use zenoh::handlers::FifoChannelHandler;
use zenoh::sample::Sample;

pub struct Subs {
    subscriber : zenoh::pubsub::Subscriber<FifoChannelHandler<Sample>>,
    key : String,
    start_time : SystemTime,
    timed_bytes : timed_bytes
}
pub struct timed_bytes {
    bytes : Vec<u8>,
    times : Vec<SystemTime>
}

pub fn start_session() -> zenoh::Session {
    return zenoh::open(zenoh::Config::default()).wait().unwrap();
}


impl Subs {
    pub fn new(k : String, session : &zenoh::Session) -> Subs {
        let start = SystemTime::now();
        let subscriber = session.declare_subscriber(&k).wait().unwrap();
        Subs { 
            subscriber : subscriber,
            key : k,
            start_time : start,
            timed_bytes : timed_bytes {
                bytes : Vec::new(),
                times : Vec::new()
            }
        }
    }
    pub fn get_test(&self) {
        while let Ok(sample) = self.subscriber.recv() {
            println!("Received: {:?}", sample.payload());
        }
    }
    pub fn get_bytes(&self) {
        while let Ok(sample) = self.subscriber.recv() {
            sample.payload();
        }
    }
    pub fn get_timed_bytes(&self) {
        let time = SystemTime::now().duration_since(self.start_time);
        while let Ok(sample) = self.subscriber.recv() {
            sample.payload();
            println!("Received: {:?}", sample.payload());
        }
    }
    // get
    // one with bytes
    // one with struct for times and u8 arr
    
}