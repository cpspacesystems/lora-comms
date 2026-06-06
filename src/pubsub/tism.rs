use std::time::Duration;

use log::warn;
use tism::dynamic::DynamicBorrowedSharedMemory;

use crate::{common::BufferType, common_config, errors, pubsub::{Connection, Publisher, Subscriber}, simulation};


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
    
    type SC = TISMOnChangeSubscriber;
    fn subscribe_on_change(&mut self, path: impl AsRef<str>) -> Self::SC {
        TISMOnChangeSubscriber(Self::subscribe(self, path))
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
                    true
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
                    warn!(target: "tism", "LRA time {:?} is eariler than REF time {:?}.", ts, self.reference_time);
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
            let d = sub.read()?;
            
            if let Some(lpt) = sub.staleness() && lpt < common_config::TISM_MAX_STALENESS {
                Ok(Some(d))
            } else {
                warn!(target: "tism", "Topic {} is closed due to staleness.", self.path);
                self.sub = None;
                Ok(None)
            }
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

#[repr(transparent)]
pub struct TISMOnChangeSubscriber(TISMSubscriber);
impl Subscriber for TISMOnChangeSubscriber {   
    fn get(&mut self) -> Result<Option<BufferType>, errors::AnyError> {
        self.0.try_open_sub();

        if let Some(sub) = &mut self.0.sub { 
            let data = sub.read_change()?;
            
            if let Some(lpt) = sub.staleness() && lpt < common_config::TISM_MAX_STALENESS {
                Ok(data)
            } else {
                warn!(target: "tism", "Topic {} is closed due to staleness.", self.0.path);
                self.0.sub = None;
                Ok(None)
            }

        } else {
            Ok(None)
        }
    }

    fn get_time_micros(&mut self) -> Result<Option<Duration>, crate::errors::AnyError> {
        self.0.get_time_micros_universal()
    }

    fn get_path(&self) -> impl AsRef<str> {
        &self.0.path
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

