use crate::{common::BufferType, error::ErrorType};


pub mod altimeter;

pub trait DataProducer {
    fn produce(&self) -> Result<BufferType, ErrorType>;
}

pub trait DataConsumer {
    fn consume(&self, buffer: BufferType) -> Result<(), ErrorType>;
}

pub struct NULLProducer {} 
impl DataProducer for NULLProducer {
    fn produce(&self) -> Result<BufferType, ErrorType> {
        Ok(BufferType::new())
    }
}
pub const NOP_PRODUCER: &NULLProducer = &NULLProducer {};

pub struct BlankProducer { size: usize }
impl BlankProducer {
    pub fn size(size: usize) -> &'static Self {
        Box::leak(Box::new(BlankProducer { size }))
    }
}
impl DataProducer for BlankProducer {
    fn produce(&self) -> Result<BufferType, ErrorType> {
        Ok(vec![0x00; self.size])
    }
}

pub struct NULLConsumer {} 
impl DataConsumer for NULLConsumer {
    fn consume(&self, _: BufferType) -> Result<(), ErrorType> {
        Ok(())
    }
}
pub const NOP_CONSUMER: &NULLConsumer = &NULLConsumer {};

