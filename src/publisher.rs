use zenoh::Wait;

fn publisher() {
    let session = zenoh::open(zenoh::Config::default()).wait().unwrap();
    let publisher = session.declare_publisher("key/expr").wait().unwrap();
    publisher.put("Hello World").wait().unwrap();
}
