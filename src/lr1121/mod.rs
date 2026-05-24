use std::{ptr::null, time};

mod lr1121wrapper;
use crate::{
    common::{Bandwidth, BufferType, LoraCodeRate, SpreadFactor},
    common_config::{
        INITIAL_CODE_RATE, LORA_125KHZ_CH0, LORA_PREAMBLE_LENGTH, UPLINK_TRANSMIT_TIMEOUT_PERIOD,
    },
    errors::{self, AnyError},
    network::{NetworkRadio, SendError},
    packet::{self, OutgoingPacketConfig, PacketMetadata, ReceivedPacket},
};
use base64::Engine;
use log::{debug, info, trace};
use lr1121wrapper::{
    Context, begin, end, getSNR, init, receive, setCodingRate, setFrequency, setSpreadingFactor,
    transmit,
};

const RADIOLIB_ERR_TX_TIMEOUT: i32 = -5;
const RADIOLIB_ERR_NONE: i32 = 0;

/// Specific PIN for lr1121 is not connected or shouldn't be used
pub const LR1121_NO_CONNECT: u32 = 0xFFFFFFFF;

/// LR1121 Frequency, in Mhz
#[derive(Debug, Clone, Copy)]
pub struct LR112FreqMhz(pub f32);
impl LR112FreqMhz {
    /// conversion from hz to Mhz
    pub const fn from_hz(value: u32) -> Self {
        Self(value as f32 / 1_000_000.0)
    }
    pub const fn as_hz(&self) -> u32 {
        (self.0 * 1_000_000.0) as u32
    }
}
impl From<LR112FreqMhz> for f32 {
    fn from(value: LR112FreqMhz) -> Self {
        value.0
    }
}

/// converts from bandwidth to radiolib acceptable bandwidth
const fn cvt_bandwidth(bw: Bandwidth) -> f32 {
    match bw {
        Bandwidth::Low125khz => 125.0,
        Bandwidth::Mid250khz => 250.0,
        Bandwidth::High500khz => 500.0,
    }
}

/// converts from LoraCodeRate to radiolib acceptable coderate
const fn cvt_coderate(cr: LoraCodeRate) -> u8 {
    (cr as u8) + 4
}

pub struct LR1121Config {
    pub spi_channel: u8,
    pub spi_speed_hz: u32,
    pub spi_device: u8,
    pub gpio_device: u8,
    pub cs: u32,
    pub irq: u32,
    pub rst: u32,
    pub busy: u32,
    pub dio8: u32,

    pub receive_freq: LR112FreqMhz, //MHz
    pub receive_bw: Bandwidth,      //kHz
    pub receive_sf: SpreadFactor,
    pub receive_cr: LoraCodeRate,
    pub sync_word: u8,
    pub power: i8, //dBm
    pub preamble_length: u16,
    pub tcxo_voltage: f32,
}

pub const DEFAULT_LR1121_CONFIG: LR1121Config = LR1121Config {
    spi_channel: 0,
    spi_device: 0,
    spi_speed_hz: 16_000_000,
    gpio_device: 4,
    cs: LR1121_NO_CONNECT,
    irq: LR1121_NO_CONNECT,
    rst: LR1121_NO_CONNECT,
    busy: LR1121_NO_CONNECT,
    dio8: LR1121_NO_CONNECT,
    receive_freq: LR112FreqMhz::from_hz(LORA_125KHZ_CH0),
    receive_bw: Bandwidth::Low125khz,
    receive_sf: SpreadFactor::SF7,
    receive_cr: LoraCodeRate::CR1,
    sync_word: 0x12,
    power: 22,
    preamble_length: LORA_PREAMBLE_LENGTH,
    tcxo_voltage: 3.3,
};

pub struct LR1121 {
    ctx: *mut Context,
    config: LR1121Config,
    currently_rec: bool,
}

impl LR1121 {
    pub fn new(config: LR1121Config) -> Self {
        let new_ctx = unsafe {
            init(
                config.spi_channel,  //spiChannel
                config.spi_speed_hz, //spiSpeed
                config.spi_device,   //spiDevice
                config.gpio_device,  //gpioDevice
                config.cs,           //CS
                config.irq,          //IRQ
                config.rst,          //RST
                config.busy,         //BUSY
                config.dio8,         //DIO8
            )
        };

        Self {
            ctx: new_ctx,
            config,
            currently_rec: false,
        }
    }
}

