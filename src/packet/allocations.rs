use std::{collections::HashMap, sync::LazyLock};

/// common type for all data section types
#[derive(Debug)]
#[derive(Copy, Clone)]
#[derive(PartialEq)]
pub struct DataSectionType {
    /// data type identifier
    pub id: u8,
    /// data type name
    pub name: &'static str,
    /// bytes in data section (0 if flexiable data section)
    pub size: usize
}

/// id allocations, please follow these allocations otherwise undefined behavior
pub mod type_allocations {
    pub const RESERVED: std::ops::Range<u8> = 0..10;
    pub const DATA: std::ops::RangeInclusive<u8> = 10..=255;
}

pub struct IDAllocMap {
    name_map: HashMap<&'static str, DataSectionType>,
    id_map: HashMap<u8, DataSectionType>
}
impl IDAllocMap {
/////////////////////
/// CONFIGURATION ///
/////////////////////

    /// ids allocated here are: [10, 100)
    fn flatbuffers(&mut self) -> &mut Self {
        self
        .ds(10, "altitude", 48)
        .ds(11, "gyro", 20)
    }

    /// ids allocated here are: [0, 10)
    fn reserved(&mut self) -> &mut Self {
        self
        .ds(0, "reset", 1)
        .ds(1, "indicator_time_gps", 9)


        .ds(5, "req_change_link_size", 20)
        .ds(6, "req_change_link_fec_cr", 20)

        .ds(8, "ack", 20)
        .ds(9, "indicator_eot", 3)    
    }


    /// ids allocated here are [250, 256)
    #[cfg(test)]
    fn test(&mut self) -> &mut Self {
        self
        .ds(250, "test0", 0)
        .ds(251, "test1", 3)
        .ds(252, "test2", 11) // hello world
        .ds(253, "test3", 64)
        .ds(254, "test4", 128)
        // .ds(255, "test5", 200) // 255 to be non existant for tests
    }
/////////////////////////
/// END CONFIGURATION ///
/////////////////////////

    /// shorthand for adding new DataSectionTypes
    pub fn ds(&mut self, id: u8, name: &'static str, size: usize) -> &mut Self {
        let data = DataSectionType {id, name, size};
        self.id_map.insert(id, data);
        self.name_map.insert(name, data);
        self
    }

    /// initializes a new IDAllocMap
    pub fn init() -> Self {
        let mut this: IDAllocMap = IDAllocMap {
            name_map: HashMap::new(),
            id_map: HashMap::new()
        };
        this.reserved();
        this.flatbuffers();
        
        #[cfg(test)]
        this.test();

        this
    }
    /// gets the dtype corrosponding to name (!panic-able)
    pub fn by_name(&self, name: &'static str) -> DataSectionType {
        return self.name_map[name];
    }
    /// gets the dtype corrosponding to id (!panic-able)
    pub fn by_id(&self, id: &u8) -> DataSectionType {
        return self.id_map[id];
    }
    /// try to get the dtype corrosponding to id
    pub fn try_id(&self, id: &u8) -> Option<DataSectionType> {
        self.id_map.get(id).copied()
    }
}

// the default id mappings
pub static DEFAULT_ALLOC_MAP: std::sync::LazyLock<IDAllocMap> = std::sync::LazyLock::new(|| {
    IDAllocMap::init()
});
/// gets the dtype corrosponding to name in the default id mappings (!panic-able)
pub fn by_name(name: &'static str) -> DataSectionType {
    DEFAULT_ALLOC_MAP.by_name(name)
}
/// gets the dtype corrosponding to id in the default id mappings (!panic-able)
pub fn by_id(id: &u8) -> DataSectionType {
    DEFAULT_ALLOC_MAP.by_id(id)
}
/// try to get the dtype corrosponding to id in the default id mappings 
pub fn try_id(id: &u8) -> Option<DataSectionType> {
    DEFAULT_ALLOC_MAP.try_id(id)
}