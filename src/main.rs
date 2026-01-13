
mod publisher;
mod subscriber;
fn main() {
    let p = publisher::Pubs::new("test".to_string());
    p.send_str("Hello World");
    //let s = subscriber::Subs::new("test".to_string());
    //s.get();
}


//lora radio code: tell zenoh when to get new data
//goes through everything it needs to get (newest version of data) and puts it into flatbuffers
//put flatbuffers into packets
//send packets into lora
//schedule packet to be sent over lora

// 8 bit enum instead of string key



// real struct and simulation struct, interface


