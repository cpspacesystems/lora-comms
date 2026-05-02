use crate::{data_handlers::{DataConsumer, DataProducer}, errors, pubsub::{Publisher, Subscriber}};



pub struct Producer<T: Subscriber> {
    sub: T,
    size: usize,
}
impl<T: Subscriber> Producer<T> {
    pub fn new(size: usize, subscriber: T) -> Producer<T> {
        Producer {
            size,
            sub: subscriber
        }
    }
}

impl<T: Subscriber> DataProducer for Producer<T> {
    fn produce(&mut self) -> Result<Option<crate::common::BufferType>, crate::errors::AnyError> {
        if let Some(d) = self.sub.get()? {
            if d.len() != self.size { 
                println!("{:?}", d);
                return Err(errors::InvalidData(format!("Raw Pubsub expected data size of {}, but got {}!", self.size, d.len())).into()); 
            }
            Ok(Some(d))
        } else {
            Ok(None)
        }     
    }
    
    fn get_size(&self) -> usize {
        self.size
    }
}

#[derive(Clone)]
pub struct Consumer<T: Publisher> {
    size: usize,
    publisher: T
}
impl<T: Publisher> Consumer<T> {
    pub fn new(size: usize, publisher: T) -> Consumer<T> {
        return Consumer { size, publisher };
    }
}
impl<T: Publisher> DataConsumer for Consumer<T> {
    fn consume(&mut self, buffer: crate::common::BufferType) -> Result<(), crate::errors::AnyError> {
        self.publisher.publish(buffer)
    }

    fn get_size(&self) -> usize {
        self.size
    }
}