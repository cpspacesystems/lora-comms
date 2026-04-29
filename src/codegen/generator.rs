use std::{collections::HashMap, time};

use crate::codegen::{DataDefEntry, Direction, NetworkType, PollRate};

const PACKER_MAX_SIZE: u64 = 240;

pub struct Generator {
    direction: Direction,
    var_name_counter: u64, 
    need_tism: bool,
    need_zenoh: bool,
    /// map to already constructed lines of the function body
    constructed_body: Vec<String>,
    /// map to DataProducer variables to be added to producer mgr
    producers: Vec<String>,
    /// map to DataConsumer variables to be added to consumer mgr
    consumers: Vec<String>,
    /// map of poll rate duration to producers to be pulled at that duration
    rate_map_producers: HashMap<time::Duration, Vec<(u64, String)>>, 
    /// map of poll rate duration to consumers
    rate_map_consumers: HashMap<time::Duration, Vec<(u64, String)>> 
}

impl Generator {
    pub fn new(direction: Direction) -> Self {
        Self { direction, 
            var_name_counter: 0,
            need_tism: false, need_zenoh: false, 
            constructed_body: Vec::new(),
            producers: Vec::new(), consumers: Vec::new(),
            rate_map_producers: HashMap::new(), rate_map_consumers: HashMap::new()
        }
    }

    /// entrys for the same pair of producer and consumer are expected to have add_entry_* be called adjecent to each other in their respective generators
    pub fn add_entry_producing(&mut self, entry: &DataDefEntry) {
        let sub_name = self.map_construct_subscriber(&entry.rate, &entry.network, &entry.source);
        let p = self.make_producer_raw_pubsub(entry.size, sub_name);
        match entry.rate {
            PollRate::ASAP | PollRate::OnChange => self.producers.push(p),
            PollRate::FixedRate(duration) => self.rate_map_producers.entry(duration).or_default().push((entry.size, p)),
        };
    }

    /// entrys for the same pair of producer and consumer are expected to have add_entry_* be called adjecent to each other in their respective generators
    pub fn add_entry_consuming(&mut self, entry: &DataDefEntry) {
        let pub_name = self.construct_publisher(entry.size, &entry.network, &entry.destination);
        let c = self.make_consumer_raw_pubsub(entry.size, pub_name);
        match entry.rate {
            PollRate::ASAP | PollRate::OnChange => self.consumers.push(c),
            PollRate::FixedRate(duration) => self.rate_map_consumers.entry(duration).or_default().push((entry.size, c)),
        };
    }

    pub fn finalize(mut self) -> String {
        self.constructed_body.reserve(self.producers.len() + self.consumers.len() + self.rate_map_consumers.len() + self.rate_map_producers.len());
        
        let mut id_initial = 20;
        // let pc = self.rate_map_consumers;
        // self.pack_rate_map(pc, "Producer", "producer_mgmt", &mut id_initial);
        let m = std::mem::take(&mut self.rate_map_producers);
        let p = std::mem::take(&mut self.producers);
        self.producers = self.pack_rate_map(m, "Producer", p);
        let m = std::mem::take(&mut self.rate_map_consumers);
        let c = std::mem::take(&mut self.consumers);
        self.consumers = self.pack_rate_map(m, "Consumer", c);
        
        for name in self.producers {
            id_initial += 1;
            self.constructed_body.push(
                format!("producer_mgmt.add_by_id({}, {}.as_rc())", id_initial, name)
            );
        }
        for name in self.consumers {
            id_initial += 1;
            self.constructed_body.push(
                format!("consumer_mgmt.add_by_id({}, {}.as_rc())", id_initial, name)
            );
        }

        self.constructed_body.push(format!("({}, {})", 
            if self.need_tism { "Some(tism)" } else { "None" },
            if self.need_zenoh { "Some(zenoh)" } else { "None" } 
        ));

        format!("
            #[allow(unused_variables)]
            pub fn codegen_initialize_{}<'a>(
                producer_mgmt: &'a mut crate::data_handlers::ProducerManager, 
                consumer_mgmt: &'a mut crate::data_handlers::ConsumerManager,
                tism_initializer: impl FnOnce() -> crate::pubsub::tism::TISMConnection, 
                zenoh_initializer: impl FnOnce() -> crate::pubsub::zenoh::ZenohConnection<'a>,
            ) -> (Option<crate::pubsub::tism::TISMConnection>, Option<crate::pubsub::zenoh::ZenohConnection<'a>>)
            {{ 
                use crate::common::AsRc;
                use crate::pubsub::Connection;
                {} {} 
                {}
            }}", self.direction.as_ref(),
            if self.need_tism { "let mut tism = tism_initializer();" } else {""},
            if self.need_zenoh { "let mut zenoh = zenoh_initializer();" } else {""},
            self.constructed_body.join(";\n"),
        )
    }

