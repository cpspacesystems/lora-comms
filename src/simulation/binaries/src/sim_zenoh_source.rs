use std::time;

use binaries::{DataSource, config};
use zenoh::Wait;

fn main() {
    let mut args = std::env::args();
    args.next().expect("Program name should always exist, this should be impossible.");
    let size = args.next().expect("Expected size.").parse::<usize>().expect("Expected valid non zero number size.");
    let path = args.next().expect("Expected publshing path");

    let mut generator = DataSource::new(size);

    let session = zenoh::open(zenoh::Config::default()).wait().expect("Unable to create a zenoh instance.");
    let publisher = session.declare_publisher(path).wait().expect("Unable to create publisher for provided path.");

    let mut last_slept_time = time::Instant::now();
    loop {
        publisher.put(generator.generate()).wait().expect("Failed to publish to zenoh!");

        let now = time::Instant::now();
        let delta = now.saturating_duration_since(last_slept_time);
        if delta < config::PUBLISH_RATE {
            std::thread::sleep(delta);
        }
        last_slept_time = now;
    }
}