

pub struct DataSource {
    size: usize,
    ncopies: usize,
    count: usize,
}

impl DataSource {
    pub fn new(size: usize) -> Self {
        let size_of_usize = std::mem::size_of::<usize>();
        Self { size, 
            count: 0,
            ncopies: if size_of_usize > size { 1 } else { size / size_of_usize }
        }
    }

    pub fn generate(&mut self) -> Vec<u8> {
        self.count = self.count.wrapping_add(1);

        let bytes = self.count.to_le_bytes();
        let mut data: Vec<u8> = bytes.repeat(self.ncopies);
        
        data.resize(self.size, 0);
        data 
    }
}