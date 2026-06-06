use std::{cell::{OnceCell, RefCell}, collections::HashMap, rc::Rc, time};

use crate::{common::AsRc, common_config, config::format::{self, Entry, Network}, data_handlers::{self, DataConsumer, DataProducer}, pubsub::{self, Connection, zenoh::ZenohPublisher}, simulation};
pub struct IDProvider {
    consuming_id: u8,
    producing_id: u8,
}
impl IDProvider {
    pub fn new_ground() -> Self {
        Self { consuming_id: 20, producing_id: 120 }
    }

    pub fn new_rocket() -> Self {
        Self { consuming_id: 120, producing_id: 20 }
    }

    pub(super) fn next_producer(&mut self) -> u8 {
        self.producing_id += 1;
        self.producing_id
    }

    pub(super) fn next_consumer(&mut self) -> u8 {
        self.consuming_id += 1;
        self.consuming_id
    }
}

pub struct Generator<T, Z>
    where   T: Fn() -> crate::pubsub::tism::TISMConnection,
            Z: Fn() -> crate::pubsub::zenoh::ZenohConnection
{
    network_tism: Option<crate::pubsub::tism::TISMConnection>,
    network_tism_initializer: T,
    network_zenoh: Option<crate::pubsub::zenoh::ZenohConnection>,
    network_zenoh_initializer: Z,
    /// map to DataProducer variables to be added to producer mgr
    producers: Vec<Rc<RefCell<dyn data_handlers::DataProducer>>>,
    /// map to DataConsumer variables to be added to consumer mgr
    consumers: Vec<Rc<RefCell<dyn data_handlers::DataConsumer>>>,
    /// map of poll rate duration to producers to be pulled at that duration
    rate_map_producers: HashMap<time::Duration, Vec<(usize, Rc<RefCell<dyn data_handlers::DataProducer>>)>>, 
    /// map of poll rate duration to consumers
    rate_map_consumers: HashMap<time::Duration, Vec<(usize, Rc<RefCell<dyn data_handlers::DataConsumer>>)>> 
}

