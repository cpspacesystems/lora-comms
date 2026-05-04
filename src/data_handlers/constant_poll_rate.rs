use std::{cell::RefCell, rc::Rc, time};

use crate::{common::BufferType, data_handlers::{DataConsumer, DataProducer}, errors, pubsub};

pub struct Producer {
    rate: time::Duration,
    producers: Vec<Rc<RefCell<dyn DataProducer>>>,
    total_size: usize,

    last_pulled_time: time::Instant,
}

impl Producer {
    pub fn new(rate: time::Duration, producers: Vec<Rc<RefCell<dyn DataProducer>>>) -> Self {
        let total_size: usize = producers.iter()
            .map(|x| x.borrow().get_size())
            .sum();

        Self { rate, producers, total_size, last_pulled_time: time::Instant::now().checked_sub(rate).expect("Expected high enough system time to validly contain rate in the past.") }
    }
}

impl DataProducer for Producer {
    fn produce(&mut self) -> Result<Option<crate::common::BufferType>, crate::errors::AnyError> {
        let now = time::Instant::now();
        if now.saturating_duration_since(self.last_pulled_time) < self.rate {
            return Ok(None);
        }

        let mut data = BufferType::with_capacity(self.total_size);
        for rc in &self.producers {
            let mut p = rc.try_borrow_mut()?;
            match p.produce() {
                Ok(Some(mut d)) => data.append(&mut d),
                Ok(None) => data.resize(data.len() + p.get_size(), 0),
                Err(e) => { 
                    println!("Encountered error while producing: {}", e);
                    data.resize(data.len() + p.get_size(), 0)
                },
            }
        };

        if data.len() != self.total_size { 
            Err(errors::InvalidData(format!("ConstantPollRate producer expected data size of {}, but got {}!", self.total_size, data.len())).into())
        } else {
            self.last_pulled_time = now;
            Ok(Some(data))
        }
    }

    fn get_size(&self) -> usize {
        self.total_size
    }
}

pub struct Consumer {
    consumers: Vec<Rc<RefCell<dyn DataConsumer>>>,
    total_size: usize,
}

impl Consumer {
    pub fn new(consumers: Vec<Rc<RefCell<dyn DataConsumer>>>) -> Self {
        let total_size: usize = consumers.iter()
            .map(|x| x.borrow().get_size())
            .sum();

        Self { consumers, total_size }
    }
}

impl DataConsumer for Consumer {
    fn consume(&mut self, mut buffer: BufferType) -> Result<(), errors::AnyError> {
        if buffer.len() != self.get_size() {
            return Err(errors::InvalidData(format!("ConstantPollRate consumer expected data size of {}, but got {}", self.total_size, buffer.len())).into());
        }

        // revese consuming the buffer
        for c in self.consumers.iter().rev() {
            let mut consumer = c.borrow_mut();
            
            let offset = buffer.len() - consumer.get_size();
            let nb = buffer.split_off(offset);
            if let Err(e) = consumer.consume(nb) {
                println!("Encountered error while consuming constant poll rate data at offset {}: {}", offset, e);
            };
        }

        Ok(())
    }

    fn get_size(&self) -> usize {
        self.total_size
    }
}





