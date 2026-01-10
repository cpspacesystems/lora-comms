
use flatbuffers;

use crate::{common::BufferType, data_handlers::DataProducer, error::{self, ErrorType, LORAError}, publisher, subscriber};

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
    fn produce(&self) -> Result<BufferType, ErrorType> {
        self.subscriber.get();

        let data = if let Ok(d) = fb_alitmeter::root_as_altimeter(&[0; 10]) { d } 
            else { return Err(LORAError::ParseFlatbufferAltimeterError(self.zenoh_path.clone()))};

        Ok((data.height() as f32).to_le_bytes().to_vec())
    }
}

pub struct Consumer {
    zenoh_path: String,
    publisher: publisher
}
