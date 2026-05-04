use std::{ptr::null, time};
use crate::{common::{Bandwidth, BufferType, SpreadFactor, LoraCodeRate}, common_config::{UPLINK_TRANSMIT_TIMEOUT_PERIOD, LORA_125KHZ_CH0, LORA_PREAMBLE_LENGTH, INITIAL_CODE_RATE}, errors::{self, AnyError}, lr1121wrapper::{Context, begin, end, getSNR, init, receive, setCodingRate, setFrequency, setSpreadingFactor, transmit}, network::{NetworkRadio, SendError}, packet::{self, OutgoingPacketConfig, PacketMetadata, ReceivedPacket}};

pub struct lr1121Config {
    pub spi_channel:     u8,
    pub spi_speed:       u32,
    pub spi_device:      u8,
    pub gpio_device:     u8,
    pub cs:              u32,
    pub irq:             u32,
    pub rst:             u32,
    pub busy:            u32,
    pub dio8:            u32,

    pub freq_mhz:        f32,
    pub bw_khz:          Bandwidth,
    pub sf:              SpreadFactor,
    pub cr:              LoraCodeRate,
    pub sync_word:       u8,
    pub power:           i8,
    pub preamble_length: u16,
    pub tcxo_voltage:    f32,
}

struct lr1121 {
    ctx: *mut  Context,
    config: lr1121Config,
    currentlyRec: bool,
}

impl lr1121 {
    pub fn new() -> Self {
        let config = lr1121Config {
            spi_channel: 0, spi_speed: 16_000_000, spi_device: 0, 
            gpio_device: 4, cs: 8, irq: 0, 
            rst: 0, busy: 0, dio8: 0, 
            freq_mhz: (LORA_125KHZ_CH0 / 1000000) as f32, bw_khz: BW_125KHZ, sf: SpreadFactor::SF7, 
            cr: LoraCodeRate::CR1, sync_word: 0, power: 22,
            preamble_length: LORA_PREAMBLE_LENGTH, tcxo_voltage: 3.3
        }; 
        



// check with Alvin for defaults



        let new_ctx = unsafe {
            init(
                config.spi_channel,    //spiChannel
                config.spi_speed,      //spiSpeed
                config.spi_device,     //spiDevice
                config.gpio_device,    //gpioDevice
                config.cs,             //CS
                config.irq,            //IRQ
                config.rst,            //RST
                config.busy,           //BUSY
                config.dio8            //DIO8
            )
        };
        
        
        Self {
            ctx: new_ctx,
            config, 
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
                self.ctx,                                   //ctx
                self.config.freq_mhz / 1000000,             //freq MHz
                lr1121_from_bandwidth(self.config.bw_khz),  //bw
                self.config.sf,                             //sf
                lr1121_from_coderate(self.config.cr),       //cr
                self.config.sync_word,                      //sync word
                self.config.power,                          //power
                self.config.preamble_length,                //preamble length
                self.config.tcxo_voltage                    //tcxo voltage
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

            let metadata = PacketMetadata {
                length: buffer.len(),
                snr: unsafe {getSNR(self.ctx)},
                frequency: self.config.freq as u32,
                sf: self.config.sf,
                coderate: self.config.cr
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
                setFrequency(self.ctx, packet_config.freq_hz);
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
        println!("INFO LR1121: Gateway successfully started operation.");
        Ok(())
    }
    /// stop the radio
    fn stop(&mut self) -> Result<(), AnyError> {
        unsafe {
            end(self.ctx);
        };
        println!("INFO LR1121: Gateway sucessfully stopped operation.");
        Ok(())
    }
    /// check if the radio is currently receiving
    fn is_currently_receiving(&mut self) -> Result<bool, AnyError> {
        Ok(self.currentlyRec)
    }


        
}

pub fn lr1121_from_bandwidth(value: Bandwidth) -> u8 {
    match value {
        Bandwidth::Low125khz => bindings_loragw_hal::BW_125KHZ,
        Bandwidth::Mid250khz => bindings_loragw_hal::BW_250KHZ,
        Bandwidth::High500khz => bindings_loragw_hal::BW_500KHZ,
    }
}

pub fn lr1121_from_coderate(value: LoraCodeRate) -> u8 {
    match value {
        LoraCodeRate::CR1 => bindings_loragw_hal::CR_LORA_4_5,
        LoraCodeRate::CR2 => bindings_loragw_hal::CR_LORA_4_6,
        LoraCodeRate::CR3 => bindings_loragw_hal::CR_LORA_4_7,
        LoraCodeRate::CR4 => bindings_loragw_hal::CR_LORA_4_8,
    }
}

pub fn lr1121_to_coderate(value: u8) -> Result<LoraCodeRate, errors::InvalidData> {
    match value {
        bindings_loragw_hal::CR_LORA_4_5 => Ok(LoraCodeRate::CR1),
        bindings_loragw_hal::CR_LORA_4_6 => Ok(LoraCodeRate::CR2),
        bindings_loragw_hal::CR_LORA_4_7 => Ok(LoraCodeRate::CR3),
        bindings_loragw_hal::CR_LORA_4_8 => Ok(LoraCodeRate::CR4),
        _ => Err(errors::InvalidData(format!("Invalid data: {value} is not a valid SX1302 lora code rate")))
    }
}
