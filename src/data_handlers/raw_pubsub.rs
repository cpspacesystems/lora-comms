use crate::{data_handlers::{DataConsumer, DataProducer}, pubsub::{Publisher, Subscriber}};



pub struct Producer<T: Subscriber> {
    sub: T
}
impl<T: Subscriber> Producer<T> {
    pub fn new(subscriber: T) -> Producer<T> {
        Producer {
            sub: subscriber
        }
    }
}

impl<T: Subscriber> DataProducer for Producer<T> {
    fn produce(&mut self) -> Result<Option<crate::common::BufferType>, crate::errors::AnyError> {
        self.sub.get()        
    }
}

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