
mod publisher;
mod subscriber;

#[repr(C)]
pub enum lr11xx_hal_status_t {
    LR11XX_HAL_STATUS_OK = 0,
    LR11XX_HAL_STATUS_ERROR = 3,
}
#[repr(C)]
pub enum lr11xx_status_t {
    LR11XX_HAL_STATUS_OK = 0,
    LR11XX_HAL_STATUS_ERROR = 3,
}

unsafe extern "C" {
    fn lr11xx_hal_write(context: *const std::ffi::c_void, command : *const i8, command_length : u16, data: *const u8, 
                                data_length : u16) -> lr11xx_hal_status_t;


    fn lr11xx_radio_set_pkt_type(context: *const std::ffi::c_void, pkt_type: u8) -> lr11xx_status_t;
}
fn main() {
    
}




// 8 bit enum instead of string key

//config function
//send function

