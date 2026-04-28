use std::time;
use crate::{common::BufferType, errors::{self, AnyError}, lr1121wrapper::{Context, begin, end, init, receive, transmit}, network::{NetworkRadio, SendError}, packet::{OutgoingPacketConfig, ReceivedPacket}};

pub struct lr1121Config {
    pub spi_channel:    u8,
    pub spi_speed:      u32,
    pub spi_device:     u8,
    pub gpio_device:    u8,
    pub cs:             u32,
    pub irq:            u32,
    pub rst:            u32,
    pub busy:           u32,
    pub dio8:           u32,

    pub freq:            f32,
    pub bw:              f32,
    pub sf:              u8,
    pub cr:              u8,
    pub sync_word:       u8,
    pub power:           i8,
    pub preamble_length: u16,
    pub tcxo_voltage:    f32,

    pub timeout:         u16
}

struct lr1121 {
    ctx: *mut  Context,
    stopRec:   bool,
    stopTrans: bool,
}

impl lr1121 {
    pub fn new() -> Self {
        let new_ctx = unsafe {
            init(
                0,    //spiChannel
                0,      //spiSpeed
                0,     //spiDevice
                0,    //gpioDevice
                0,            //CS
                0,           //IRQ
                0,           //RST
                0,          //BUSY
                0           //DIO8
            )
        };
        Self {
            ctx: new_ctx,
            stopRec: false,
            stopTrans: false
        }
    }
}

pub enum error {
    error(i32),
}

impl NetworkRadio for lr1121 {
    type ConfigureError = error;
    /// configure the raido
    fn configure(&mut self) -> Result<(), Self::ConfigureError> {
        let state = unsafe {
            begin(
                self.ctx,   //ctx
                000.0,         //freq MHz
                000.0,           //bw
                0,               //sf
                0,               //cr
                0,         //sync word
                22,           //power
                0,   //preamble length
                3.3     //tcxo voltage
            )
        };
        if state != 0 {
            return Err(error::error(state));
        }
        println!("INFO LR1121: radio configuration finished");
        Ok(())
    }
        
    type ReceiveError = error;
    /// receive packets from radio
    fn try_receive(&mut self) -> Result<Vec<ReceivedPacket>, Self::ReceiveError> {

        
        
        let packets: Vec<ReceivedPacket>;
        let mut buffer = [0u8; 256]; 
        let data_ptr = buffer.as_mut_ptr();
        loop {
            if self.stopRec {
                return Ok(Vec::new()); 
            }

            let result = unsafe {
                receive(self.ctx, data_ptr, buffer.len(), UPLINK_TRANSMIT_BEGIN_PERIOD)
            };

            if result == 0 {
                let packets = vec![ReceivedPacket(&buffer)]; 
                println!("INFO LR1121: Got new packet: {:#?}", packets);
                return Ok(packets);
        }
    }
        
    type CustomSendError = AnyError;
    /// send packets from radio
    fn try_send(&mut self, packet_config: OutgoingPacketConfig, payload: &BufferType) -> Result<time::Duration, SendError<Self::CustomSendError>> {
        let delay = 0; 
        let address = 0;
        //do I change freq and everything else per packet
        let result = unsafe {
            transmit(self.ctx, delay, payload.as_ptr(), payload.len(), address)
        }; //payload maybe shouldn't be a reference
        if result != 0 {
            return Err(SendError::RadioBusy);
        }
        Ok(time::Duration::ZERO)

    }

    /// start the radio
    fn start(&mut self) -> Result<(), AnyError> {




        println!("INFO LR1121: Gateway susscessfully started operation.");
        Ok(())
    }
    /// stop the radio
    fn stop(&mut self) -> Result<(), AnyError> {
        unsafe {
            end(self.ctx);
        };
        println!("INFO LR1121: Gateway susscessfully stopped operation.");
        Ok(())
    }
    /// check if the radio is currently receiving
    fn is_currently_receiving(&mut self) -> Result<bool, AnyError> {
        ///?????
    }
}

//end and destory radio and context??