#include "wrapper.h"
#include <RadioLib.h>
#include "hal/RPi/PiHal.h"

// create a new instance of the HAL class
// use SPI channel 1, because on Waveshare LoRaWAN Hat,
// the LR1121 CS is connected to CE1
// PiHal* hal = new PiHal(0, 2000000, 0, 0);

// now we can create the radio module
// pinout corresponds to the Waveshare LoRaWAN Hat
// NSS pin:   7
// DIO1 pin:  17
// NRST pin:  22
// BUSY pin:  not connected

/// @brief 
struct Context {
    PiHal* hal;
    LR1121* radio;
    ~Context() {
      delete radio;
      delete hal; 
    }
};


// default is int16_t begin(float freq = 434.0, float bw = 125.0, uint8_t sf = 9, uint8_t cr = 7, uint8_t syncWord = RADIOLIB_LR11X0_LORA_SYNC_WORD_PRIVATE, int8_t power = 10, uint16_t preambleLength = 8, float tcxoVoltage = 1.6);
//Module::Module(RadioLibHal *hal, uint32_t cs, uint32_t irq, uint32_t rst, uint32_t gpio)

/// @brief 
/// @param spiChannel 
/// @param spiSpeed 
/// @param spiDevice 
/// @param gpioDevice 
/// @param cs 
/// @param irq 
/// @param rst 
/// @return 
Context* init(uint8_t spiChannel, uint32_t spiSpeed, uint8_t spiDevice, uint8_t gpioDevice, uint32_t cs, uint32_t irq, uint32_t rst) {
    Context* context = new Context();
    context->hal = new PiHal(spiChannel, spiSpeed, spiDevice, gpioDevice);
    // no gpio
    context->radio = &(LR1121)new Module(context->hal, cs, irq, rst, RADIOLIB_NC);
    return context;
    
}

// default is int16_t begin(float freq = 434.0, float bw = 125.0, uint8_t sf = 9, uint8_t cr = 7, uint8_t syncWord = RADIOLIB_LR11X0_LORA_SYNC_WORD_PRIVATE, int8_t power = 10, uint16_t preambleLength = 8, float tcxoVoltage = 1.6);
int begin(Context* context, float freq, float bw, uint8_t sf, uint8_t cr, uint8_t syncWord = RADIOLIB_LR11X0_LORA_SYNC_WORD_PRIVATE, int8_t power = 10, uint16_t preambleLength, float tcxoVoltage) {
    int state = context->radio->begin(freq, bw, sf, cr, syncWord = RADIOLIB_LR11X0_LORA_SYNC_WORD_PRIVATE, power, preambleLength, tcxoVoltage);
    if(state != RADIOLIB_ERR_NONE) {
        printf("failed, code %d\n", state);
        return(1);
    }
    printf("success!\n");
    
    return 0;
}

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
int reveive(Context* context, uint8_t* data, size_t len, RadioLibTime_t timeout) {
  int output = context->radio->receive(data, len, timeout);
  if(output != RADIOLIB_ERR_NONE) {
    printf("Failed RX: %d", output);
  }
  return output;
}
int setFrequency(Context* context, float frequency) {
  int output = context->radio->setFrequency(frequency);
  if(output != RADIOLIB_ERR_NONE) {
    printf("Failed setFrequency: %f", output);
  }
  return output;
}
int setPower(Context* context, uint8_t power) {
  int output = context->radio->setOutputPower(power);
  if(output != RADIOLIB_ERR_NONE) {
    printf("Failed setPower: %f", power);
  }
  return output;
}

int setSpreadingFactor(Context* context, uint8_t sf, bool legacy) {
  int output = context->radio->setSpreadingFactor(sf, legacy);
}
int setCodingRate(Context* context, uint8_t cr, bool longInterleave) {
  int output = context->radio->setCodingRate(cr, longInterleave);
}
int setBitRate(Context* context, float br) {
  int output = context->radio->setBitRate(br);
}
int setPreambleLength(Context* context, size_t preambleLength) { // default 8
  int output = context->radio->setPreambleLength(preambleLength);
}

void end(Context* context) {
  if(context) {
    delete context;
  }
}