
use std::time;
use std::time::Duration;
use zenoh;
use zenoh::Wait;
use zenoh::handlers::Callback;
//use zenoh::pubsub::Publisher;
//use zenoh::handlers::FifoChannelHandler;
//use zenoh::sample::Sample;

pub struct Pubs<'a> {
    publisher : zenoh::pubsub::Publisher<'a>,
    session : zenoh::Session,
    key : String
}

impl<'a> Pubs<'a> {
    pub fn new(k : String) -> Pubs<'a> {
        let session = zenoh::open(zenoh::Config::default()).wait().unwrap();
        let publisher = session.declare_publisher(k.clone()).wait().unwrap();
        Pubs {
            session : session, 
            publisher : publisher,
            key : k,
        }
    }
    pub fn send_str(&self, s : &str) {
        self.publisher.put(s).wait().unwrap()
    }
    pub fn send_vec(&self, v : &Vec<u8>) {
        self.publisher.put(v).wait().unwrap()
    }
    pub fn send<F>(d : Duration, call : F) 
    where
        F: Fn(),
    {
        let time_start = time::SystemTime::now();
        loop {
            if let Ok(dur) = time::SystemTime::now().duration_since(time_start) && dur >= d {
                call();
            }
        }
    }
    
}
