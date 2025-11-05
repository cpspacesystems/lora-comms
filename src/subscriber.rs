use zenoh;
use zenoh::Wait;
use zenoh::pubsub::Subscriber;
use zenoh::handlers::FifoChannelHandler;
use zenoh::sample::Sample;

struct subs {
    session : zenoh::Session,
    subscriber : zenoh::pubsub::Subscriber<FifoChannelHandler<Sample>>
}

impl subs {
    fn new() -> subs {
        let session = zenoh::open(zenoh::Config::default()).wait().unwrap();
        let subscriber = session.declare_subscriber("key/expr").wait().unwrap();

        subs {
            session: session, 
            subscriber: subscriber
        }
    }
    fn get(&self) {
        while let Ok(sample) = self.subscriber.recv() {
            println!("Received: {:?}", sample.payload());
        }
    }
}