pub type TypeID = u8;

#[repr(u8)]
#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(Eq, Hash, PartialEq)]
pub enum TypeIDs {
    // Reserved/Special Types
    // ids allocated here are: [0, 10)

    Reset = 1,
    QOSNotify = 2, // Quality of service notify

    ACK = 9,

    // User defined types
    // ids allocated here are: [10, 250)
    
    // produced by rocket -- [10, 200) 
    Altimeter1 = 10,
    Altimeter2 = 11,
    Altimeter3 = 12,

    ConstantPollRate1Hz = 21,
    ConstantPollRateHalfHz = 22,
    ConstantPollRate10Hz = 23,

    GPS1 = 30,

    FullPrepreppedTelemteryPacket = 101,
    RawData = 103,
    // produced by ground/mc -- [200, 250)
    SignalAbort = 201,
    SignalLaunch = 202,
    SignalDeployParachute = 203,

    // types for testing 
    // ids allocated here are [250, 256)
    #[cfg(test)]
    Test0 = 250,
    #[cfg(test)]
    Test1 = 251,
    #[cfg(test)]
    Test2 = 252,
    #[cfg(test)]
    Test3 = 253,
    #[cfg(test)]
    Test4 = 254,
    #[cfg(test)] // 255 must not be created in any of the conumer/producer mgs for tests to pass
    Test5 = 255,
}

impl From<TypeIDs> for TypeID {
    fn from(value: TypeIDs) -> Self {
        value as TypeID
    }
}
impl From<&TypeIDs> for TypeID {
    fn from(value: &TypeIDs) -> Self {
        *value as TypeID
    }
}