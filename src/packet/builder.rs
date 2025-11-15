use std::{error::Error, io::BufReader};

use crate::packet::{common::DATA_BOUNDARY, data_section::{self, create_data_section}, error::ErrorType, types::{self, BufferType}};


pub struct PacketBuilder {
    data_sections: Vec<BufferType>,
}

impl PacketBuilder {
    pub fn add(&mut self, data_type: types::DataSectionType, data: Vec<u8>) -> Result<(), ErrorType> {
        self.data_sections.push(create_data_section(data_type, data)?);
        Ok(())
    }

    pub fn add_reserved(&mut self, data: BufferType) {
        self.data_sections.push(data);
    }
    pub fn add_raw(&mut self, data: BufferType) {
        self.data_sections.push(data);
    }

    pub fn build(&mut self) -> BufferType {
        std::mem::take(&mut self.data_sections).into_iter().flatten().collect()
    }
}