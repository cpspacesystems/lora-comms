use crate::{data_handlers::{DataConsumer, DataProducer}, errors, pubsub::{Publisher, Subscriber}};



pub struct Producer<const N: usize, T: Subscriber> {
    sub: T
}
impl<const N: usize, T: Subscriber> Producer<N, T> {
    pub fn new(subscriber: T) -> Producer<N, T> {
        Producer {
            sub: subscriber
        }
    }
}

impl<const N: usize, T: Subscriber> DataProducer for Producer<N, T> {
    fn produce(&mut self) -> Result<Option<crate::common::BufferType>, crate::errors::AnyError> {
        if let Some(d) = self.sub.get()? {
            if d.len() != N { return Err(errors::InvalidData(format!("Raw Pubsub expected data size of {}, but got {}!", N, d.len())).into()); }
            Ok(Some(d))
        } else {
            Ok(None)
        }     
    }
    
    fn get_size(&self) -> usize {
        N
    }
}

#[derive(Clone)]
pub struct Consumer<const N: usize, T: Publisher<N>> {
    publisher: T
}
impl<const N: usize, T: Publisher<N>> Consumer<N, T> {
    pub fn new(publisher: T) -> Consumer<N, T> {
        return Consumer { publisher };
    }
}
impl<const N: usize, T: Publisher<N>> DataConsumer for Consumer<N, T> {
    fn consume(&mut self, buffer: crate::common::BufferType) -> Result<(), crate::errors::AnyError> {
        self.publisher.publish(buffer)
    }

    fn get_size(&self) -> usize {
        N
    }
}