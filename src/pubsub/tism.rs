use tism::dynamic::DynamicBorrowedSharedMemory;

use crate::{common::BufferType, errors, pubsub::{Connection, Publisher, Subscriber, SubscriberOnChange}};


pub struct TISMConnection;

impl Connection for TISMConnection {
    type S = TISMSubscriber;
    fn subscribe(&mut self, path: String) -> Self::S {
        TISMSubscriber {
            sub: tism::dynamic::wait_and_open(path).unwrap()
        }
    }
    
    type SC = TISMSubscriber;
    fn subscribe_on_change(&mut self, path: String) -> Self::S {
        Self::subscribe(self, path)
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
impl SubscriberOnChange for TISMSubscriber {   
    fn get_onchange(&mut self) -> Result<Option<BufferType>, errors::AnyError> {
        Ok(self.sub.read_change()?)   
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

