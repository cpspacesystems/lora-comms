pub type DataSectionType = u8;
pub type BufferType = Vec<u8>; 
pub type GPSTime = u64; 

// id allocations, please follow these allocations otherwise undefined behavior
pub mod type_allocations {
    pub const RESERVED: std::ops::Range<u8> = 1..10;
    pub const FLATBUFFERS: std::ops::Range<u8> = 10..100;
}

// ids allocated are: [10, 100)
pub mod flatbuffers {
    use crate::packet::types::DataSectionType;
    pub const ALITITUDE: DataSectionType = 10; 
    pub const GYRO: DataSectionType = 11;
    pub const PUMP: DataSectionType = 12;
}

// ids allocated are: [0, 10)
pub mod reserved {
    use crate::packet::types::DataSectionType;

    pub const RESET: DataSectionType = 0;
    pub const INDICATOR_TIME_GPS: DataSectionType = 1;


    
    pub const REQ_CHANGE_LINK_SIZE: DataSectionType = 5;
    pub const REQ_CHANGE_LINK_FEC_CR: DataSectionType = 6;

    pub const ACK: DataSectionType = 8;
    pub const INDICATOR_EOT: DataSectionType = 9; 
}

