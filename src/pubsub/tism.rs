use std::time::Duration;

use log::warn;
use tism::dynamic::DynamicBorrowedSharedMemory;

use crate::{common::BufferType, errors, pubsub::{Connection, Publisher, Subscriber, SubscriberOnChange}, simulation};


pub struct TISMConnection {
    reference_time: std::time::Instant
}

impl TISMConnection {
    pub fn new() -> Self {
        Self { reference_time: std::time::Instant::now() }
    }
}

impl Connection for TISMConnection {
    type S = TISMSubscriber;
    fn subscribe(&mut self, path: impl AsRef<str>) -> Self::S {
        TISMSubscriber {
            reference_time: self.reference_time.clone(),
            path: path.as_ref().to_owned(),
            sub: None,
        }
    }
    
    type SC = TISMSubscriber;
    fn subscribe_on_change(&mut self, path: impl AsRef<str>) -> Self::S {
        Self::subscribe(self, path)
    }    

    type P = TISMPublisher;
    fn publish(&mut self, size: usize, path: impl AsRef<str>) -> Self::P {
        TISMPublisher {
            publisher: tism::dynamic::create(path.as_ref(), size).expect(format!("Unable to create TISM allocation {}", path.as_ref()).as_ref()),
            expected_size: size
        }
    }
}

pub struct TISMSubscriber {
    reference_time: std::time::Instant,
    path: String,
    sub: Option<DynamicBorrowedSharedMemory>
}
impl TISMSubscriber {
    pub fn try_open_sub(&mut self) -> bool {
        if let None = self.sub {
            match tism::dynamic::open(&self.path) {
                Ok(s) => {
                    self.sub = Some(s);
                    true
                },
                Err(e) => {
                    warn!(target: "tism", "Unable to open TISM allocation with error: {e}");
                    false
                },
            }
        } else {
            true
        }
    }

    fn get_time_micros_universal(&mut self) -> Result<Option<Duration>, crate::errors::AnyError> {
        if let Some(sub) = &mut self.sub {
            if let Some(ts) = sub.last_read_at() {
                if let Some(d) = ts.checked_duration_since(self.reference_time) {
                    Ok(Some(d))
                } else {
                    warn!(target: "tism", "LRA time is eariler than REF time.");
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

impl Subscriber for TISMSubscriber {
    
    fn get(&mut self) -> Result<Option<crate::common::BufferType>, crate::errors::AnyError> {
        self.try_open_sub();

        if let Some(sub) = &mut self.sub { 
            Ok(Some(sub.read()?))
        } else {
            Ok(None)
        }
    }
    
    fn get_time_micros(&mut self) -> Result<Option<Duration>, crate::errors::AnyError> {
        self.get_time_micros_universal()
    }
    
    fn get_path(&self) -> impl AsRef<str> {
        &self.path
    }
}
impl SubscriberOnChange for TISMSubscriber {   
    fn get_onchange(&mut self) -> Result<Option<BufferType>, errors::AnyError> {
        self.try_open_sub();

        if let Some(sub) = &mut self.sub { 
            Ok(sub.read_change()?)
        } else {
            Ok(None)
        }
    }

    fn get_time_micros(&mut self) -> Result<Option<Duration>, crate::errors::AnyError> {
        self.get_time_micros_universal()
    }

    fn get_path(&self) -> impl AsRef<str> {
        &self.path
    }
}


pub struct TISMPublisher {
    expected_size: usize,
    publisher: tism::dynamic::OwnedDynamicSharedMemory
}
impl Publisher for TISMPublisher {
    fn publish(&mut self, data: crate::common::BufferType) -> Result<(), crate::errors::AnyError> {
        if data.len() != self.expected_size {
            return Err(errors::InvalidData(format!("Expected {} bytes, got {}", self.expected_size, data.len())).into());
        }
        
        self.publisher.write(data.try_into().unwrap())?;
        Ok(())
    }
}

