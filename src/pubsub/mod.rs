use std::usize;

use crate::{common::BufferType, errors::AnyError};

pub mod zenoh;
pub mod tism;

pub trait Connection { 
    type S: Subscriber; type SC: SubscriberOnChange; type P: Publisher;
    fn subscribe(&mut self, path: impl AsRef<str>) -> Self::S;
    fn subscribe_on_change(&mut self, path: impl AsRef<str>) -> Self::SC;
    fn publish(&mut self, size: usize, path: impl AsRef<str>) -> Self::P;
}

pub trait Publisher {
    fn publish(&mut self, data: BufferType) -> Result<(), AnyError>;
}

pub trait Subscriber {
    fn get(&mut self) -> Result<Option<BufferType>, AnyError>;
}

pub trait SubscriberOnChange {
    fn get_onchange(&mut self) -> Result<Option<BufferType>, AnyError>;
}