    fn pack_rate_map(&mut self, map: HashMap<time::Duration, Vec<(u64, String)>>, mgr_type: &str, mut mgr: Vec<String>) -> Vec<String> {
        for (d, v) in map {
            let mut packs: Vec<(u64, Vec<String>)> = Vec::new();

            'outer: for (size, name) in v {
                for (len, pack) in &mut packs {
                    if *len + size < PACKER_MAX_SIZE {
                        *len += size;
                        pack.push(name);
                        continue 'outer;
                    } 
                }

                // no pack big enough for pub to fit into, make new pack
                packs.push((size, vec![name]));
            }

            for (_, pack) in packs {
                let name = self.make_var("", format!(
                    "crate::data_handlers::constant_poll_rate::{}::new(std::time::Duration::new({}, {}), [{}])",
                    mgr_type, d.as_secs(), d.subsec_nanos(), pack.iter().map(|s| format!("{}.as_rc()", s)).collect::<Vec<_>>().join(",")
                ));

                mgr.push(name);
            }
        }

        mgr
    }

    fn make_var(&mut self, dtype: impl AsRef<str>, var_contents: impl AsRef<str>) -> String {
        self.var_name_counter += 1;
        let name = format!("codegen_var_{}", self.var_name_counter);
        self.constructed_body.push(
            format!("let {}: {} = {}", name, if !dtype.as_ref().is_empty() { dtype.as_ref() } else { "_" },  var_contents.as_ref())
        );
        name
    }

    fn make_producer_raw_pubsub(&mut self, size: u64, sub_name: impl AsRef<str>) -> String {
        self.make_var(
            format!("crate::data_handlers::raw_pubsub::Producer<{}, _>", size), 
            format!("crate::data_handlers::raw_pubsub::Producer::new({})", sub_name.as_ref())
        )
    }

    fn make_consumer_raw_pubsub(&mut self, size: u64, pub_name: impl AsRef<str>) -> String {
        self.make_var(
            format!("crate::data_handlers::raw_pubsub::Consumer<{}, _>", size), 
            format!("crate::data_handlers::raw_pubsub::Consumer::new({})", pub_name.as_ref())
        )
    }

    fn map_pubsub(&mut self, network: &NetworkType) -> &str {
        match network {
            NetworkType::TISM => {
                self.need_tism = true;
                "tism"
            },
            NetworkType::Zenoh => { 
                self.need_zenoh = true;
                "zenoh"
            },
        }
    }

    fn map_construct_subscriber(&mut self, rate: &PollRate, network: &NetworkType, path: &str) -> String {
        let body =match rate {
            PollRate::ASAP | PollRate::FixedRate(_) => format!("{}.subscribe(\"{}\".to_string())", self.map_pubsub(network), path),
            PollRate::OnChange => format!("{}.subscribe_on_change(\"{}\".to_string())", self.map_pubsub(network), path),
        };

        self.make_var("", body)
    }

    fn construct_publisher(&mut self, size: u64, network: &NetworkType, path: &str) -> String {
        let body = format!("{}.publish::<{}>(\"{}\".to_string())", self.map_pubsub(network), size, path);

        self.make_var("", body)
    }
}
