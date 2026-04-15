use tism::dynamic::DynamicBorrowedSharedMemory;

use crate::{common::BufferType, pubsub::{Connection, Publisher, Subscriber}};


pub struct TISMConnection;

impl Connection for TISMConnection {
    type S = TISMSubscriber;
    fn subscribe(&mut self, path: String) -> Self::S {
        TISMSubscriber {
            sub: tism::dynamic::open(path).unwrap()
        }
    }

    type P = TISMPublisher;
    fn publish(&mut self, path: String) -> Self::P {
        TISMPublisher {
            publisher: tism::lazy::create(path)
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

pub struct TISMPublisher {
    publisher: tism::lazy::LazyOwnedSharedMemory<BufferType, String>
}
impl Publisher for TISMPublisher {
    fn publish(&mut self, data: crate::common::BufferType) -> Result<(), crate::errors::AnyError> {
        self.publisher.write(data)?;
        Ok(())
    }
}

