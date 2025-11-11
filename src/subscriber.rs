use zenoh;
use zenoh::Wait;
//use zenoh::pubsub::Subscriber;
use zenoh::handlers::FifoChannelHandler;
use zenoh::sample::Sample;

pub struct Subs {
    session : zenoh::Session,
    subscriber : zenoh::pubsub::Subscriber<FifoChannelHandler<Sample>>,
    key : String
}

impl Subs {
    pub fn new(k : String) -> Subs {
        let session = zenoh::open(zenoh::Config::default()).wait().unwrap();
        let subscriber = session.declare_subscriber(&k).wait().unwrap();
        Subs {
            session : session, 
            subscriber : subscriber,
            key : k,
        }
    }
    pub fn get(&self) {
        while let Ok(sample) = self.subscriber.recv() {
            println!("Received: {:?}", sample.payload());
        }
    }
    
}