use std::marker::PhantomData;

use zenoh::{Config, Wait, config};

use crate::pubsub::{Connection, Publisher, Subscriber};

pub struct ZenohConnection<'a> { 
    _marker: PhantomData<&'a Self>, 
    session: zenoh::Session
}
impl ZenohConnection<'_> {
    pub fn new() -> Self {
        ZenohConnection {
            _marker: PhantomData,
            session: zenoh::open(Config::default()).wait().unwrap(),
        }
    }
}
impl<'a> Connection for ZenohConnection<'a> {
    type S = ZenohSubscriber;
    fn subscribe(&mut self, path: String) -> Self::S {
        ZenohSubscriber {
            subscriber: self.session.declare_subscriber(path).wait().unwrap(),
        }
    }

    type P<const N: usize> = ZenohPublisher<'a, N>;
    fn publish<const N: usize>(&mut self, path: String) -> Self::P<N> {
        ZenohPublisher {
            publisher: self.session.declare_publisher(path).wait().unwrap(),
        }
    }
}

pub struct ZenohPublisher<'a, const N: usize> {
    publisher: zenoh::pubsub::Publisher<'a>
}
impl<const N: usize> Publisher<N> for ZenohPublisher<'_, N> {
    fn publish(&mut self, data: crate::common::BufferType) -> Result<(), crate::errors::AnyError> {
        self.publisher.put(data).wait()?;
        Ok(())
    }
}

pub struct ZenohSubscriber {
    subscriber: zenoh::pubsub::Subscriber<zenoh::handlers::FifoChannelHandler<zenoh::sample::Sample>>
}
impl Subscriber for ZenohSubscriber {
    fn get(&mut self) -> Result<Option<crate::common::BufferType>, crate::errors::AnyError> {
        let sample = self.subscriber.try_recv()?;
        if let Some(s) = sample {
            Ok(Some(s.payload().to_bytes().into()))
        } else {
            Ok(None)
        }
    }
}