impl NetworkRadio for LR1121 {
    type ConfigureError = AnyError;
    /// configure the raido
    fn configure(&mut self) -> Result<(), Self::ConfigureError> {
        let state = unsafe {
            begin(
                self.ctx,
                self.config.receive_freq.into(),
                cvt_bandwidth(self.config.receive_bw),
                self.config.receive_sf.into(),
                cvt_coderate(self.config.receive_cr),
                self.config.sync_word,
                self.config.power,
                self.config.preamble_length,
                self.config.tcxo_voltage,
            )
        };
        if state != 0 {
            return Err(format!("Failed to Configure LR1121 with error: {}", state).into());
        }
        info!(target: "LR1121", "Radio configuration finished");
        Ok(())
    }

    type ReceiveError = AnyError;
    /// receive packets from radio
    fn try_receive(&mut self) -> Result<Vec<ReceivedPacket>, Self::ReceiveError> {
        self.currently_rec = true;

        let mut buffer = [0u8; 256];
        let data_ptr = buffer.as_mut_ptr();
        let result = unsafe {
            if let s = setFrequency(self.ctx, self.config.receive_freq.into())
                && s != 0
            {
                return Err(format!("set FREQ failed {}", s).as_str().into());
            };
            if let s = setCodingRate(self.ctx, cvt_coderate(self.config.receive_cr), false)
                && s != 0
            {
                return Err(format!("set CR failed {}", s).as_str().into());
            };
            receive(
                self.ctx,
                data_ptr,
                buffer.len(),
                UPLINK_TRANSMIT_TIMEOUT_PERIOD.as_millis() as u32,
            )
        };

        if result >= 0 {
            let metadata = PacketMetadata {
                length: result as usize,
                snr: unsafe { getSNR(self.ctx) },
                frequency: self.config.receive_freq.as_hz(),
                sf: self.config.receive_sf,
                coderate: self.config.receive_cr,
            };

            let packets = vec![ReceivedPacket {
                data: buffer[..result as usize].to_vec(),
                meta: metadata,
            }];
            debug!(target: "LR1121", "IN: {}", metadata);
            trace!(target: "LR1121", "ICAP: {}", base64::prelude::BASE64_STANDARD.encode(packets[0].data.as_slice()));
            self.currently_rec = false;
            return Ok(packets);
        } else {
            self.currently_rec = false;
            return Ok(Vec::new());
        }
    }

    type CustomSendError = AnyError;
    /// send packets from radio
    fn try_send(
        &mut self,
        packet_config: OutgoingPacketConfig,
        payload: &BufferType,
    ) -> Result<time::Duration, SendError<Self::CustomSendError>> {
        let delay = 0;
        let address = 0;
        match packet_config.modulation {
            crate::packet::OutgoingPacketModulation::LoRa {
                spread_factor,
                coderate,
                ..
            } => unsafe {
                if let s = setSpreadingFactor(self.ctx, spread_factor.into(), false)
                    && s != 0
                {
                    return Err(format!("set SF failed {}", s).as_str().into());
                };
                if let s = setCodingRate(self.ctx, cvt_coderate(coderate), false)
                    && s != 0
                {
                    return Err(format!("set CR failed {}", s).as_str().into());
                };
                if let s = setFrequency(
                    self.ctx,
                    LR112FreqMhz::from_hz(packet_config.freq_hz).into(),
                ) && s != 0
                {
                    return Err(format!("set FREQ failed {}", s).as_str().into());
                };

                debug!(target: "LR1121", "OUT: FREQ {}, {:?}, {:?}, LEN {}", packet_config.freq_hz, spread_factor, coderate, payload.len());
            },
            _ => return Err("Unsupported modulation".into()),
        }

        trace!(target: "LR1121", "OCAP: {}", base64::prelude::BASE64_STANDARD.encode(payload.as_slice()));

        let result = unsafe { transmit(self.ctx, delay, payload.as_ptr(), payload.len(), address) };
        if result != RADIOLIB_ERR_NONE && result != RADIOLIB_ERR_TX_TIMEOUT {
            return Err(format!(
                "INFO LR1121: Encountered error while transmitting: {}",
                result
            )
            .as_str()
            .into());
        }
        Ok(time::Duration::ZERO)
    }

    /// start the radio
    fn start(&mut self) -> Result<(), AnyError> {
        info!(target: "LR1121", "Radio successfully started operation.");
        Ok(())
    }
    /// stop the radio
    fn stop(&mut self) -> Result<(), AnyError> {
        unsafe {
            end(self.ctx);
        };
        info!(target: "LR1121", "Radio successfully stopped operation.");
        Ok(())
    }
    /// check if the radio is currently receiving
    fn is_currently_receiving(&mut self) -> Result<bool, AnyError> {
        Ok(self.currently_rec)
    }
}
