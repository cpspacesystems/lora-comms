use std::{collections::HashMap, fmt::Debug, ops::Deref, rc::Rc, sync};

use crate::{configure, common::BufferType, error::ErrorType};

pub type ID = u8;
pub type OutgoingDataProducer = &'static (dyn Fn() -> Result<BufferType, ErrorType> + Sync);
pub type IncomingDataConsumer = &'static (dyn Fn(BufferType) -> Result<(), ErrorType> + Sync);

// placeholder functions for producers and consumers
pub fn producer_nop() -> OutgoingDataProducer { &|| {Ok(BufferType::new())} }
pub fn consumer_nop() -> IncomingDataConsumer { &|_| {Ok(())} }
#[macro_export]
macro_rules! producer_size {
    ($size:expr) => {
        &|| Ok(vec![0; $size])
    };
}

/// common type for all data section records
#[derive(Copy, Clone)]
pub struct Record {
    /// data type identifier
    pub id: ID,
    /// data type name
    pub name: &'static str,
    /// bytes in data section (0 if flexiable data section)
    pub size: usize,
    /// outgoing data producer
    pub producer: OutgoingDataProducer,
    /// incoming data consumer
    pub consumer: IncomingDataConsumer,
}
impl Debug for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Record").field("id", &self.id).field("name", &self.name).field("size", &self.size)
         .field("producer", &"FuncPtr").field("consumer", &"FuncPtr").finish()
    }
}
impl PartialEq for Record {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.name == other.name && self.size == other.size
    }
}
impl Record {
    pub fn produce(&self) -> Result<BufferType, ErrorType> { (self.producer)() }
    pub fn consume(&self, data: BufferType) -> Result<(), ErrorType> { (self.consumer)(data) } 
}

/// id allocations, please follow these allocations otherwise undefined behavior
pub mod type_allocations {
    use crate::packet::record::ID;

    pub const RESERVED: std::ops::Range<ID> = 0..10;
    pub const DATA: std::ops::RangeInclusive<ID> = 10..=255;
}

pub struct RecordMap {
    name_map: HashMap<&'static str, Record>,
    id_map: HashMap<u8, Record>,
}
impl RecordMap {
    /// shorthand for adding new DataSectionTypes
    pub fn ds(&mut self, id: ID, name: &'static str, size: usize, producer: OutgoingDataProducer, consumer: IncomingDataConsumer) -> &mut Self {
        let data = Record {id, name, size, producer, consumer};
        self.id_map.insert(id, data);
        self.name_map.insert(name, data);
        self
    }

    /// initializes a new IDAllocMap
    pub fn init() -> Self {
        let mut this: RecordMap = RecordMap {
            name_map: HashMap::new(),
            id_map: HashMap::new(),
        };
        
        configure::configure(&mut this);

        this
    }
    /// gets the dtype corrosponding to name (!panic-able)
    pub fn by_name(&self, name: &'static str) -> Record {
        return self.name_map[name];
    }
    /// gets the dtype corrosponding to id (!panic-able)
    pub fn by_id(&self, id: &ID) -> Record {
        return self.id_map[id];
    }
    /// try to get the dtype corrosponding to id
    pub fn try_id(&self, id: &ID) -> Option<Record> {
        self.id_map.get(id).copied()
    }
}

// the default id mappings
pub static DEFAULT_RECORD_MAP: std::sync::LazyLock<RecordMap> = std::sync::LazyLock::new(|| {
    RecordMap::init()
});
/// gets the dtype corrosponding to name in the default id mappings (!panic-able)
pub fn by_name(name: &'static str) -> Record {
    DEFAULT_RECORD_MAP.by_name(name)
}
/// gets the dtype corrosponding to id in the default id mappings (!panic-able)
pub fn by_id(id: &ID) -> Record {
    DEFAULT_RECORD_MAP.by_id(id)
}
/// try to get the dtype corrosponding to id in the default id mappings 
pub fn try_id(id: &ID) -> Option<Record> {
    DEFAULT_RECORD_MAP.try_id(id)
}