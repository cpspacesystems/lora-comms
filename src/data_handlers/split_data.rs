use std::{cell::RefCell, collections::HashMap, ops::Range, rc::Rc, time};

use by_address::ByThinAddress;

use crate::{common::{AsRc, BufferType, assert_np}, common_config::{self, PACKER_MAX_SIZE}, data_handlers::{DataConsumer, DataProducer}, errors, pubsub};

pub struct Producer {
    size: usize,
    fragment_idx: usize,
    source: Rc<RefCell<SplitSource>>,
}
impl Producer {
    pub fn split_producer(producer: Rc<RefCell<dyn DataProducer>>) -> Vec<Rc<RefCell<dyn DataProducer>>> {
        let total_size = producer.borrow().get_size();
        let source_rc = SplitSource { 
            producer,
            has_data: false,
            data_fragments: Vec::new(),
        }.as_rc();
        let mut source = source_rc.borrow_mut();

        // split the source into a bunch of sections
        // this algorithm must maintain behavior parity as the one in Consumer
        let mut splits: Vec<Rc<RefCell<dyn DataProducer>>> = Vec::new();
        let mut fake_data = Vec::with_capacity(total_size);
        fake_data.resize(total_size, 0);
        let (chunks, remainder) = fake_data.as_chunks::<PACKER_MAX_SIZE>();
        let mut idx = 0;
        for _ in chunks {
            source.data_fragments.push((true, Vec::with_capacity(PACKER_MAX_SIZE), PACKER_MAX_SIZE));
            splits.push(Self::new(PACKER_MAX_SIZE, idx, source_rc.clone()).as_rc());
            idx += 1;
        }
        if !remainder.is_empty() {
            source.data_fragments.push((true, Vec::with_capacity(PACKER_MAX_SIZE), remainder.len()));
            splits.push(Self::new(remainder.len(), idx, source_rc.clone()).as_rc());
        }

        splits
    }

    pub(self) fn new(size: usize, idx: usize, source: Rc<RefCell<SplitSource>>) -> Self {
        Self { size, fragment_idx: idx, source }
    }
}

impl DataProducer for Producer {
    fn produce(&mut self) -> Result<Option<BufferType>, errors::AnyError> {
        self.source.borrow_mut().produce(self.fragment_idx)
    }

    fn get_size(&self) -> usize {
        self.size
    }
}

pub struct Consumer {
    destination: Rc<RefCell<SplitDest>>,
    size: usize,
    consume_id: usize,
    fragment_idx: usize,
}
impl Consumer {
    pub fn split_consumer(consumer: Rc<RefCell<dyn DataConsumer>>) -> Vec<Rc<RefCell<dyn DataConsumer>>> {
        let total_size = { consumer.borrow().get_size() };
        let dest_rc = SplitDest {
            consumer,
            data_fragments: Vec::new(),
        }.as_rc();
        let mut dest = dest_rc.borrow_mut();

        // split the destination into a bunch of sections
        // this algorithm must maintain behavior parity as the one in Producer
        let mut splits: Vec<Rc<RefCell<dyn DataConsumer>>> = Vec::new();
        let mut fake_data = Vec::with_capacity(total_size);
        fake_data.resize(total_size, 0);
        let (chunks, remainder) = fake_data.as_chunks::<PACKER_MAX_SIZE>();
        let mut idx = 0;
        for _ in chunks {
            dest.data_fragments.push((0, BufferType::new()));
            splits.push(Self::new(PACKER_MAX_SIZE, idx, dest_rc.clone()).as_rc());
            idx += 1;
        }
        if !remainder.is_empty() {
            dest.data_fragments.push((0, BufferType::new()));
            splits.push(Self::new(remainder.len(), idx, dest_rc.clone()).as_rc());
        }

        splits
    }

