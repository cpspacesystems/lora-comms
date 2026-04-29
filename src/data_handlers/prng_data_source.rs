use std::{cell::{Cell, RefCell}, hash::{BuildHasher, Hasher, RandomState}};

use crate::{common::{self, BufferType}, data_handlers::{DataConsumer, DataProducer}, errors::{self, AnyError}};


pub struct PRNG {
    generator: oorandom::Rand32,
    size: usize, // number of bytes geneated per call to produce
}
impl PRNG {
    /// creates a deterministic RNG that outputs raw data with size  
    pub fn new(size: usize) -> Self {
        Self {
            generator: common::get_prng(),
            size: size,
        }
    }
}

impl DataProducer for PRNG {
    fn produce(&mut self) -> Result<Option<BufferType>, crate::errors::AnyError> {
        let mut res = BufferType::with_capacity(self.size);
        let mut n_bytes = 0;
        while n_bytes + 4 < self.size {
            let rand_data: [u8; 4] = { self.generator.rand_u32().to_le_bytes() };
            res.extend(rand_data);
            n_bytes += 4;
        };
        
        // fill left over capacity
        let diff = self.size - n_bytes;
        if diff > 0 {
            let rand_data: [u8; 4] = { self.generator.rand_u32().to_le_bytes() };
            res.extend(&rand_data[0..diff]); // diff is guranteed to be <= 4
        };
        
        Ok(Some(res))
    }
    
    fn get_size(&self) -> usize {
        self.size
    }
}

impl DataConsumer for PRNG {
    fn consume(&mut self, _buffer: BufferType) -> Result<(), crate::errors::AnyError> {
        Ok(())
        // let mut res = BufferType::with_capacity(self.size);
        // let mut n_bytes = 0;
        // while n_bytes + 4 < self.size {
        //     let rand_data: [u8; 4] = { self.generator.rand_u32().to_le_bytes() };
        //     res.extend(rand_data);
        //     n_bytes += 4;
        // };
        
        // // fill left over capacity
        // let diff = self.size - n_bytes;
        // if diff > 0 {
        //     let rand_data: [u8; 4] = { self.generator.rand_u32().to_le_bytes() };
        //     res.extend(&rand_data[0..diff]); // diff is guranteed to be <= 4
        // };

        // if buffer == res {
        //     Ok(())
        // } else {
        //     println!("expected {}, got: {}", hex::encode_upper(buffer), hex::encode_upper(res));
        //     Err(errors::PRNGConsumerUnexpected.into())
        // }
    }

    fn get_size(&self) -> usize {
        self.size
    }
}