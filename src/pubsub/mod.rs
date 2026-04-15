use crate::{common::BufferType, errors::AnyError};

pub mod zenoh;
pub mod tism;

pub trait Connection {
    type S: Subscriber; type P: Publisher;
    fn subscribe(&mut self, path: String) -> Self::S;
    fn publish(&mut self, path: String) -> Self::P;
}

pub trait Publisher {
    fn publish(&mut self, data: BufferType) -> Result<(), AnyError>;
}

pub trait Subscriber {
    fn get(&mut self) -> Result<Option<BufferType>, AnyError>;
}
