
use flatbuffers;

use crate::{common::BufferType, data_handlers::{DataConsumer, DataProducer}, errors::{self, AnyError}, pubsub};

#[allow(unused_imports)]
#[path = "../../gen/flatbuffers/alitmeter_generated.rs"]
mod fb_alitmeter;


pub struct Producer<const N: usize, T: pubsub::Subscriber> {
    subscriber: T
}

impl<const N: usize, T: pubsub::Subscriber> Producer<N, T> {
    pub fn new(subscriber: T) -> Producer<N, T> {
        Producer {
            subscriber
        }
    }
}

impl<const N: usize, T: pubsub::Subscriber> DataProducer for Producer<N, T> {
    /// gets altimeter data from zenoh and produces a binary representation
    /// 
    /// this produces 4 bytes
    fn produce(&mut self) -> Result<Option<BufferType>, AnyError> {
        
        // let data;
        if let Some(r) = self.subscriber.get()? {
            // data = if let Ok(a) = fb_alitmeter::root_as_altimeter(&r) { a } 
            // else { return Err(errors::ParseFlatbufferAltimeterError("".to_string()).into()); };
            // Ok(Some((data.height() as f32).to_le_bytes().to_vec()))
            Ok(Some(r))
        } else {
            Ok(None)
        }
    }

    fn get_size(&self) -> usize {
        N
    }
}

pub struct Consumer<const N: usize, T: pubsub::Publisher<N>> {
    publisher: T,
}
impl<const N: usize, T: pubsub::Publisher<N>> Consumer<N, T> {
    pub fn new(publisher: T) -> Consumer<N, T> {
        return Consumer { publisher };
    }
}
impl<const N: usize, T: pubsub::Publisher<N>> DataConsumer for Consumer<N, T> {
    fn consume(&mut self, buffer: BufferType) -> Result<(), AnyError> {
        self.publisher.publish(buffer)?;
        Ok(())
    }

    fn get_size(&self) -> usize {
        N
    }
}
