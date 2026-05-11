use std::{ptr::null, time};

mod lr1121wrapper;
use lr1121wrapper::{Context, begin, end, getSNR, init, receive, setCodingRate, setFrequency, setSpreadingFactor, transmit};
use crate::{common::{BufferType, SpreadFactor, LoraCodeRate}, common_config::{UPLINK_TRANSMIT_TIMEOUT_PERIOD, LORA_125KHZ_CH0, LORA_PREAMBLE_LENGTH, INITIAL_CODE_RATE}, errors::{self, AnyError}, network::{NetworkRadio, SendError}, packet::{self, OutgoingPacketConfig, PacketMetadata, ReceivedPacket}};

const RADIOLIB_ERR_TX_TIMEOUT: i32 = -5;
const RADIOLIB_ERR_NONE: i32 = 0;



pub struct LR1121Config {
    pub spi_channel:    u8,
    pub spi_speed:      u32,
    pub spi_device:     u8,
    pub gpio_device:    u8,
    pub cs:             u32,
    pub irq:            u32,
    pub rst:            u32,
    pub busy:           u32,
    pub dio8:           u32,

    pub freq:            f32, //MHz
    pub bw:              f32, //kHz
    pub sf:              u8,
    pub cr:              u8,
    pub sync_word:       u8,
    pub power:           i8,  //dBm
    pub preamble_length: u16,
    pub tcxo_voltage:    f32,
}

pub const DEFAULT_LR1121_CONFIG: LR1121Config = LR1121Config {
    spi_channel: 0, spi_speed: 16_000_000, spi_device: 0, 
    gpio_device: 4, cs: 18, irq: 0, 
    rst: 5, busy: 6, dio8: 0, 
    freq: (LORA_125KHZ_CH0) as f32, bw: 125.0, sf: 0, 
    cr: 0x1, sync_word: 0, power: 22,
    preamble_length: LORA_PREAMBLE_LENGTH, tcxo_voltage: 3.3
};

pub struct LR1121 {
    ctx: *mut  Context,
    config: LR1121Config,
    currently_rec: bool,
}

impl LR1121 {
    pub fn new(config: LR1121Config) -> Self {
        let new_ctx = unsafe {
            init(
                config.spi_channel,    //spiChannel
                config.spi_speed,        //spiSpeed
                config.spi_device,      //spiDevice
                config.gpio_device,    //gpioDevice
                config.cs,                        //CS
                config.irq,                       //IRQ
                config.rst,                       //RST
                config.busy,                      //BUSY
                config.dio8                       //DIO8
            )
        };
        
        
        Self {
            ctx: new_ctx,
            config, 
            currently_rec: false
        }
    }
}

impl NetworkRadio for LR1121 {
    type ConfigureError = AnyError;
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
            return Err(format!("Failed to Configure LR1121 with error: {}", state).into());
        }
        println!("INFO LR1121: radio configuration finished");
        Ok(())
    }
        
    type ReceiveError = AnyError;
    /// receive packets from radio
    fn try_receive(&mut self) -> Result<Vec<ReceivedPacket>, Self::ReceiveError> {
        self.currently_rec = true;

        let mut buffer = [0u8; 256]; 
        let data_ptr = buffer.as_mut_ptr();
        let result = unsafe {
            if let s = setFrequency(self.ctx, self.config.freq) && s != 0 { return Err(format!("set FREQ failed {}", s).as_str().into()); };
            receive(self.ctx, data_ptr, buffer.len(), UPLINK_TRANSMIT_TIMEOUT_PERIOD.as_millis() as u32)
        };


        if result >= 0 {
            let sf = match self.config.sf {
                5 => SpreadFactor::SF5,
                6 => SpreadFactor::SF6,
                7 => SpreadFactor::SF7,
                8 => SpreadFactor::SF8,
                9 => SpreadFactor::SF9,
                _ => panic!("invalid spread factor {}", self.config.sf),
            };
            let cr = match self.config.cr - 4 {
                1 => LoraCodeRate::CR1,
                2 => LoraCodeRate::CR2,
                3 => LoraCodeRate::CR3,
                4 => LoraCodeRate::CR4,
                _ => panic!("invalid coderate {}", self.config.cr),
            };

            let metadata = PacketMetadata {
                length: result as usize,
                snr: unsafe {getSNR(self.ctx)},
                frequency: self.config.freq as u32,
                sf: sf,
                coderate: cr
            };


           let packets = vec![ReceivedPacket {
                data: buffer[..result as usize].to_vec(),
                meta: metadata,
           }]; 
            // println!("INFO LR1121: Got new packet: {:#?}", packets);
            self.currently_rec = false;
            return Ok(packets);
        } else {
            self.currently_rec = false;
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
                if let s = setSpreadingFactor(self.ctx, spread_factor.into(), false) && s != 0 { return Err(format!("set SF failed {}", s).as_str().into()); };
                if let s = setCodingRate(self.ctx, 4 + coderate as u8, false) && s != 0 { return Err(format!("set CR failed {}", s).as_str().into()); };
                if let s = setFrequency(self.ctx, packet_config.freq_hz as f32 / 1_000_000.0) && s != 0 { return Err(format!("set FREQ failed {}", s).as_str().into()); };
            },
            _ => return Err("Unsupported modulation".into())
        }
        
        let result = unsafe {
            transmit(self.ctx, delay, payload.as_ptr(), payload.len(), address)
        };
        if result != RADIOLIB_ERR_NONE && result != RADIOLIB_ERR_TX_TIMEOUT {
            return Err(format!("INFO LR1121: Encountered error while transmitting: {}", result).as_str().into());
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
        Ok(self.currently_rec)
    }
}