use crate::byte_ops::data_section::{create_data_section, decode_data_section};

mod byte_ops;

#[tokio::main]
async fn main() {
    println!("Starting!"); 

    let data = b"hello world".to_vec(); 

    let buffer = create_data_section(byte_ops::types::flatbuffers::ALITITUDE, data);

    let decoded = decode_data_section(buffer).unwrap(); 

    println!("{}, {:?}", decoded.dtype, decoded.bytes);


}
