#define RADIOLIB_LR1121_FIRMWARE_0103
#include "wrapper.h"

#include <RadioLib.h>
#include "hal/RPi/PiHal.h"
#include "modules/LR11x0/firmware/lr1121_transceiver_0103.h"

#ifdef __cplusplus
extern "C" {
#endif

struct Context {
    PiHal*   hal;
    Module*  mod;
    LR1121*  radio;

    uint32_t pin_cs;
    uint32_t pin_irq;
    uint32_t pin_rst;
    uint32_t pin_busy;
    uint32_t pin_dio8;

    ~Context() {
        delete radio;
        delete mod;
        hal->term();
        delete hal;
    }
};

Context* init(uint8_t spiChannel, uint32_t spiSpeed, uint8_t spiDevice,
              uint8_t gpioDevice, uint32_t cs, uint32_t irq, uint32_t rst,
              uint32_t busy, uint32_t dio8) {
    Context* ctx = new Context();

    ctx->pin_cs   = cs;
    ctx->pin_irq  = irq;
    ctx->pin_rst  = rst;
    ctx->pin_busy = busy;
    ctx->pin_dio8 = dio8;

    ctx->hal = new PiHal(spiChannel, spiSpeed, spiDevice, gpioDevice);
    ctx->hal->init();

    // Setup RST pin
    ctx->hal->pinMode(rst, ctx->hal->GpioModeOutput);
    ctx->hal->digitalWrite(rst, ctx->hal->GpioLevelHigh);
    ctx->hal->delay(10);

    // Setup DIO8 - LOW for normal mode
    ctx->hal->pinMode(dio8, ctx->hal->GpioModeOutput);
    printf("Setting DIO8 LOW for normal mode\n");
    ctx->hal->digitalWrite(dio8, ctx->hal->GpioLevelLow);
    ctx->hal->delay(10);

    // Pulse RST
    ctx->hal->digitalWrite(rst, ctx->hal->GpioLevelLow);
    ctx->hal->delay(50);
    ctx->hal->digitalWrite(rst, ctx->hal->GpioLevelHigh);
    ctx->hal->delay(100);

    // Wait for BUSY to go high then low
    uint32_t elapsed = 0;
    while (ctx->hal->digitalRead(busy) == 0 && elapsed < 500) {
        ctx->hal->delay(10);
        elapsed += 10;
    }
    elapsed = 0;
    while (ctx->hal->digitalRead(busy) == 1 && elapsed < 3000) {
        ctx->hal->delay(10);
        elapsed += 10;
    }

    ctx->mod   = new Module(ctx->hal, cs, irq, rst, busy);
    ctx->radio = new LR1121(ctx->mod);

    uint8_t device = 0x00;
    printf("Versionhbjvfduebhjfidejb: '%d'\n", ctx->radio->getVersion(NULL, &device, NULL, NULL));

    return ctx;
}

int begin(Context* ctx, float freq, float bw, uint8_t sf, uint8_t cr,
          uint8_t syncWord, int8_t power, uint16_t preambleLength, float tcxoVoltage) {
    printf("begin: freq=%.1f tcxo=%.1f\n", freq, tcxoVoltage);
    ctx->radio->reset();
    //ctx->hal->delay(500);
    //ctx->radio->XTAL = false; ///????
    int state = ctx->radio->begin(freq, bw, sf, cr, syncWord, power, preambleLength, tcxoVoltage);
    printf("Final state=%d\n", state);
    return state;
}

int transmit(Context* ctx, uint16_t delay_ms, const uint8_t* data, size_t len, uint8_t addr) {
    int state = ctx->radio->transmit(data, len, addr);
    if (state == RADIOLIB_ERR_NONE) {
        printf("Transmit success!\n");
        ctx->hal->delay(delay_ms);
    } else {
        printf("Transmit failed, code %d\n", state);
    }
    return state;
}

int receive(Context* ctx, uint8_t* data, size_t len, RadioLibTime_t timeout) {
    return ctx->radio->receive(data, len, timeout);
}

int setFrequency(Context* ctx, float frequency) {
    return ctx->radio->setFrequency(frequency);
}

int setPower(Context* ctx, uint8_t power) {
    return ctx->radio->setOutputPower(power);
}

int setSpreadingFactor(Context* ctx, uint8_t sf, bool legacy) {
    return ctx->radio->setSpreadingFactor(sf, legacy);
}

int setCodingRate(Context* ctx, uint8_t cr, bool longInterleave) {
    return ctx->radio->setCodingRate(cr, longInterleave);
}

int setBitRate(Context* ctx, float br) {
    return ctx->radio->setBitRate(br);
}

int setPreambleLength(Context* ctx, size_t preambleLength) {
    return ctx->radio->setPreambleLength(preambleLength);
}

void end(Context* ctx) {
    if (ctx) {
        delete ctx;
    }
}

int flash_firmware(Context* ctx) {
    printf("Entering bootloader mode...\n");

    // DIO8 HIGH before reset = bootloader mode
    ctx->hal->digitalWrite(ctx->pin_dio8, ctx->hal->GpioLevelHigh);
    ctx->hal->delay(10);

    // Reset into bootloader
    ctx->radio->reset();
    ctx->hal->delay(500);

    // Wait for BUSY to clear
    uint32_t elapsed = 0;
    while (ctx->hal->digitalRead(ctx->pin_busy) == 1 && elapsed < 3000) {
        ctx->hal->delay(10);
        elapsed += 10;
    }

    printf("Flashing LR1121 firmware...\n");
    int16_t state = ctx->radio->updateFirmware(
        lr11xx_firmware_image,
        LR11XX_FIRMWARE_IMAGE_SIZE
    );
    printf("updateFirmware returned: %d\n", state);

    // Return DIO8 LOW for normal mode
    ctx->hal->digitalWrite(ctx->pin_dio8, ctx->hal->GpioLevelLow);
    ctx->hal->delay(10);

    if (state != RADIOLIB_ERR_NONE) {
        printf("Flash failed: %d\n", state);
        return (int)state;
    }

    printf("Flash successful.\n");
    return 0;
}

int16_t reset(Context* ctx) {
    return ctx->radio->reset();
}

float getSNR(Context* ctx) {
    return ctx->radio->getSNR();
}

#ifdef __cplusplus
}
#endif
