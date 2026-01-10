mod sx1302;
mod packet;
mod configure;
mod common;
mod error;
mod publisher;
mod subscriber;
mod data_handlers;

fn build_packet() {
    let p1 = data_handlers::altimeter::Producer::new("/test/altimeter1".into());

}

fn main() {
    println!("Program starting");

    // configure zenoh
    // start zenoh
    // configure sx1302
    // start sx1302
    let mut f_exit = false;
    while !f_exit {
        // try fetch packets from sx1302
        // if have packets, convert into flat bufs
        // publish to zenoh
        // sleep by what ever ms for new packets to appear
    }; 
}



