
/// Configuration structure for common SX1302 parameters
/// Indepth configuration, such as configuration gains and offsets 
/// can be done by changing values in the configure function of sc1302.rs
pub struct SX1302Configuration<'a> {
    /// Path to the corrosponding SPI device for this SX1302
    pub device_spi_path: &'a str,
    /// Base Frequency of lora
    /// 
    /// 8x 125khz bandwidth receive channels spread 400khz apart are created, plus
    /// 
    /// 1x 500khz bandwdith receive channel with SF7 centered at 1200khz after the last 125khz channel 
    pub comm_base_frequency_hz: u32,
    /// Weather or Full Duplex communications are enabled
    pub comm_full_duplex: bool,
    /// Should packets have fine timestamps on them
    pub packet_fine_timestamps: bool,
    
}

// default configuration
pub const DEFAULT_SX1302_CONFIG: SX1302Configuration = SX1302Configuration {
    device_spi_path: "/dev/spidev0.0",
    comm_base_frequency_hz: 907300000, // 907.3 base frequency to avoid the more often used lower channels
    comm_full_duplex: false, 
    packet_fine_timestamps: true
};

