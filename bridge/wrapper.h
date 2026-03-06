#pragma once
#include <stdint.h>
#include <stddef.h>

#define RADIOLIB_LR11X0_LORA_SYNC_WORD_PRIVATE 0x12

typedef struct Context context;
typedef uint32_t RadioLibTime_t;

#ifdef __cplusplus
extern "C" {
#endif

Context* init(uint8_t spiChannel, uint32_t spiSpeed, uint8_t spiDevice, uint8_t gpioDevice, uint32_t cs, uint32_t irq, uint32_t rst);

int begin(Context* context, float freq, float bw, uint8_t sf, uint8_t cr, uint8_t syncWord = RADIOLIB_LR11X0_LORA_SYNC_WORD_PRIVATE, int8_t power = 10, uint16_t preambleLength, float tcxoVoltage);

int trasmit(Context* context, uint16_t delay, const uint8_t* package, size_t len, uint8_t addr);

int reveive(Context* context, uint8_t* data, size_t len, RadioLibTime_t timeout);
int setFrequency(Context* context, float frequency);

int setPower(Context* context, uint8_t power);

int setSpreadingFactor(Context* context, uint8_t sf, bool legacy);

int setCodingRate(Context* context, uint8_t cr, bool longInterleave);

int setBitRate(Context* context, float br);

int setPreambleLength(Context* context, size_t preambleLength);

void end(Context* context);

#ifdef __cplusplus
}
#endif