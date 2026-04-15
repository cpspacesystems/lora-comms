
use flatbuffers;

use crate::{common::BufferType, data_handlers::{DataConsumer, DataProducer}, errors::{self, AnyError}, pubsub};

#[allow(unused_imports)]
#[path = "../../gen/flatbuffers/alitmeter_generated.rs"]
mod fb_alitmeter;


pub struct Producer<T: pubsub::Subscriber> {
    subscriber: T
}

impl<T: pubsub::Subscriber> Producer<T> {
    pub fn new(subscriber: T) -> Producer<T> {
        Producer {
            subscriber
        }
    }
}

impl<T: pubsub::Subscriber> DataProducer for Producer<T> {
    /// gets altimeter data from zenoh and produces a binary representation
    /// 
    /// this produces 4 bytes
    fn produce(&mut self) -> Result<Option<BufferType>, AnyError> {
        
        let data;
        if let Some(r) = self.subscriber.get()? {
            data = if let Ok(a) = fb_alitmeter::root_as_altimeter(&r) { a } 
            else { return Err(errors::ParseFlatbufferAltimeterError("".to_string()).into()); };
            Ok(Some((data.height() as f32).to_le_bytes().to_vec()))
        } else {
            Ok(None)
        }
    }
}

pub struct Consumer<T: pubsub::Publisher> {
    buffer_size: usize,
    publisher: T,
}
impl<T: pubsub::Publisher> Consumer<T> {
    pub fn new(buffer_size: usize, publisher: T) -> Consumer<T> {
        return Consumer { buffer_size, publisher };
    }
}
impl<T: pubsub::Publisher> DataConsumer for Consumer<T> {
    fn consume(&mut self, buffer: BufferType) -> Result<(), AnyError> {
        self.publisher.publish(buffer)?;
        Ok(())
    }

    fn get_size(&self) -> usize {
        self.buffer_size
    }
}
