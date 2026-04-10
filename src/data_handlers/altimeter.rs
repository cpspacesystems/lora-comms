
use flatbuffers;

use crate::{common::BufferType, data_handlers::{DataConsumer, DataProducer}, errors::{self, AnyError}, publisher, subscriber};

#[allow(unused_imports)]
#[path = "../../gen/flatbuffers/alitmeter_generated.rs"]
mod fb_alitmeter;


pub struct Producer {
    zenoh_path: String,
    subscriber: subscriber::Subs
}

impl Producer {
    pub fn new(zenoh_path: String) -> Producer {
        Producer {
            zenoh_path: zenoh_path.clone(),
            subscriber: subscriber::Subs::new(zenoh_path)
        }
    }
}

impl DataProducer for Producer {
    /// gets altimeter data from zenoh and produces a binary representation
    /// 
    /// this produces 4 bytes
    fn produce(&self) -> Result<BufferType, AnyError> {
        self.subscriber.get();

        let data = if let Ok(d) = fb_alitmeter::root_as_altimeter(&[0; 10]) { d } 
            else { return Err(errors::ParseFlatbufferAltimeterError(self.zenoh_path.clone()).into())};

        Ok((data.height() as f32).to_le_bytes().to_vec())
    }
    
    fn has_data(&self) -> Result<bool, AnyError> {
        Ok(true)
    }
}

pub struct Consumer<'a> {
    buffer_size: usize,
    zenoh_path: String,
    publisher: publisher::Pubs<'a>,
}
impl<'a> Consumer<'a> {
    pub fn new(buffer_size: usize, zenoh_path: String) -> Self {
        return Self { buffer_size, zenoh_path: zenoh_path.clone(), publisher: publisher::Pubs::new(zenoh_path) };
    }
}
impl<'a> DataConsumer for Consumer<'a> {
    fn consume(&self, buffer: BufferType) -> Result<(), AnyError> {
        self.publisher.send_vec(&buffer);
        Ok(())
    }

    fn get_size(&self) -> usize {
        self.buffer_size
    }
}
