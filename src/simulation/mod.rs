
mod data_generator;

static TOKIO_RT: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
pub fn get_tokio_or_init() -> tokio::runtime::Handle {
    if let Ok(h) = tokio::runtime::Handle::try_current() {
        h
    } else {
        TOKIO_RT.get_or_init(|| {
            tokio::runtime::Runtime::new().unwrap()
        }).handle().clone()
    }
}

#[cfg(feature = "hardware_attached_full_system")]
pub mod hardware_attached {
    use std::{process, thread::sleep, time};

    use log::info;

use crate::{common_config, pubsub::{self, Connection}, simulation::{data_generator::DataSource, get_tokio_or_init}};

    pub fn spawn_tism(size: usize, path: String) {
        info!(target: "HWAS", "Spawn tism pub size of {} at {}", size, path);

        std::thread::spawn(move || {
            let publisher = pubsub::tism::TISMConnection::new().publish(size, path);

            spawn_data_source(size, publisher);
        });
    }

    pub fn spawn_zenoh(size: usize, path: String) {
        info!(target: "HWAS", "Spawn Zenoh pub size of {} at {}", size, path);

        std::thread::spawn(move || {
            let publisher = pubsub::zenoh::ZenohConnection::new().publish(size, path);

            spawn_data_source(size, publisher);
        });
    }

    fn spawn_data_source(size: usize, mut publisher: impl pubsub::Publisher) {
        let mut generator = DataSource::new(size);
        
        let mut last_slept_time = time::Instant::now();
        loop {
            publisher.publish(generator.generate()).expect("HWAS: Failed to publish!");
            
            let now = time::Instant::now();
            let delta = now.saturating_duration_since(last_slept_time);
            if delta < common_config::HWAS_PUBLISH_RATE {
                std::thread::sleep(delta);
            }
            last_slept_time = now;
        }
    }
}