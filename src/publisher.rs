use zenoh;
use zenoh::Wait;
use zenoh::pubsub::Publisher;
use zenoh::handlers::FifoChannelHandler;
use zenoh::sample::Sample;

struct Pubs<'a> {
    publisher : zenoh::pubsub::Publisher<'a>,
    session : zenoh::Session,
}

impl<'a> Pubs<'a> {
    fn new() -> Pubs<'a> {
        let session = zenoh::open(zenoh::Config::default()).wait().unwrap();
        let publisher = session.declare_publisher("key/expr").wait().unwrap();

        Pubs {
            session : session, 
            publisher : publisher
        }
    }
    fn send_str(&self, s : &str) {
        self.publisher.put(s).wait().unwrap()
    }
    fn send_vec(&self, v : &Vec<u8>) {
        self.publisher.put(v).wait().unwrap()
    }
}


fn publisher() {
    
    
}
