use std::time;
use crate::{common::{BufferType, SpreadFactor, LoraCodeRate}, common_config::UPLINK_TRANSMIT_TIMEOUT_PERIOD, errors::{self, AnyError}, lr1121wrapper::{Context, begin, end, getSNR, init, receive, setCodingRate, setFrequency, setSpreadingFactor, transmit}, network::{NetworkRadio, SendError}, packet::{self, OutgoingPacketConfig, PacketMetadata, ReceivedPacket}};

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
    config: lr1121Config,
    currentlyRec: bool,
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
            config: lr1121Config {
                spi_channel: 0, spi_speed: 0, spi_device: 0, 
                gpio_device: 0, cs: 0, irq: 0, 
                rst: 0, busy: 0, dio8: 0, 
                freq: 0.0, bw: 0.0, sf: 0, 
                cr: 0, sync_word: 0, power: 22, 
                preamble_length: 0, tcxo_voltage: 3.3, timeout: 0},
            currentlyRec: false
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
                self.ctx,                              //ctx
                self.config.freq,                              //freq MHz
                self.config.bw,                                //bw
                self.config.sf,                                //sf
                self.config.cr,                                //cr
                self.config.sync_word,               //sync word
                self.config.power,                             //power
                self.config.preamble_length,   //preamble length
                self.config.tcxo_voltage          //tcxo voltage
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
        self.currentlyRec = true;

        let mut buffer = [0u8; 256]; 
        let data_ptr = buffer.as_mut_ptr();
        let result = unsafe {
            receive(self.ctx, data_ptr, buffer.len(), UPLINK_TRANSMIT_TIMEOUT_PERIOD.as_millis() as u32)
        };


        if result == 0 {
            let sf = match self.config.sf {
                5 => SpreadFactor::SF5,
                6 => SpreadFactor::SF6,
                7 => SpreadFactor::SF7,
                8 => SpreadFactor::SF8,
                9 => SpreadFactor::SF9,
                _ => panic!("invalid spread factor {}", self.config.sf),
            };
            let cr = match self.config.cr {
                1 => LoraCodeRate::CR1,
                2 => LoraCodeRate::CR2,
                3 => LoraCodeRate::CR3,
                4 => LoraCodeRate::CR4,
                _ => panic!("invalid coderate {}", self.config.cr),
            };

            let metadata = PacketMetadata {
                length: buffer.len(),
                snr: unsafe {getSNR(self.ctx)},
                frequency: self.config.freq as u32,
                sf: sf,
                coderate: cr
            };


           let packets = vec![ReceivedPacket {
                data: buffer.to_vec(),
                meta: metadata,
           }]; 
            println!("INFO LR1121: Got new packet: {:#?}", packets);
            self.currentlyRec = false;
            return Ok(packets);
        } else {
            self.currentlyRec = false;
            return Ok(Vec::new());
        }
        
    }
        
    type CustomSendError = AnyError;
    /// send packets from radio
    fn try_send(&mut self, packet_config: OutgoingPacketConfig, payload: &BufferType) -> Result<time::Duration, SendError<Self::CustomSendError>> {
        let delay = 0; 
        let address = 0;
        match packet_config.modulation {
            crate::packet::OutgoingPacketModulation::LoRa { spread_factor, coderate, .. } => unsafe {
                setSpreadingFactor(self.ctx, spread_factor.into(), false);
                setCodingRate(self.ctx, coderate as u8, false);



// check with Alvin on longInterleave

                setFrequency(self.ctx, packet_config.freq_hz as f32);
            },
            _ => return Err("Unsupported modulation".into())
        }
        
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
        Ok(self.currentlyRec)
    }
}

//end and destory radio and context??







////// add snr to radiolib
//
// add snr
