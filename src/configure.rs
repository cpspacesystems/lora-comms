use crate::{packet::record::{RecordMap, consumer_nop}, producer_size};

/// This file maps all data producers and consumers with corrosponding
/// names and packet ids used transmission.  

///////////////////
// CONFIGURATION //
///////////////////

// id allocation overview
pub fn configure(map: &mut RecordMap) {
    reserved(map); // [0, 10)
    flatbuffers(map); // [0, 100)

    #[cfg(test)]
    test(map); //[250, 256)
}

// ids allocated here are: [10, 100)
fn flatbuffers(map: &mut RecordMap){
    map
    .ds(10, "altitude", 48, producer_size!(48), consumer_nop())
    .ds(11, "gyro", 20, producer_size!(20), consumer_nop());
}

// ids allocated here are: [0, 10)
fn reserved(map: &mut RecordMap) {
    map
    .ds(0, "reset", 1, producer_size!(1), consumer_nop())
    .ds(1, "indicator_time_gps", 9, producer_size!(9), consumer_nop())

    .ds(5, "req_change_link_size", 20, producer_size!(20), consumer_nop())
    .ds(6, "req_change_link_fec_cr", 20, producer_size!(20), consumer_nop())

    .ds(8, "ack", 20, producer_size!(20), consumer_nop())
    .ds(9, "indicator_eot", 3, producer_size!(3), consumer_nop())
    ;
}


// ids allocated here are [250, 256)
#[cfg(test)]
fn test(map: &mut RecordMap) {
    use crate::packet::record::producer_nop;

    map
    .ds(250, "test0", 0, producer_nop(), consumer_nop())
    .ds(251, "test1", 3, producer_size!(3), consumer_nop())
    .ds(252, "test2", 11, producer_size!(11), consumer_nop()) // hello world
    .ds(253, "test3", 64, producer_size!(64), consumer_nop())
    .ds(254, "test4", 128, producer_size!(128), consumer_nop())
    // 255 to be non existant for tests
    ;
}
///////////////////////
// END CONFIGURATION //
///////////////////////