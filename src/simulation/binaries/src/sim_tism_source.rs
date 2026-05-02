use std::time;

use binaries::{DataSource, config};


fn main() {
    let mut args = std::env::args();
    args.next().expect("Program name should always exist, this should be impossible.");
    let size = args.next().expect("Expected size.").parse::<usize>().expect("Expected valid non zero number size.");
    let path = args.next().expect("Expected publshing path");

    let mut generator = DataSource::new(size);

    // let mut tism = tism::create(path.clone(), generator.generate())
    let mut tism = tism::dynamic::create(path.clone(), size)
        .expect("Should be able to create TISM allocation!");

    println!("Made tism allocation {} of size {}", path, size);

    let mut last_slept_time = time::Instant::now();
    loop {
        tism.write(generator.generate()).expect("Failed to write to TISM");

        // if let Ok(mut wr_lock) = tism.write_lock() {
        //     *wr_lock.as_mut() = generator.generate();
        // } else {
        //     panic!("Failed to write lock TISM");
        // }
        
        let now = time::Instant::now();
        let delta = now.saturating_duration_since(last_slept_time);
        if delta < config::PUBLISH_RATE {
            std::thread::sleep(delta);
        }
        last_slept_time = now;
    }
}