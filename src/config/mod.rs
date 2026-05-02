use std::{fs, path::Path};

use crate::{config::{format::{Entry, TLF}, generator::{Generator, IDProvider}}, errors::AnyError, pubsub::{tism::TISMConnection, zenoh::ZenohConnection}};



pub mod format;
pub mod generator;

pub fn parse(path: impl AsRef<Path>) -> Result<TLF, AnyError> {
    let data = fs::read_to_string(path)?;
    
    let parsed = toml::from_str::<TLF>(&data)?;
    
    Ok(parsed)
}