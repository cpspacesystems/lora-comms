use std::collections::HashMap;

use toml::Table;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TLF {
    rocket_to_ground: Vec<Entry>,
    ground_to_rocket: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
pub struct Entry {
    rate: PollRate,
    size: u8,
    source_network: Network,
    source_path: String,
    destination_network: Network,
    destination_path: String,
}

#[derive(Debug, Deserialize)]
pub enum PollRate {
    Fixed1Hz,
    FixedHalfHz,
    Fixed10Hz,
    ASAP,
    OnChange,
}

#[derive(Debug, Deserialize)]
pub enum Network {
    TISM,
    Zenoh,
}
