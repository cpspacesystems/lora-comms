use std::{time::Duration, usize};

use crate::{common::BufferType, errors::AnyError};

pub mod zenoh;
pub mod tism;

pub trait Connection { 
    type S: Subscriber; type SC: Subscriber; type P: Publisher;
    fn subscribe(&mut self, path: impl AsRef<str>) -> Self::S;
    fn subscribe_on_change(&mut self, path: impl AsRef<str>) -> Self::SC;
    fn publish(&mut self, size: usize, path: impl AsRef<str>) -> Self::P;
}

pub trait Publisher {
    fn publish(&mut self, data: BufferType) -> Result<(), AnyError>;
}

pub trait Subscriber {
    fn get_path(&self) -> impl AsRef<str>;
    fn get(&mut self) -> Result<Option<BufferType>, AnyError>;
    fn get_time_micros(&mut self) -> Result<Option<Duration>, crate::errors::AnyError>;
}