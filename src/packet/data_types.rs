#[cfg(test)]
use crate::data_handlers::BlankConsumer;
use crate::data_handlers::{self, BlankProducer};

pub type ID = u8;

#[allow(non_upper_case_globals)]
pub mod id_map {
    use crate::packet::data_types::ID;
    // Reserved Types
    // ids allocated here are: [0, 10)

    pub const r_reset: ID = 1;

    // User defined types
    // ids allocated here are: [10, 100)
    pub const altimeter1: ID = 10;
    pub const altimeter2: ID = 11;
    pub const altimeter3: ID = 12;

    pub const IMU1: ID = 21;
    pub const IMU2: ID = 22;
    pub const IMU3: ID = 23;

    pub const GPS1: ID = 30;

    // types for testing 
    // ids allocated here are [250, 256)
    #[cfg(test)]
    pub const __test0: ID = 250;
    #[cfg(test)]
    pub const __test1: ID = 251;
    #[cfg(test)]
    pub const __test2: ID = 252;
    #[cfg(test)]
    pub const __test3: ID = 253;
    #[cfg(test)]
    pub const __test4: ID = 254;
    // 255 to be non existant for tests
}

/// manages life time and look up of all data consumers 
pub struct ConsumerManager<'a> {
    altimeter1: data_handlers::altimeter::Consumer<'a>,
    altimeter2: data_handlers::altimeter::Consumer<'a>,
    altimeter3: data_handlers::altimeter::Consumer<'a>,

    #[cfg(test)]
    __test0: BlankConsumer,
    #[cfg(test)]
    __test1: BlankConsumer,
    #[cfg(test)]
    __test2: BlankConsumer,
    #[cfg(test)]
    __test3: BlankConsumer,
    #[cfg(test)]
    __test4: BlankConsumer,  
}
impl ConsumerManager<'_> {
    /// initializes a new ConsumerManager and all managed consumers 
    pub fn init() -> Self {
        Self { 
            altimeter1: data_handlers::altimeter::Consumer::new(4, "test/altimeter1".into()), 
            altimeter2: data_handlers::altimeter::Consumer::new(4, "test/altimeter2".into()), 
            altimeter3: data_handlers::altimeter::Consumer::new(4, "test/altimeter3".into()),

            #[cfg(test)]
            __test0: BlankConsumer::size(0),
            #[cfg(test)]
            __test1: BlankConsumer::size(3),
            #[cfg(test)]
            __test2: BlankConsumer::size(11),
            #[cfg(test)]
            __test3: BlankConsumer::size(64),
            #[cfg(test)]
            __test4: BlankConsumer::size(128),
        }
    }

    /// gets consumer by id defined in id_map
    pub const fn get_consumer_by_id(&self, id: ID) -> Option<&dyn data_handlers::DataConsumer> {
        Some(match id {
            id_map::altimeter1 => &self.altimeter1,
            id_map::altimeter2 => &self.altimeter2,
            id_map::altimeter3 => &self.altimeter3,

            #[cfg(test)]
            id_map::__test0 => &self.__test0,
            #[cfg(test)]
            id_map::__test1 => &self.__test1,
            #[cfg(test)]
            id_map::__test2 => &self.__test2,
            #[cfg(test)]
            id_map::__test3 => &self.__test3,
            #[cfg(test)]
            id_map::__test4 => &self.__test4,
            _ => return None
        })
    }
}

/// manages life time and all look up of all data producers
pub struct ProducerManager {
    altimeter1: data_handlers::altimeter::Producer,
    altimeter2: data_handlers::altimeter::Producer,
    altimeter3: data_handlers::altimeter::Producer,

    #[cfg(test)]
    __test0: BlankProducer,
    #[cfg(test)]
    __test1: BlankProducer,
    #[cfg(test)]
    __test2: BlankProducer,
    #[cfg(test)]
    __test3: BlankProducer,
    #[cfg(test)]
    __test4: BlankProducer,    

}

impl ProducerManager {
    /// initializes a new ProducerManager and all managed producers 
    pub fn init() -> Self {
        Self { 
            altimeter1: data_handlers::altimeter::Producer::new("test/altimeter1".into()), 
            altimeter2: data_handlers::altimeter::Producer::new("test/altimeter2".into()), 
            altimeter3: data_handlers::altimeter::Producer::new("test/altimeter3".into()),
        

            #[cfg(test)]
            __test0: BlankProducer::size(0),
            #[cfg(test)]
            __test1: BlankProducer::size(3),
            #[cfg(test)]
            __test2: BlankProducer::size(11),
            #[cfg(test)]
            __test3: BlankProducer::size(64),
            #[cfg(test)]
            __test4: BlankProducer::size(128),
        }
    }

    /// gets producer by id defined in id_map
    pub const fn get_producer_by_id(&self, id: ID) -> Option<&dyn data_handlers::DataProducer> {
        Some(match id {
            id_map::altimeter1 => &self.altimeter1,
            id_map::altimeter2 => &self.altimeter2,
            id_map::altimeter3 => &self.altimeter3,

            #[cfg(test)]
            id_map::__test0 => &self.__test0,
            #[cfg(test)]
            id_map::__test1 => &self.__test1,            
            #[cfg(test)]
            id_map::__test2 => &self.__test2,
            #[cfg(test)]
            id_map::__test3 => &self.__test3,
            #[cfg(test)]
            id_map::__test4 => &self.__test4,

            _ => return None
        })
    }
}