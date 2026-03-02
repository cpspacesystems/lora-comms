#include <RadioLib.h>
#include "hal/RPi/PiHal.h"

// create a new instance of the HAL class
// use SPI channel 1, because on Waveshare LoRaWAN Hat,
// the LR1121 CS is connected to CE1
PiHal* hal = new PiHal(0, 2000000, 0, 0);

// now we can create the radio module
// pinout corresponds to the Waveshare LoRaWAN Hat
// NSS pin:   7
// DIO1 pin:  17
// NRST pin:  22
// BUSY pin:  not connected

struct Context {
    PiHal* hal;
    LR1121* radio;
    ~Context() {
      delete radio
      delete hal; 
    }
};

// default is int16_t begin(float freq = 434.0, float bw = 125.0, uint8_t sf = 9, uint8_t cr = 7, uint8_t syncWord = RADIOLIB_LR11X0_LORA_SYNC_WORD_PRIVATE, int8_t power = 10, uint16_t preambleLength = 8, float tcxoVoltage = 1.6);
//Module::Module(RadioLibHal *hal, uint32_t cs, uint32_t irq, uint32_t rst, uint32_t gpio)
Context* init(uint8_t spiChannel, uint32_t spiSpeed, uint8_t spiDevice, uint8_t gpioDevice, uint32_t cs, uint32_t irq, uint32_t rst) {
    Context* context = new Context();
    context->hal = new PiHal(spiChannel, spiSpeed, spiDevice, gpioDevice);
    // no gpio
    context->radio = &(LR1121)new Module(hal, cs, irq, rst, RADIOLIB_NC);
    return context;
    
}

// default is int16_t begin(float freq = 434.0, float bw = 125.0, uint8_t sf = 9, uint8_t cr = 7, uint8_t syncWord = RADIOLIB_LR11X0_LORA_SYNC_WORD_PRIVATE, int8_t power = 10, uint16_t preambleLength = 8, float tcxoVoltage = 1.6);
int begin(Context* context, float freq, float bw, uint8_t sf, uint8_t cr, uint8_t syncWord = RADIOLIB_LR11X0_LORA_SYNC_WORD_PRIVATE, int8_t power = 10, uint16_t preambleLength, float tcxoVoltage) {
    int state = context->radio->begin(freq, bw, sf, cr, syncWord = RADIOLIB_LR11X0_LORA_SYNC_WORD_PRIVATE, power, preambleLength, tcxoVoltage)
    if(state != RADIOLIB_ERR_NONE) {
        printf("failed, code %d\n", state);
        return(1);
    }
    printf("success!\n");
    
    return 0;
}

int trasmit(Context* context, uint16_t delay, uint8_t package, size_t len, uint8_t addr) {
    int state = context->radio->transmit(package, len, addr);
    if(state == RADIOLIB_ERR_NONE) {
      printf("success!\n");
      hal->delay(delay);
    } else {
      printf("failed, code %d\n", state);
    }
  return(0);

}

void end(Context* context) {
  if(context) {
    delete context;
  }
}