use std::{borrow::Cow, marker::PhantomData};

use zenoh::{Config, Wait, config, sample::Sample};

use crate::pubsub::{Connection, Publisher, Subscriber, SubscriberOnChange};

pub struct ZenohConnection { 
    // _marker: PhantomData<&'a Self>, 
    session: zenoh::Session
}
impl ZenohConnection {
    pub fn new() -> Self {
        ZenohConnection {
            // _marker: PhantomData,
            session: zenoh::open(Config::default()).wait().unwrap(),
        }
    }
}
impl Connection for ZenohConnection {
    type S = ZenohSubscriber;
    fn subscribe(&mut self, path: impl AsRef<str>) -> Self::S {
        ZenohSubscriber {
            subscriber: self.session.declare_subscriber(path.as_ref())
                .with(zenoh::handlers::RingChannel::new(1))
                .wait().unwrap(),
        }
    }

    type SC = ZenohOnChangeSubscriber;
    fn subscribe_on_change(&mut self, path: impl AsRef<str>) -> Self::SC {
        ZenohOnChangeSubscriber {
            subscriber: self.session.declare_subscriber(path.as_ref())
                .with(zenoh::handlers::RingChannel::new(1))
                .wait().unwrap(),
            last: None
        }
    }

    type P = ZenohPublisher;
    fn publish(&mut self, _size: usize, path: impl AsRef<str>) -> Self::P {
        ZenohPublisher {
            publisher: self.session.declare_publisher(path.as_ref().to_owned()).wait().unwrap(),
        }
    }
}

pub struct ZenohPublisher {
    publisher: zenoh::pubsub::Publisher<'static>,
}
impl Publisher for ZenohPublisher {
    fn publish(&mut self, data: crate::common::BufferType) -> Result<(), crate::errors::AnyError> {
        self.publisher.put(data).wait()?;
        Ok(())
    }
}

pub struct ZenohSubscriber {
    subscriber: zenoh::pubsub::Subscriber<zenoh::handlers::RingChannelHandler<zenoh::sample::Sample>>,
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

pub struct ZenohOnChangeSubscriber {
    subscriber: zenoh::pubsub::Subscriber<zenoh::handlers::RingChannelHandler<zenoh::sample::Sample>>,
    last: Option<Vec<u8>>,
}
impl SubscriberOnChange for ZenohOnChangeSubscriber {
    fn get_onchange(&mut self) -> Result<Option<crate::common::BufferType>, crate::errors::AnyError> {
        let sample = self.subscriber.try_recv()?;
        if let Some(s) = sample {
            let r = s.payload().to_bytes().into();

            if let Some(l) = &self.last && r == *l {
                return Ok(None);
            }

            self.last = Some(r);
            Ok(self.last.clone())
        } else {
            Ok(None)
        }   
    }
}