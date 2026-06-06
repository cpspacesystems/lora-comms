use crate::{data_handlers::{DataConsumer, DataProducer}, errors, pubsub::{self, Publisher, Subscriber}};

use flatbuffers;

#[path = "../../gen/flatbuffers/LastUpdatedTime_generated.rs"]
mod fb_last_updated_time;

pub const LastUpdateTimeNetworkSize: usize = 5;
pub const LastUpdateTimeTopicPostFix: &'static str = "_lora_lut";

pub struct Producer<T: Subscriber> {
    with_lut: bool,
    sub: T,
    size: usize,
}
impl<T: Subscriber> Producer<T> {
    pub fn new(mut size: usize, subscriber: T, with_last_update_time: bool) -> Producer<T> {
        size = if with_last_update_time { size + LastUpdateTimeNetworkSize } else { size };
        Producer {
            with_lut: with_last_update_time,
            size,
            sub: subscriber
        }
    }
}

impl<T: Subscriber> DataProducer for Producer<T> {
    fn produce(&mut self) -> Result<Option<crate::common::BufferType>, crate::errors::AnyError> {
        if let Some(mut d) = self.sub.get()? {
            let data_expected_size = if self.with_lut { self.size - LastUpdateTimeNetworkSize } else { self.size };
            
            if d.len() != data_expected_size { 
                return Err(errors::InvalidData(format!("Raw Pubsub expected data size of {}, but got {}!", data_expected_size, d.len())).into()); 
            }

            if self.with_lut {
                let mut ts: u128 = 0;
                if let Some(dur) = self.sub.get_time_micros()? {
                    ts = dur.as_micros();
                }

                d = [&ts.to_le_bytes()[..LastUpdateTimeNetworkSize], d.as_slice()].concat();
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
    publisher: T,
    lut_publisher: Option<T>,
}
impl<T: Publisher> Consumer<T> {
    pub fn new(size: usize, network: &mut impl pubsub::Connection<P = T>, path: impl AsRef<str>, with_last_update_time: bool) -> Consumer<T> {
        return Consumer {
            size: if with_last_update_time { size + LastUpdateTimeNetworkSize } else { size },
            publisher: network.publish(size, path.as_ref()),
            lut_publisher: if with_last_update_time { Some(network.publish(24, path.as_ref().to_owned() + "_lora_lut")) } else { None }, 
        };
    }
}
impl<T: Publisher> DataConsumer for Consumer<T> {
    fn consume(&mut self, buffer: crate::common::BufferType) -> Result<(), crate::errors::AnyError> {
        if buffer.len() != self.size { 
            return Err(errors::InvalidData(format!("Raw Pubsub expected consuming data size of {}, but got {}!", self.size, buffer.len())).into()); 
        }

        if let Some(p) = &mut self.lut_publisher {
            let mut ts_micros_bytes = [0_u8; size_of::<u64>()];
            ts_micros_bytes[..LastUpdateTimeNetworkSize].copy_from_slice(&buffer[..LastUpdateTimeNetworkSize]);
            let ts = u64::from_le_bytes(ts_micros_bytes);

            let mut fb_builder = flatbuffers::FlatBufferBuilder::new();

            let mut lutb = fb_last_updated_time::cpss::tom::lora::LastUpdatedTimeBuilder::new(&mut fb_builder);
            lutb.add_data(& fb_last_updated_time::cpss::tom::lora::LastUpdatedTimeData::new(ts));
            let lutbo = lutb.finish();
            fb_builder.finish_minimal(lutbo);    
            
            p.publish(fb_builder.finished_data().to_owned())?;
        
            self.publisher.publish(buffer[LastUpdateTimeNetworkSize..].to_vec())
        } else {
            self.publisher.publish(buffer)
        }
    }

    fn get_size(&self) -> usize {
        self.size
    }
}