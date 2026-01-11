use crate::{common::BufferType, error::ErrorType};


pub mod altimeter;

/// generic trait for data producers
pub trait DataProducer {
    /// produces some binary data
    fn produce(&self) -> Result<BufferType, ErrorType>;
}

/// generic trait for data consumers 
pub trait DataConsumer {
    /// consumes some binary data
    fn consume(&self, buffer: BufferType) -> Result<(), ErrorType>;
    /// how much data is expected to be provided for consumtion
    fn get_size(&self) -> usize;
}

/// a producer that produces all zeros for size, useful for testing and placeholders 
pub struct BlankProducer { size: usize }
impl BlankProducer {
    pub const fn size(size: usize) -> Self {
        BlankProducer { size }
    }
}
impl DataProducer for BlankProducer {
    fn produce(&self) -> Result<BufferType, ErrorType> {
        Ok(vec![0x00; self.size])
    }
}
/// a consumer that consumes any data with size, useful for testing and placeholders
pub struct BlankConsumer { size: usize } 
impl BlankConsumer {
    pub const fn size(size: usize) -> Self {
        BlankConsumer { size }
    }
}
impl DataConsumer for BlankConsumer {
    fn consume(&self, _: BufferType) -> Result<(), ErrorType> {
        Ok(())
    }
    fn get_size(&self) -> usize {
        self.size
    }
}