mod packet;
mod configure;
mod common;
mod error;
mod publisher;
mod subscriber;
mod data_handlers;

//lora radio code: tell zenoh when to get new data
//goes through everything it needs to get (newest version of data) and puts it into flatbuffers
//put flatbuffers into packets
//send packets into lora
//schedule packet to be sent over lora
fn main() {
    println!("Program starting");

    let mut f_exit = false;
    while !f_exit {
        // grab stuff from zenoh and convert it to data section
        // send said packet to LR1100 interface 
        // configure LR1100 
        // send bytes
        //
    }; 
}



