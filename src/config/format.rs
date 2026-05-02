use std::{collections::HashMap, str::FromStr, time};

use toml::Table;
use serde::Deserialize;

use crate::errors::AnyError;

#[derive(Debug, Deserialize)]
pub struct TLF {
    pub rocket_to_ground: Vec<Entry>,
    pub ground_to_rocket: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
pub struct Entry {
    pub rate: PollRate,
    pub size: u8,
    pub source_network: Network,
    pub source_path: String,
    pub destination_network: Network,
    pub destination_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum PollRate {
    FixedRate(time::Duration), // xHz
    ASAP,
    OnChange,
}

#[derive(Debug, Deserialize)]
pub enum Network {
    TISM,
    Zenoh,
}

impl FromStr for PollRate {
    type Err = AnyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ASAP" => Ok(Self::ASAP),
            "OnChange" => Ok(Self::OnChange),
            s => Ok(
                Self::FixedRate(time::Duration::from_secs_f64(s.
                    strip_suffix("Hz").ok_or("Expected Suffix Hz for FixedPollRate.")?
                    .parse::<f64>()?
                ))
            )
        }
    }
}
impl TryFrom<String> for PollRate {
    type Error = AnyError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<PollRate> for String {
    fn from(value: PollRate) -> Self {
        match value {
            PollRate::FixedRate(duration) => format!("{}Hz", duration.as_secs_f64()),
            PollRate::ASAP => "ASAP".to_string(),
            PollRate::OnChange => "OnChange".to_string(),
        }
    }
}