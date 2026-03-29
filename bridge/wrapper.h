#pragma once
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#define RADIOLIB_LR11X0_LORA_SYNC_WORD_PRIVATE 0x12

typedef struct Context context;
typedef uint32_t RadioLibTime_t;

#ifdef __cplusplus
extern "C" {
#endif

/// @brief 
/// @param spiChannel 
/// @param spiSpeed 
/// @param spiDevice 
/// @param gpioDevice 
/// @param cs 
/// @param irq 
/// @param rst 
/// @return 
Context* init(uint8_t spiChannel, uint32_t spiSpeed, uint8_t spiDevice, uint8_t gpioDevice, uint32_t cs, uint32_t irq, uint32_t rst);


/// @brief 
/// @param context 
/// @param freq 
/// @param bw 
/// @param sf 
/// @param cr 
/// @param syncWord = RADIOLIB_LR11X0_LORA_SYNC_WORD_PRIVATE
/// @param power = 10
/// @param preambleLength 
/// @param tcxoVoltage 
/// @return 
int begin(Context* context, float freq, float bw, uint8_t sf, uint8_t cr, uint8_t syncWord, int8_t power, uint16_t preambleLength, float tcxoVoltage);

/// @brief 
/// @param context 
/// @param delay 
/// @param package 
/// @param len 
/// @param addr 
/// @return 
int trasmit(Context* context, uint16_t delay, const uint8_t* package, size_t len, uint8_t addr);

/// @brief 
/// @param context 
/// @param data 
/// @param len 
/// @param timeout 
/// @return 
int receive(Context* context, uint8_t* data, size_t len, RadioLibTime_t timeout);

/// @brief 
/// @param context 
/// @param frequency 
/// @return 
int setFrequency(Context* context, float frequency);

/// @brief 
/// @param context 
/// @param power 
/// @return 
int setPower(Context* context, uint8_t power);

/// @brief 
/// @param context 
/// @param sf 
/// @param legacy 
/// @return 
int setSpreadingFactor(Context* context, uint8_t sf, bool legacy);

/// @brief 
/// @param context 
/// @param cr 
/// @param longInterleave 
/// @return 
int setCodingRate(Context* context, uint8_t cr, bool longInterleave);

/// @brief 
/// @param context 
/// @param br 
/// @return 
int setBitRate(Context* context, float br);

/// @brief 
/// @param context 
/// @param preambleLength 
/// @return 
int setPreambleLength(Context* context, size_t preambleLength);

/// @brief 
/// @param context 
void end(Context* context);

#ifdef __cplusplus
}
#endif