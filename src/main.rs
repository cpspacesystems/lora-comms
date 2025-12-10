use crate::packet::builder;
mod packet;
mod publisher;
mod subscriber;

//lora radio code: tell zenoh when to get new data
//goes through everything it needs to get (newest version of data) and puts it into flatbuffers
//put flatbuffers into packets
//send packets into lora
//schedule packet to be sent over lora
fn main() {
    println!("Program starting");

    let mut f_exit = false;
    while !f_exit {
        let mut builder = packet::builder::PacketBuilder::new();

        // grab stuff from zenoh and convert it to data section
        // builder.add(data_type, data)
        
        let raw_packet = builder.build(); 
        // send said packet to LR1100 interface 
        // configure LR1100 
        // send bytes
        //
    }; 
}



