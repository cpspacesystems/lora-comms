use crate::{common::BufferType, errors::AnyError, network_ids::TypeID};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::network_ids::TypeIDs;

pub mod altimeter;
pub mod prng_data_source;
pub mod raw_pubsub;
pub mod constant_poll_rate;
// pub mod r_qos;

/// generic trait for data producers
pub trait DataProducer {
    /// produces some binary data <br>
    /// (Note: returning an empty buffer is still counted as sussefully produced. Just that the data is nothing.)
    fn produce(&mut self) -> Result<Option<BufferType>, AnyError>;
    
    /// how much data is expected to be produced
    fn get_size(&self) -> usize;
}

/// generic trait for data consumers 
pub trait DataConsumer {
    /// consumes some binary data
    fn consume(&mut self, buffer: BufferType) -> Result<(), AnyError>;
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
    fn produce(&mut self) -> Result<Option<BufferType>, AnyError> {
        Ok(Some(vec![0x00; self.size]))
    }
    
    fn get_size(&self) -> usize {
        self.size
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
    fn consume(&mut self, _: BufferType) -> Result<(), AnyError> {
        Ok(())
    }
    fn get_size(&self) -> usize {
        self.size
    }
}

/// manages life time and look up of all data consumers 
pub struct ConsumerManager {
    consumers: HashMap<TypeID, Rc<RefCell<dyn DataConsumer>>>,
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
            use crate::common::AsRc;

            this.add(TypeIDs::Test0, BlankConsumer::size(0).as_rc());
            this.add(TypeIDs::Test1, BlankConsumer::size(3).as_rc());
            this.add(TypeIDs::Test2, BlankConsumer::size(11).as_rc());
            this.add(TypeIDs::Test3, BlankConsumer::size(64).as_rc());
            this.add(TypeIDs::Test4, BlankConsumer::size(128).as_rc());
        }
        this
    }

    /// adds consumer to avaliable consumers managed by this ConsumerManager
    #[inline]
    pub fn add(&mut self, id: TypeIDs, consumer: Rc<RefCell<dyn DataConsumer>>) {
        self.add_by_id(id.into(), consumer);
    }
    /// adds consumer to avaliable consumers managed by this ConsumerManager
    pub fn add_by_id(&mut self, id: TypeID, consumer: Rc<RefCell<dyn DataConsumer>>) {
        println!("Added Consumer of id {}", id);
        self.consumers.insert(id, consumer);
    }

    /// gets consumer by id defined in id_map
    pub fn get_consumer_by_id(&self, id: &TypeIDs) -> Option<Rc<RefCell<dyn DataConsumer>>> {
        match self.consumers.get(&id.into()) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }

    /// gets consumer by id in form of u8 defined in id_map
    pub fn get_consumer_by_u8(&self, id: u8) -> Option<Rc<RefCell<dyn DataConsumer>>> {
        match self.consumers.get(&id) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }
}

/// manages life time and all look up of all data producers
pub struct ProducerManager {
    producers: HashMap<TypeID, Rc<RefCell<dyn DataProducer>>>,
}

impl ProducerManager {
    /// initializes a new ProducerManager and all managed producers 
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut this = Self { producers: HashMap::new() };

        #[cfg(test)] // auto init test consumers if in test mode
        {
            use crate::common::AsRc;

            this.add(TypeIDs::Test0, BlankProducer::size(0).as_rc());
            this.add(TypeIDs::Test1, BlankProducer::size(3).as_rc());
            this.add(TypeIDs::Test2, BlankProducer::size(11).as_rc());
            this.add(TypeIDs::Test3, BlankProducer::size(64).as_rc());
            this.add(TypeIDs::Test4, BlankProducer::size(128).as_rc());
        }

        this
    }

    /// adds producer to avaliable producers managed by this ProducerManager
    #[inline]
    pub fn add(&mut self, id: TypeIDs, producer: Rc<RefCell<dyn DataProducer>>) {
        self.add_by_id(id.into(), producer);
    }
    /// adds producer to avaliable producers managed by this ProducerManager
    pub fn add_by_id(&mut self, id: TypeID, producer: Rc<RefCell<dyn DataProducer>>) {
        println!("Added producer of id {}", id);
        self.producers.insert(id, producer);
    }

    /// gets producer by id defined in id_map
    pub fn get_producer_by_id(&self, id: &TypeIDs) -> Option<Rc<RefCell<dyn DataProducer>>> {
        match self.producers.get(&id.into()) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }
    /// gets producer by id in form of u8 defined in id_map
    pub fn get_producer_by_u8(&self, id: u8) -> Option<Rc<RefCell<dyn DataProducer>>> {
        match self.producers.get(&id) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }

    /// gets an iterator over all avaliable producers
    pub fn iter_producers(&self) -> std::collections::hash_map::Iter<'_, TypeID, Rc<RefCell<dyn DataProducer>>> {
        self.producers.iter()
    }

}