    pub(self) fn new(size: usize, idx: usize, dest: Rc<RefCell<SplitDest>>) -> Self {
        Self { destination: dest, size, consume_id: 0, fragment_idx: idx }
    }
}
impl DataConsumer for Consumer {
    fn consume(&mut self, buffer: BufferType) -> Result<(), errors::AnyError> {
        self.consume_id = self.consume_id.wrapping_add(1);
        if buffer.len() != self.size {
            Err(format!("Split Consumer expected size of {}, but got {}", self.size, buffer.len()).into())
        } else {
            self.destination.borrow_mut().consume(self.consume_id, self.fragment_idx, buffer)
        }
    }

    fn get_size(&self) -> usize {
        self.size
    }
}

struct SplitDest {
    consumer: Rc<RefCell<dyn DataConsumer>>,
    data_fragments: Vec<(usize, BufferType)>
}
impl SplitDest {
    pub fn consume(&mut self, id: usize, fragment_idx: usize, fragment: BufferType) -> Result<(), errors::AnyError> {
        self.data_fragments[fragment_idx] = (id, fragment);

        // check if all fragments have the same id => aka all fragments have been received
        let mut i = self.data_fragments.iter();
        let example = i.next().expect("Expected at least one fragment in split's data fragments!");
        // NOTE: We can not destinguish between lost fragments or fragments not yet arrived without expensive lookups and checks
        if i.all(|(frag_id, _)| *frag_id == example.0) {
            // construct consolidated data
            let mut consumer = self.consumer.borrow_mut();
            let mut consolidated = BufferType::with_capacity(consumer.get_size());
            for (_, frag) in &mut self.data_fragments {
                consolidated.append(frag);
            }
            // consume data
            consumer.consume(consolidated)
        } else {
            // This either means that fragments have not yet arrived, or fragments were lost
            Ok(())
        }
    }
}

struct SplitSource {
    producer: Rc<RefCell<dyn DataProducer>>,
    has_data: bool,
    data_fragments: Vec<(bool, BufferType, usize)>
}
impl SplitSource {
    pub(self) fn produce(&mut self, idx: usize) -> Result<Option<BufferType>, errors::AnyError> {
        // check if we need to produce new data
        if self.data_fragments.iter().all(|(taken, _, _)| *taken) {
            let data = match self.producer.borrow_mut().produce() {
                Ok(Some(d)) => { 
                    self.has_data = true;
                    d
                },
                Ok(None) => {
                    self.has_data = false;
                    self.data_fragments.iter_mut().for_each(|(taken, _, _)| *taken = false);
                    return Ok(None);
                }
                Err(e) => 
                    return Err(format!("Split Source encountered error while producing up stream: {}", e).into()),
            };
            // split data
            let (chunks, remainder) = data.as_chunks::<PACKER_MAX_SIZE>();
            if chunks.len() + ( if !remainder.is_empty() { 1 } else { 0 }) != self.data_fragments.len() {
                return Err(format!("Split Source expected {} chunks, but got {} chunks!", self.data_fragments.len(), chunks.len() + 1).into());
            }
            // update frags
            chunks.iter()
                .zip(&mut self.data_fragments)
                .for_each(|(c, (taken, frag, size))| {
                    debug_assert!(c.len() == *size);
                    *taken = false;
                    frag.clear();
                    frag.extend(c.iter());
                })
            ;
            if !remainder.is_empty() {
                let (taken, frag, size) = self.data_fragments.last_mut().expect("Split Source: Expected last fragment to exist!");
                debug_assert!(*taken == true);
                debug_assert!(remainder.len() == *size);
                *taken = false;
                frag.clear();
                frag.extend(remainder.iter());
            }
        }

        // return data fragment
        let (taken, frag, _) = &mut self.data_fragments[idx];
        if *taken {
            return Err("Split Source: Fragment already taken!".into());
        } 
        *taken = true;

        if !self.has_data {
            Ok(None)
        } else {
            Ok(Some(std::mem::take(frag)))
        }
    }
}



