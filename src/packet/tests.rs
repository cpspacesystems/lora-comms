use bitint::bitint_literals;

use crate::{common_config, network_ids::TypeIDs};

use super::*;

#[test]
fn test_outgoing_packet_builder() {
    let producers = ProducerManager::new();
    let mut builder =  OutgoingFrameBuilder::new(&producers);

    assert!(builder.build(TSMCtrlInfo::default()).is_empty());

    assert!(matches!(builder.gather_by_id(TypeIDs::from_repr(255).unwrap()), Err(e) if e.is::<errors::GatherUnknownTypeError>()));

    assert_eq!(builder
        .gather_by_id(TypeIDs::Test1).unwrap()
        .gather_by_id(TypeIDs::from_repr(252).unwrap()).unwrap()
        .gather_by_id(TypeIDs::Test3).unwrap()
        .build(),
        [vec![0xFB], vec![0x00; 3], vec![0xFC], vec![0x00; 11], vec![0xFD], vec![0x00; 64]].concat() 
    );
}

#[test]
fn test_consume_incoming_packet() {
    let consumers = ConsumerManager::new();

    assert!(matches!(decode_incoming_packet(&consumers, 0, vec![common_config::LORA_REGONATION_CODE ^ 0x0, 0x0, 0xEE]), Err(e) if e.is::<errors::DecodeUnknownTypeError>()));
    assert!(decode_incoming_packet(&consumers, 0, vec![common_config::LORA_REGONATION_CODE ^ 0x0, 0x0]).is_ok());

    assert!(decode_incoming_packet(&consumers, 83, [
        TSMCtrlInfo::new(120_U7, true).to_wire(83),
        vec![0xFB], vec![0x00; 3], vec![0xFC], vec![0x00; 11], vec![0xFD], vec![0x00; 64]
    ].concat()).is_ok());

    assert!(decode_incoming_packet(&consumers, 83, [
        TSMCtrlInfo::new(120_U7, true).to_wire(81),
        vec![0xFB], vec![0x00; 3], vec![0xFC], vec![0x00; 11], vec![0xFD], vec![0x00; 64]
    ].concat()).is_err());
}