impl<T, Z> Generator<T, Z>
    where   T: Fn() -> crate::pubsub::tism::TISMConnection,
        Z: Fn() -> crate::pubsub::zenoh::ZenohConnection
{
    pub fn new(
        tism_initializer: T, 
        zenoh_initializer: Z
    ) -> Self {
        Self {
            network_tism: None,
            network_tism_initializer: tism_initializer,
            network_zenoh: None, 
            network_zenoh_initializer: zenoh_initializer,
            producers: Vec::new(), consumers: Vec::new(),
            rate_map_producers: HashMap::new(), rate_map_consumers: HashMap::new()
        }
    }

    /// entrys for the same pair of producer and consumer are expected to have add_entry_* be called adjecent to each other in their respective generators
    pub fn add_entry_producing(&mut self, entry: &Entry) {
        #[cfg(feature = "hardware_attached_full_system")]
        self.hwas_spawn_publisher(entry.size.into(), &entry.source_network, &entry.source_path);
        
        let p = self.create_producer(&entry.source_network, entry.size.into(), entry.rate == format::PollRate::OnChange, &entry.source_path);
        for s in self.optionally_split_producer(entry.size.into(), p) {
            match entry.rate {
                format::PollRate::ASAP | format::PollRate::OnChange => self.producers.push(s),
                format::PollRate::FixedRate(duration) => self.rate_map_producers.entry(duration).or_default().push((entry.size.into(), s)),
            };
        };
    }

    pub fn add_producing_entries(&mut self, entries: &Vec<Entry>) {
        for e in entries {
            self.add_entry_producing(e);
        }
    }

    /// entrys for the same pair of producer and consumer are expected to have add_entry_* be called adjecent to each other in their respective generators
    pub fn add_entry_consuming(&mut self, entry: &Entry) {
        let c = self.create_consumer(&entry.destination_network, entry.size.into(), &entry.destination_path);

        for s in self.optionally_split_consumer(entry.size.into(), c) {
            match entry.rate {
                format::PollRate::ASAP | format::PollRate::OnChange => self.consumers.push(s),
                format::PollRate::FixedRate(duration) => self.rate_map_consumers.entry(duration).or_default().push((entry.size.into(), s)),
            };
        };
    }

    pub fn add_consuming_entries(&mut self, entries: &Vec<Entry>) {
        for e in entries {
            self.add_entry_consuming(e);
        }
    }

    pub fn finalize(mut self, id_provider: &mut IDProvider, 
        producer_mgmt: &mut crate::data_handlers::ProducerManager, 
        consumer_mgmt: &mut crate::data_handlers::ConsumerManager,
    ) -> 
        (Option<pubsub::tism::TISMConnection>, Option<pubsub::zenoh::ZenohConnection>) 
    {
        let p = std::mem::take(&mut self.rate_map_producers);
        let prod_packed = self.pack_rate_map(p);
        self.add_packed_poll_rate_producers(prod_packed);

        let c = std::mem::take(&mut self.rate_map_consumers);
        let cons_packed = self.pack_rate_map(c);
        self.add_packed_poll_rate_consumers(cons_packed);

        for p in self.producers {
            producer_mgmt.add_by_id(id_provider.next_producer(), p);
        }
        for c in self.consumers {
            consumer_mgmt.add_by_id(id_provider.next_consumer(), c);
        }

        (self.network_tism, self.network_zenoh)
    }

    fn pack_rate_map<V>(&mut self, map: HashMap<time::Duration, Vec<(usize, V)>>) -> Vec<(time::Duration, Vec<V>)> {
        let mut packed = Vec::new();
        
        // sort the hash map cuz HashMaps have no guranteed iteration order and we depend on that for ids to match
        let mut sorted: Vec<(time::Duration, Vec<(usize, V)>)> = map.into_iter().collect();
        sorted.sort_by(|(a, _), (b, _)| a.cmp(b));

        for (d, v) in sorted {
            let mut packs: Vec<(usize, Vec<V>)> = Vec::new();

            'outer: for (size, value) in v {
                for (len, pack) in &mut packs {
                    if *len + size < crate::common_config::PACKER_MAX_SIZE {
                        *len += size;
                        pack.push(value);
                        continue 'outer;
                    }
                }

                // no pack big enough for pub to fit into, make new pack
                packs.push((size, vec![value]));
            }

            for (_, pack) in packs {
                packed.push((d, pack));
            }
        }
        packed
    }

    fn network_get_tism(&mut self) -> &mut pubsub::tism::TISMConnection {
        if self.network_tism.is_none() { 
            self.network_tism = Some((self.network_tism_initializer)()); 
        };

        self.network_tism.as_mut().unwrap()
    }

    fn network_get_zenoh(&mut self) -> &mut pubsub::zenoh::ZenohConnection {
        if self.network_zenoh.is_none() { 
            self.network_zenoh = Some((self.network_zenoh_initializer)()); 
        };

        self.network_zenoh.as_mut().unwrap()    
    }

    fn optionally_split_producer(&self, size: usize, producer: Rc<RefCell<dyn DataProducer>>) -> Vec<Rc<RefCell<dyn DataProducer>>>{
        if size > common_config::PACKER_MAX_SIZE {
            data_handlers::split_data::Producer::split_producer(producer)
        } else {
            vec![producer]
        }
    }

    fn optionally_split_consumer(&self, size: usize, consumer: Rc<RefCell<dyn DataConsumer>>) -> Vec<Rc<RefCell<dyn DataConsumer>>>{
        if size > common_config::PACKER_MAX_SIZE {
            data_handlers::split_data::Consumer::split_consumer(consumer)
        } else {
            vec![consumer]
        }
    }

    fn create_producer(&mut self, network: &Network, size: usize, is_on_change: bool, name: impl AsRef<str>) -> std::rc::Rc<std::cell::RefCell<dyn data_handlers::DataProducer>>  {
        match network {
            Network::TISM => {
                if is_on_change { 
                    data_handlers::raw_pubsub::Producer::new(size, self.network_get_tism().subscribe_on_change(name), true).as_rc() 
                } else {   
                    data_handlers::raw_pubsub::Producer::new(size, self.network_get_tism().subscribe(name), true).as_rc() 
                }
            },
            Network::Zenoh => {
                if is_on_change { 
                    data_handlers::raw_pubsub::Producer::new(size, self.network_get_zenoh().subscribe_on_change(name), true).as_rc() 
                } else {   
                    data_handlers::raw_pubsub::Producer::new(size, self.network_get_zenoh().subscribe(name), true).as_rc() 
                }
            },
        }
    }

    fn create_consumer(&mut self, network: &Network, size: usize, name: impl AsRef<str>) -> std::rc::Rc<std::cell::RefCell<dyn data_handlers::DataConsumer>>  {
        match network {
            Network::TISM => {
                data_handlers::raw_pubsub::Consumer::new(size, self.network_get_tism(), name, true).as_rc()
            },
            Network::Zenoh => {
                data_handlers::raw_pubsub::Consumer::new(size, self.network_get_zenoh(), name, true).as_rc()
            },
        }
    }


    fn add_packed_poll_rate_producers(&mut self, packed_prods: Vec<(time::Duration, Vec<Rc<RefCell<dyn DataProducer>>>)>) {
        for (d, p) in packed_prods {
            self.producers.push(data_handlers::constant_poll_rate::Producer::new(d, p).as_rc());
        }
    }

    fn add_packed_poll_rate_consumers(&mut self, packed_cons: Vec<(time::Duration, Vec<Rc<RefCell<dyn DataConsumer>>>)>) {
        for (_d, p) in packed_cons {
            self.consumers.push(data_handlers::constant_poll_rate::Consumer::new(p).as_rc());
        }
    }

    #[cfg(feature = "hardware_attached_full_system")]
    fn hwas_spawn_publisher(&mut self, size: usize, network: &Network, path: &str) {
        match network {
            Network::TISM => simulation::hardware_attached::spawn_tism(size, path.to_string()),
            Network::Zenoh => simulation::hardware_attached::spawn_zenoh(size, path.to_string()),
        }
    }
}
