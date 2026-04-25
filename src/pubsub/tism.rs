use tism::dynamic::DynamicBorrowedSharedMemory;

use crate::{common::BufferType, errors, pubsub::{Connection, Publisher, Subscriber}};


pub struct TISMConnection;

impl Connection for TISMConnection {
    type S = TISMSubscriber;
    fn subscribe(&mut self, path: String) -> Self::S {
        TISMSubscriber {
            sub: tism::dynamic::open(path).unwrap()
        }
    }

    type P<const N: usize> = TISMPublisher<N>;
    fn publish<const N: usize>(&mut self, path: String) -> Self::P<N> {
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

pub struct TISMPublisher<const N: usize> {
    publisher: tism::lazy::LazyOwnedSharedMemory<[u8; N], String>
}
impl<const N: usize> Publisher<N> for TISMPublisher<{ N }> {
    fn publish(&mut self, data: crate::common::BufferType) -> Result<(), crate::errors::AnyError> {
        if data.len() != N {
            return Err(errors::InvalidData(format!("Expected {} bytes, got {}", data.len(), N)).into());
        }
        
        self.publisher.write(data.try_into().unwrap())?;
        Ok(())
    }
}

