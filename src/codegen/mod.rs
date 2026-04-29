use std::time;


pub mod parse_data_def;
pub mod generator;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Direction {
    RocketToGround,
    GroundToRocket,
}
impl AsRef<str> for Direction {
    fn as_ref(&self) -> &str {
        match &self {
            Direction::RocketToGround => "rocket",
            Direction::GroundToRocket => "ground",
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum NetworkType {
    TISM, Zenoh
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PollRate {
    ASAP,
    OnChange,
    FixedRate(time::Duration),
}

#[derive(Debug, PartialEq, Clone)]
pub struct DataDefEntry {
    rate: PollRate,
    size: u64,
    network: NetworkType,
    source: String,
    destination: String,
}