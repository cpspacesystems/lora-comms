use zenoh;
use zenoh::Wait;
use zenoh::pubsub::Publisher;
use zenoh::handlers::FifoChannelHandler;
use zenoh::sample::Sample;

struct pubs<'a> {
    session : zenoh::Session,
    publisher : zenoh::pubsub::Publisher<'a>,
}

impl<'a> pubs<'a> {
    fn new() -> pubs<'a> {
        let session = zenoh::open(zenoh::Config::default()).wait().unwrap();
        let publisher = session.declare_publisher("key/expr").wait().unwrap();

        pubs {
            session : session, 
            publisher : publisher
        }
    }
    fn sendStr(&self, s : &str) {
        self.publisher.put("Hello World").wait().unwrap()
    }
}



fn publisher() {
    
    
}
