mod publisher;
mod subscriber;
mod lr1121wrapper;

use lr1121wrapper::*;

const SPI_CHANNEL: u8 = 0;
const SPI_SPEED: u32 = 500_000;
const SPI_DEVICE: u8 = 1;
const GPIO_DEVICE: u8 = 4;
const PIN_CS: u32 = 0xFFFFFFFF;
const PIN_INT: u32 = 7;
const PIN_RST: u32 = 5;
const PIN_BUSY: u32 = 6;
const PIN_DIO8: u32 = 0xFFFFFFFF;

// Radio
const FREQUENCY: f32 = 915.0;
const BANDWIDTH: f32 = 125.0;
const SPREADING_FACTOR: u8 = 7;
const CODING_RATE: u8 = 5;
const SYNC_WORD: u8 = 0x12;
const TX_POWER: i8 = 10;
const PREAMBLE_LENGTH: u16 = 8;
const TCXO_VOLTAGE: f32 = 3.3; //1.8 or 0.0???

fn main() {
    unsafe {
        let ctx = init(
            SPI_CHANNEL,
            SPI_SPEED,
            SPI_DEVICE,
            GPIO_DEVICE,
            PIN_CS,
            PIN_INT,
            PIN_RST,
            PIN_BUSY,
            PIN_DIO8,
        );

        if ctx.is_null() {
            eprintln!("Failed to init");
            return;
        }
        println!("LR1121 initialized");

        // Attempt firmware update
        // println!("Attempting firmware update");
        // let flash_result = flash_firmware(ctx);
        // println!("flash_firmware() returned: {}", flash_result);

        // if flash_result != 0 {
        //     println!("Firmware update failed with code: {}", flash_result);
        //     end(ctx);
        //     return;
        // }

        // println!("Firmware update suceeded");
        // let reset_result = reset(ctx);
        // println!("reset() returned: {}", reset_result);

        println!("Starting radio");

        let begin_result = begin(
            ctx,
            FREQUENCY,
            BANDWIDTH,
            SPREADING_FACTOR,
            CODING_RATE,
            SYNC_WORD,
            TX_POWER,
            PREAMBLE_LENGTH,
            TCXO_VOLTAGE,
        );

        println!("begin() returned: {}", begin_result);

        if begin_result == 0 {
            

            let message = b"hello from rust";
            let transmit_result = transmit(ctx, 0, message.as_ptr(), message.len(), 0);
            println!("transmit() returned: {}", transmit_result);



             
        } else {
            eprintln!("Radio begin failed");
        }

        end(ctx);
        println!("LR1121 ended.");
    }
}


//[ 20 3, 241, 251, 0, 0, 0, 252, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 253, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, ]