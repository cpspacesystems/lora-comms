use std::{ptr::null, time};

mod lr1121wrapper;
use lr1121wrapper::{Context, begin, end, getSNR, init, receive, setCodingRate, setFrequency, setSpreadingFactor, transmit};
use crate::{common::{BufferType, SpreadFactor, LoraCodeRate}, common_config::{UPLINK_TRANSMIT_TIMEOUT_PERIOD, LORA_125KHZ_CH0, LORA_PREAMBLE_LENGTH, INITIAL_CODE_RATE}, errors::{self, AnyError}, network::{NetworkRadio, SendError}, packet::{self, OutgoingPacketConfig, PacketMetadata, ReceivedPacket}};

pub struct LR1121Config {
    spi_channel:    u8,
    spi_speed:      u32,
    spi_device:     u8,
    gpio_device:    u8,
    cs:             u32,
    irq:            u32,
    rst:            u32,
    busy:           u32,
    dio8:           u32,

    freq:            f32,
    bw:              f32,
    sf:              u8,
    cr:              u8,
    sync_word:       u8,
    power:           i8,
    preamble_length: u16,
    tcxo_voltage:    f32,

    timeout:         u16
}

pub struct LR1121 {
    ctx: *mut  Context,
    config: LR1121Config,
    currently_rec: bool,
}

impl LR1121 {
    pub fn new() -> Self {
        let config = LR1121Config {
            spi_channel: 0, spi_speed: 16_000_000, spi_device: 7, 
            gpio_device: 4, cs: 8, irq: 0, 
            rst: 0, busy: 0, dio8: 0, 
            freq: (LORA_125KHZ_CH0) as f32, bw: 125.0, sf: 0, 
            cr: 0x1, sync_word: 0, power: 22,
            preamble_length: LORA_PREAMBLE_LENGTH, tcxo_voltage: 3.3, timeout: 0
        }; 
        
        let new_ctx = unsafe {
            init(
                config.spi_channel,    //spiChannel
                config.spi_speed,      //spiSpeed
                config.spi_device,     //spiDevice
                config.gpio_device,    //gpioDevice
                config.cs,            //CS
                config.irq,           //IRQ
                config.rst,           //RST
                config.busy,          //BUSY
                config.dio8          //DIO8
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
                setSpreadingFactor(self.ctx, spread_factor.into(), false);
                setCodingRate(self.ctx, coderate as u8, false);
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
        Ok(self.currently_rec)
    }
}