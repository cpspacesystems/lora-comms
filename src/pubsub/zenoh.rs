use std::{borrow::Cow, marker::PhantomData};

use zenoh::{Config, Wait, config, sample::Sample};

use crate::pubsub::{Connection, Publisher, Subscriber};

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
            path: path.as_ref().to_owned(),
            last_update_time: None,
            subscriber: self.session.declare_subscriber(path.as_ref())
                .with(zenoh::handlers::RingChannel::new(1))
                .wait().unwrap(),
        }
    }

    type SC = ZenohOnChangeSubscriber;
    fn subscribe_on_change(&mut self, path: impl AsRef<str>) -> Self::SC {
        ZenohOnChangeSubscriber {
            path: path.as_ref().to_owned(),
            last_update_time: None,
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
    path: String,
    last_update_time: Option<std::time::Duration>,
    subscriber: zenoh::pubsub::Subscriber<zenoh::handlers::RingChannelHandler<zenoh::sample::Sample>>,
}
impl Subscriber for ZenohSubscriber {
    fn get(&mut self) -> Result<Option<crate::common::BufferType>, crate::errors::AnyError> {
        let sample = self.subscriber.try_recv()?;
        if let Some(s) = sample {
            self.last_update_time = if let Some(ts) = s.timestamp() {
                let t = ts.get_time();
                // TODO: THIS SHOULD BE AN OFFSET, NOT JUST THE TIME CONVERTED TO DURATION
                Some(std::time::Duration::new(t.as_secs().into(), t.subsec_nanos()))
            } else {
                None
            };
            Ok(Some(s.payload().to_bytes().into()))
        } else {
            Ok(None)
        }
    }
    
    fn get_time_micros(&mut self) -> Result<Option<std::time::Duration>, crate::errors::AnyError> {
        Ok(self.last_update_time)
    }

    fn get_path(&self) -> impl AsRef<str> {
        &self.path
    }
}

pub struct ZenohOnChangeSubscriber {
    path: String,
    last_update_time: Option<std::time::Duration>,
    subscriber: zenoh::pubsub::Subscriber<zenoh::handlers::RingChannelHandler<zenoh::sample::Sample>>,
    last: Option<Vec<u8>>,
}
impl Subscriber for ZenohOnChangeSubscriber {
    fn get(&mut self) -> Result<Option<crate::common::BufferType>, crate::errors::AnyError> {
        let sample = self.subscriber.try_recv()?;
        if let Some(s) = sample {
            let r = s.payload().to_bytes().into();

            if let Some(l) = &self.last && r == *l {
                return Ok(None);
            }

            self.last_update_time = if let Some(ts) = s.timestamp() {
                let t = ts.get_time();
                // TODO: THIS SHOULD BE AN OFFSET, NOT JUST THE TIME CONVERTED TO DURATION
                Some(std::time::Duration::new(t.as_secs().into(), t.subsec_nanos()))
            } else {
                None
            };

            self.last = Some(r);
            Ok(self.last.clone())
        } else {
            Ok(None)
        }   
    }
   
    fn get_time_micros(&mut self) -> Result<Option<std::time::Duration>, crate::errors::AnyError> {
        Ok(self.last_update_time)
    }

    fn get_path(&self) -> impl AsRef<str> {
        &self.path
    }    
}