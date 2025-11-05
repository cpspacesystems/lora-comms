use zenoh;
use zenoh::Wait;
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
}


fn publisher() {
    
    
}
