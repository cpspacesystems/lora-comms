#[repr(C)]
pub struct Context {
    _private: [u8; 0],
}

#[cfg(target_family = "unix")]
#[link(name = "lora_bridge", kind = "static")]
#[link(name = "lgpio", kind = "static")]
#[link(name = "RadioLib", kind = "static")]
#[link(name = "stdc++", kind = "static")]
unsafe extern "C" {
    pub fn init(
        spiChannel: u8,
        spiSpeed: u32,
        spiDevice: u8,
        gpioDevice: u8,
        cs: u32,
        irq: u32,
        rst: u32,
        busy: u32,
        dio8: u32,
    ) -> *mut Context;

    pub fn begin(
        context: *mut Context,
        freq: f32,
        bw: f32,
        sf: u8,
        cr: u8,
        syncWord: u8,
        power: i8,
        preambleLength: u16,
        tcxoVoltage: f32,
    ) -> i32;

    pub fn transmit(
        context: *mut Context,
        delay: u16,
        package: *const u8,
        len: usize,
        addr: u8,
    ) -> i32;

    pub fn receive(context: *mut Context, data: *mut u8, len: usize, timeout: u32) -> i32;

    pub fn setFrequency(context: *mut Context, frequency: f32) -> i32;
    pub fn setPower(context: *mut Context, power: u8) -> i32;
    pub fn setSpreadingFactor(context: *mut Context, sf: u8, legacy: bool) -> i32;
    pub fn setCodingRate(context: *mut Context, cr: u8, longInterleave: bool) -> i32;
    pub fn setBitRate(context: *mut Context, br: f32) -> i32;
    pub fn setPreambleLength(context: *mut Context, preambleLength: usize) -> i32;
    pub fn end(context: *mut Context);

    pub fn flash_firmware(ctx: *mut Context) -> i32;
    pub fn reset(context: *mut Context) -> i16;

    pub fn getSNR(context: *mut Context) -> f32;
}
