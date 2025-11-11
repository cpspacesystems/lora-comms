use crate::packet::types::GPSTime;
use std::{cell::RefCell, collections::{HashMap, HashSet}, io::Write, rc::{Rc, Weak}, usize};

use crate::packet::{self, data_section, types::BufferType};

const CRTITICAL_PURGE_TIME: GPSTime = 1000;
const HIGH_PURGE_TIME: GPSTime = 10000;
const NORMAL_PURGE_TIME: GPSTime = 1000000;
const LOW_PURGE_TIME: GPSTime = 1000000000;

#[derive(Clone, Copy)]
enum Priority {
    Critical, 
    High, 
    Normal, 
    Low
}

#[derive(Clone)]
pub struct DataSection {
    priority: Priority,
    data: BufferType
}

#[derive(Clone)]
pub struct DataFrame {
    time: GPSTime,
    critical: Vec<Rc<DataSection>>,
    high: Vec<Rc<DataSection>>,
    normal: Vec<Rc<DataSection>>, 
    low: Vec<Rc<DataSection>>
}

impl DataFrame {
    pub fn new(time: GPSTime) -> DataFrame {
        DataFrame { time, critical: Vec::new(), high: Vec::new(), normal: Vec::new(), low: Vec::new() }
    } 

    pub fn add_data_section(&mut self, ds: DataSection) {
        match ds.priority {
            Priority::Critical => self.critical.push(Rc::new(ds)),
            Priority::High => self.high.push(Rc::new(ds)),
            Priority::Normal => self.normal.push(Rc::new(ds)),
            Priority::Low => self.low.push(Rc::new(ds)),
        }
    }
}

pub struct Composer {

    // data frames containing priority sections
    critical_df: Vec<Rc<RefCell<DataFrame>>>, 
    high_df: Vec<Rc<RefCell<DataFrame>>>,
    normal_df: Vec<Rc<RefCell<DataFrame>>>,
    low_df: Vec<Rc<RefCell<DataFrame>>>
}

impl Composer {
    pub fn new() -> Composer {
        Composer { critical_df: Vec::new(), high_df: Vec::new(), normal_df: Vec::new(), low_df: Vec::new() }
    }

    // adds data frame into composer
    pub fn add_data_frame(&mut self, raw_df: DataFrame) {
        let rc = Rc::new(RefCell::new(raw_df)); 
        let df = rc.borrow();

        if df.critical.len() > 0 {
            self.critical_df.push(Rc::clone(&rc));
        }
        if df.high.len() > 0 {
            self.high_df.push(Rc::clone(&rc));
        }
        if df.normal.len() > 0 {
            self.high_df.push(Rc::clone(&rc));
        }
        if df.low.len() > 0 {
            self.low_df.push(Rc::clone(&rc));
        }
    }

    pub fn compose_packet(&mut self, packet_size: usize) {
        let mut current_size: usize = 0;

        let mut transmit_data= HashMap::<GPSTime, Vec<Rc<DataSection>>>::new();

        Self::_condense_data(&mut self.critical_df, CRTITICAL_PURGE_TIME, &mut transmit_data, &mut current_size, packet_size);
        Self::_condense_data(&mut self.high_df, HIGH_PURGE_TIME, &mut transmit_data, &mut current_size, packet_size);
        Self::_condense_data(&mut self.normal_df, NORMAL_PURGE_TIME, &mut transmit_data, &mut current_size, packet_size);
        Self::_condense_data(&mut self.low_df, LOW_PURGE_TIME, &mut transmit_data, &mut current_size, packet_size);
    
        // pack packet into bytes
    }
    #[inline(always)]
    fn _condense_data(
        prioity_vec: &mut Vec<Rc<RefCell<DataFrame>>>, 
        purge_time: GPSTime,
        transmit_data: &mut HashMap<GPSTime, Vec<Rc<DataSection>>>, 
        current_size: &mut usize,
        packet_size: usize
    ) {
        prioity_vec.retain(|x| {
            let mut df = x.borrow_mut();
            // purge old data frames
            if df.time > purge_time {
                return false;
            }

            let time = df.time;
            df.critical.retain(|y| {
                let dsize = y.data.len(); 
                // packet will be too big, data section retained for future transmission
                if dsize + *current_size > packet_size {
                    return true;
                }
                
                // add data to buffer
                match transmit_data.entry(time) {
                    std::collections::hash_map::Entry::Occupied(mut e) => e.get_mut().push(Rc::clone(y)),
                    std::collections::hash_map::Entry::Vacant(e) => { e.insert(vec![Rc::clone(y)]); },
                };
                false // release reference to data section
            });

            true 
        });
    }

}