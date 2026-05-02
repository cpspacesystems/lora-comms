use tism::dynamic::DynamicBorrowedSharedMemory;

use crate::{common::BufferType, errors, pubsub::{Connection, Publisher, Subscriber, SubscriberOnChange}, simulation};


pub struct TISMConnection;

impl Connection for TISMConnection {
    type S = TISMSubscriber;
    fn subscribe(&mut self, path: impl AsRef<str>) -> Self::S {
        TISMSubscriber {
            sub: tism::dynamic::wait_and_open(path.as_ref()).unwrap()
        }
    }
    
    type SC = TISMSubscriber;
    fn subscribe_on_change(&mut self, path: impl AsRef<str>) -> Self::S {
        Self::subscribe(self, path)
    }    

    type P = TISMPublisher;
    fn publish(&mut self, size: usize, path: impl AsRef<str>) -> Self::P {
        TISMPublisher {
            publisher: tism::dynamic::create(path.as_ref(), size).unwrap(),
            expected_size: size
        }
    }
}

pub struct TISMSubscriber {
    sub: DynamicBorrowedSharedMemory
}
impl Subscriber for TISMSubscriber {
    fn get(&mut self) -> Result<Option<crate::common::BufferType>, crate::errors::AnyError> {
        Ok(Some(self.sub.read()?))
    }
}
impl SubscriberOnChange for TISMSubscriber {   
    fn get_onchange(&mut self) -> Result<Option<BufferType>, errors::AnyError> {
        Ok(self.sub.read_change()?)   
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

