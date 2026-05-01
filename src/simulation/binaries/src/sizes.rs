
#[repr(u8)]
pub enum FiniteSizedU8Array {
    Len1([u8; 1]) = 1, 
    Len2([u8; 2]) = 2, 
    Len3([u8; 3]) = 3, 
    Len4([u8; 4]) = 4, 
}

impl FiniteSizedU8Array {
    pub const fn get_size(size: u8) -> FiniteSizedU8Array {
        match size {
            1 => FiniteSizedU8Array::Len1([0]),
            _ => panic!()
        }
    }
}