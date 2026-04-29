use std::usize;

use crate::{common::BufferType, errors::AnyError};

pub mod zenoh;
pub mod tism;

pub trait Connection { 
    type S: Subscriber; type SC: SubscriberOnChange; type P<const N: usize>: Publisher<N>;
    fn subscribe(&mut self, path: String) -> Self::S;
    fn subscribe_on_change(&mut self, path: String) -> Self::SC;
    fn publish<const N: usize>(&mut self, path: String) -> Self::P<N>;
}

pub trait Publisher<const N: usize> {
    fn publish(&mut self, data: BufferType) -> Result<(), AnyError>;
}

pub trait Subscriber {
    fn get(&mut self) -> Result<Option<BufferType>, AnyError>;
}

pub trait SubscriberOnChange {
    fn get_onchange(&mut self) -> Result<Option<BufferType>, AnyError>;
}
