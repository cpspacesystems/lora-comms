

pub struct DataSource {
    size: usize,
    raw_byte_too_big: bool,
    ncopies: usize,
    unit_max: usize,
    count: usize,
}

impl DataSource {
    pub fn new(size: usize) -> Self {
        let size_of_usize = std::mem::size_of::<usize>();
        Self { size, 
            count: 0,
            raw_byte_too_big: size_of_usize > size, 
            unit_max: if size_of_usize > size {
                2_usize.pow((size * 8) as u32) - 1
            } else { usize::MAX },
            ncopies: if size_of_usize > size { 1 } else { size / size_of_usize }
        }
    }

    pub fn generate(&mut self) -> Vec<u8> {
        self.count = self.count.wrapping_add(1);
        if self.raw_byte_too_big {
            if self.count > self.unit_max { self.count = 0 };
        }

        let bytes = self.count.to_le_bytes();
        let mut data: Vec<u8> = bytes.repeat(self.ncopies);
        
        data.resize(self.size, 0);
        data 
    }
}