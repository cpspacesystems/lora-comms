use crate::packet::data_section::{create_data_section, decode_data_section};

mod packet;

#[tokio::main]
async fn main() {
    println!("Starting!"); 

    let data = b"hello world".to_vec(); 

    let buffer = create_data_section(packet::types::flatbuffers::ALITITUDE, data).unwrap();

    let decoded = decode_data_section(buffer).unwrap(); 

    println!("{}, {:?}", decoded.dtype, decoded.bytes);


}
