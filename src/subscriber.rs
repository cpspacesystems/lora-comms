use zenoh::Wait;

fn subscriber() {
    let session = zenoh::open(zenoh::Config::default()).wait().unwrap();
    let subscriber = session.declare_subscriber("key/expr").wait().unwrap();
    while let Ok(sample) = subscriber.recv() {
        println!("Received: {:?}", sample.payload());
    }
    
}