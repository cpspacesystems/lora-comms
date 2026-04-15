#define RADIOLIB_LR1121_FIRMWARE_0103

#include "wrapper.h"
#include <RadioLib.h>
#include "hal/RPi/PiHal.h"
#include "modules/LR11x0/firmware/lr1121_transceiver_0103.h"



#ifdef __cplusplus
extern "C" {
#endif

struct Context {
    PiHal* hal;
    LR1121* radio;
    ~Context() {
      delete radio;
      this->hal->term();
      delete hal; 
    }
};




Context* init(uint8_t spiChannel, uint32_t spiSpeed, uint8_t spiDevice, 
              uint8_t gpioDevice, uint32_t cs, uint32_t irq, uint32_t rst, 
              uint32_t busy, uint32_t dio8) {
    Context* ctx = new Context();
    ctx->hal = new PiHal(spiChannel, spiSpeed, spiDevice, gpioDevice, busy);
    ctx->hal->init();

    // Setup RST pin
    ctx->hal->pinMode(rst, ctx->hal->GpioModeOutput);
    ctx->hal->digitalWrite(rst, ctx->hal->GpioLevelHigh);
    ctx->hal->delay(10);
    
    // Setup DIO8
    ctx->hal->pinMode(dio8, ctx->hal->GpioModeOutput);
    
    // TRY DIO8 LOW for normal operation mode
    printf("Setting DIO8 LOW for normal mode\n");
    ctx->hal->digitalWrite(dio8, ctx->hal->GpioLevelLow);
    ctx->hal->delay(10);

    // Pulse RST
    ctx->hal->digitalWrite(rst, ctx->hal->GpioLevelLow);
    ctx->hal->delay(50);
    ctx->hal->digitalWrite(rst, ctx->hal->GpioLevelHigh);
    ctx->hal->delay(100);

    // Wait for BUSY
    uint32_t elapsed = 0;
    while(ctx->hal->digitalRead(busy) == 0 && elapsed < 500) {
        ctx->hal->delay(10);
        elapsed += 10;
    }

    elapsed = 0;
    while(ctx->hal->digitalRead(busy) == 1 && elapsed < 3000) {
        ctx->hal->delay(10);
        elapsed += 10;
    }

    ctx->radio = new LR1121(new Module(ctx->hal, cs, irq, rst, busy));
    return ctx;
}

  int begin(Context* context, float freq, float bw, uint8_t sf, uint8_t cr, uint8_t syncWord, int8_t power, uint16_t preambleLength, float tcxoVoltage) {
      printf("begin called: freq=%.1f tcxo=%.1f\n", freq, tcxoVoltage);


      context->radio->reset();
      context->hal->delay(500);  


      
      context->radio->XTAL = (tcxoVoltage < 0.1);

      int state = context->radio->begin(freq, bw, sf, cr, syncWord, power, preambleLength, tcxoVoltage);
      
      
      printf("Final state=%d\n", state);
      return state;
  }


  /// @brief 
  /// @param context 
  /// @param delay 
  /// @param package 
  /// @param len 
  /// @param addr 
  /// @return 
  int trasmit(Context* context, uint16_t delay, const uint8_t* package, size_t len, uint8_t addr) {
      int state = context->radio->transmit(package, len, addr);
      if(state == RADIOLIB_ERR_NONE) {
        printf("success!\n");
        context->hal->delay(delay);
      } else {
        printf("failed, code %d\n", state);
      }
    return(0);
  }
  /// @brief 
  /// @param context 
  /// @param data 
  /// @param len 
  /// @param timeout 
  /// @return 
  int receive(Context* context, uint8_t* data, size_t len, RadioLibTime_t timeout) {
    return context->radio->receive(data, len, timeout);
  }
  /// @brief 
  /// @param context 
  /// @param frequency 
  /// @return 
  int setFrequency(Context* context, float frequency) {
    return context->radio->setFrequency(frequency);
  }
  /// @brief 
  /// @param context 
  /// @param power 
  /// @return 
  int setPower(Context* context, uint8_t power) {
    return context->radio->setOutputPower(power);
  }
  /// @brief 
  /// @param context 
  /// @param sf 
  /// @param legacy 
/// @return 
int setSpreadingFactor(Context* context, uint8_t sf, bool legacy) {
  return context->radio->setSpreadingFactor(sf, legacy);
}
/// @brief 
/// @param context 
/// @param cr 
/// @param longInterleave 
/// @return 
int setCodingRate(Context* context, uint8_t cr, bool longInterleave) {
  return context->radio->setCodingRate(cr, longInterleave);
}
/// @brief 
/// @param context 
/// @param br 
/// @return 
int setBitRate(Context* context, float br) {
  return context->radio->setBitRate(br);
}
/// @brief 
/// @param context 
/// @param preambleLength 
/// @return 
int setPreambleLength(Context* context, size_t preambleLength) { // default 8
  return context->radio->setPreambleLength(preambleLength);
}
/// @brief 
/// @param context 
void end(Context* context) {
  if(context) {
    delete context;
  }
}

int flash_firmware(Context* context) {
    printf("Flashing LR1121 firmware...\n");

    // 1. Attempt standard RadioLib update
    int16_t state = context->radio->updateFirmware(
        lr11xx_firmware_image,
        LR11XX_FIRMWARE_IMAGE_SIZE
    );

    if (state == RADIOLIB_ERR_NONE) {
        printf("Standard flash successful.\n");
        return 0;
    }

    printf("Standard flash failed (%d). Trying manual write...\n", state);

    // 2. Manual fallback using proven parameters
    Module* mod = context->radio->getMod();

    // Reset into bootloader
    context->hal->digitalWrite(17, context->hal->GpioLevelLow);
    context->hal->delay(50);
    context->hal->digitalWrite(17, context->hal->GpioLevelHigh);
    context->hal->delay(200);

    // Erase flash (0x8000)
    mod->SPIwriteStream(0x8000, NULL, 0, false, false);
    context->hal->delay(3000);
    printf("Erase complete.\n");

    // Write firmware in 16‑word chunks using encrypted command 0x8001
    const size_t chunkSize = 16;
    size_t totalWords = LR11XX_FIRMWARE_IMAGE_SIZE;
    uint8_t buffer[4 + chunkSize * 4];

    for (size_t i = 0; i < totalWords; i += chunkSize) {
        size_t words = (totalWords - i < chunkSize) ? (totalWords - i) : chunkSize;
        uint32_t offset = i * 4;

        buffer[0] = (offset >> 24) & 0xFF;
        buffer[1] = (offset >> 16) & 0xFF;
        buffer[2] = (offset >> 8) & 0xFF;
        buffer[3] = offset & 0xFF;

        for (size_t j = 0; j < words; j++) {
            uint32_t w = lr11xx_firmware_image[i + j];
            buffer[4 + j*4] = (w >> 24) & 0xFF;
            buffer[5 + j*4] = (w >> 16) & 0xFF;
            buffer[6 + j*4] = (w >> 8) & 0xFF;
            buffer[7 + j*4] = w & 0xFF;
        }

        state = mod->SPIwriteStream(0x8001, buffer, 4 + words * 4, false, false);
        if (state != 0) {
            printf("Write failed at chunk %zu (error %d)\n", i, state);
            return -1;
        }
        context->hal->delay(10);
    }

    printf("Manual flash complete.\n");

    // Reboot
    context->hal->digitalWrite(17, context->hal->GpioLevelLow);
    context->hal->delay(50);
    context->hal->digitalWrite(17, context->hal->GpioLevelHigh);
    context->hal->delay(500);

    return 0;
}

int16_t reset(Context* context) {
  return context->radio->reset();
}


#ifdef __cplusplus
}
#endif