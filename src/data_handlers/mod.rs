use crate::{common::BufferType, errors::AnyError};
use std::{collections::HashMap, rc::Rc};

use crate::network_ids::TypeIDs;

pub mod altimeter;
pub mod prng_data_source;
// pub mod r_qos;

/// generic trait for data producers
pub trait DataProducer {
    /// produces some binary data <br>
    /// (Note: returning an empty buffer is still counted as sussefully produced. Just that the data is nothing.)
    fn produce(&self) -> Result<BufferType, AnyError>;
    /// weather or not this producer has data to produce
    fn has_data(&self) -> Result<bool, AnyError>;
}

/// generic trait for data consumers 
pub trait DataConsumer {
    /// consumes some binary data
    fn consume(&self, buffer: BufferType) -> Result<(), AnyError>;
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
    fn produce(&self) -> Result<BufferType, AnyError> {
        Ok(vec![0x00; self.size])
    }
    
    fn has_data(&self) -> Result<bool, AnyError> {
        Ok(true)
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
    fn consume(&self, _: BufferType) -> Result<(), AnyError> {
        Ok(())
    }
    fn get_size(&self) -> usize {
        self.size
    }
}

/// manages life time and look up of all data consumers 
pub struct ConsumerManager {
    consumers: HashMap<TypeIDs, Rc<dyn DataConsumer>>,
}

impl ConsumerManager {
    /// initializes a new ConsumerManager
    pub fn new() -> Self {
        #[allow(unused_mut)] // some intellisense doesn't regonize that this is needed if tests is enabled
        let mut this = Self { 
            consumers: HashMap::new(),
        };

        #[cfg(test)] // auto init test consumers if in test mode
        {
            this.add(TypeIDs::Test0, Rc::new(BlankConsumer::size(0)));
            this.add(TypeIDs::Test1, Rc::new(BlankConsumer::size(3)));
            this.add(TypeIDs::Test2, Rc::new(BlankConsumer::size(11)));
            this.add(TypeIDs::Test3, Rc::new(BlankConsumer::size(64)));
            this.add(TypeIDs::Test4, Rc::new(BlankConsumer::size(128)));
        }
        this
    }

    /// adds consumer to avaliable consumers managed by this ConsumerManager
    pub fn add(&mut self, id: TypeIDs, consumer: Rc<dyn DataConsumer>) {
        self.consumers.insert(id, consumer);
    }

    /// gets consumer by id defined in id_map
    pub fn get_consumer_by_id(&self, id: &TypeIDs) -> Option<Rc<dyn DataConsumer>> {
        match self.consumers.get(id) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }

    /// gets consumer by id in form of u8 defined in id_map
    pub fn get_consumer_by_u8(&self, id: u8) -> Option<Rc<dyn DataConsumer>> {
        let id = match TypeIDs::from_repr(id) {
            Some(v) => v,
            None => return None,
        }; 

        match self.consumers.get(&id) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }
}

/// manages life time and all look up of all data producers
pub struct ProducerManager {
    producers: HashMap<TypeIDs, Rc<dyn DataProducer>>,
}

impl ProducerManager {
    /// initializes a new ProducerManager and all managed producers 
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut this = Self { producers: HashMap::new() };

        #[cfg(test)] // auto init test consumers if in test mode
        {
            this.producers.insert(TypeIDs::Test0, Rc::new(BlankProducer::size(0)));
            this.producers.insert(TypeIDs::Test1, Rc::new(BlankProducer::size(3)));
            this.producers.insert(TypeIDs::Test2, Rc::new(BlankProducer::size(11)));
            this.producers.insert(TypeIDs::Test3, Rc::new(BlankProducer::size(64)));
            this.producers.insert(TypeIDs::Test4, Rc::new(BlankProducer::size(128)));
        }

        this
    }

    /// adds producer to avaliable producers managed by this ProducerManager
    pub fn add(&mut self, id: TypeIDs, producer: Rc<dyn DataProducer>) {
        self.producers.insert(id, producer);
    }


    /// gets producer by id defined in id_map
    pub fn get_producer_by_id(&self, id: &TypeIDs) -> Option<Rc<dyn DataProducer>> {
        match self.producers.get(id) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }
    /// gets producer by id in form of u8 defined in id_map
    pub fn get_producer_by_u8(&self, id: u8) -> Option<Rc<dyn DataProducer>> {
        let id = match TypeIDs::from_repr(id) {
            Some(v) => v,
            None => return None,
        }; 

        match self.producers.get(&id) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }

    /// gets an iterator over all avaliable producers
    pub fn iter_producers(&self) -> std::collections::hash_map::Iter<'_, TypeIDs, Rc<dyn DataProducer>> {
        self.producers.iter()
    }

}