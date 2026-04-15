#!/bin/sh

# This script is intended to be used on SX1302 CoreCell platform, it performs
# the following actions:
#       - export/unpexort GPIO23 and GPIO18 used to reset the SX1302 chip and to enable the LDOs
#       - export/unexport GPIO22 used to reset the optional SX1261 radio used for LBT/Spectral Scan
#
# Usage examples:
#       ./reset_lgw.sh stop
#       ./reset_lgw.sh start

# GPIO mapping has to be adapted with HW
#

SX1302_RESET_PIN=17     # SX1302 reset
SX1302_POWER_EN_PIN=18  # SX1302 power enable
SX1261_RESET_PIN=5     # SX1261 reset (LBT / Spectral Scan)
AD5338R_RESET_PIN=13    # AD5338R reset (full-duplex CN490 reference design)

WAIT_GPIO() {
    sleep 0.1
}

init() {
    # set GPIOs as output
    raspi-gpio set $SX1302_RESET_PIN op; WAIT_GPIO
    raspi-gpio set $SX1261_RESET_PIN op; WAIT_GPIO
    raspi-gpio set $SX1302_POWER_EN_PIN op; WAIT_GPIO
    raspi-gpio set $AD5338R_RESET_PIN op; WAIT_GPIO
}

reset() {
    echo "CoreCell reset through GPIO$SX1302_RESET_PIN..."
    echo "SX1261 reset through GPIO$SX1302_RESET_PIN..."
    echo "CoreCell power enable through GPIO$SX1302_POWER_EN_PIN..."
    echo "CoreCell ADC reset through GPIO$AD5338R_RESET_PIN..."

    # write output for SX1302 CoreCell power_enable and reset
    raspi-gpio set $SX1302_POWER_EN_PIN dh; WAIT_GPIO

    raspi-gpio set $SX1302_RESET_PIN dh; WAIT_GPIO
    raspi-gpio set $SX1302_RESET_PIN dl; WAIT_GPIO

    raspi-gpio set $SX1261_RESET_PIN dl; WAIT_GPIO
    raspi-gpio set $SX1261_RESET_PIN dh; WAIT_GPIO

    raspi-gpio set $AD5338R_RESET_PIN dl; WAIT_GPIO
    raspi-gpio set $AD5338R_RESET_PIN dh; WAIT_GPIO
}

term() {
    # cleanup all GPIOs
    if [ -d /sys/class/gpio/gpio$SX1302_RESET_PIN ]
    then
        raspi-gpio set $SX1302_RESET_PIN ip; WAIT_GPIO
    fi
    if [ -d /sys/class/gpio/gpio$SX1261_RESET_PIN ]
    then
        raspi-gpio set $SX1261_RESET_PIN ip; WAIT_GPIO
    fi
    if [ -d /sys/class/gpio/gpio$SX1302_POWER_EN_PIN ]
    then
        raspi-gpio set $SX1302_POWER_EN_PIN ip; WAIT_GPIO
    fi
    if [ -d /sys/class/gpio/gpio$AD5338R_RESET_PIN ]
    then
        raspi-gpio set $AD5338R_RESET_PIN ip; WAIT_GPIO
    fi
}

case "$1" in
    start)
    term # just in case
    init
    reset
    ;;
    stop)
    reset
    term
    ;;
    *)
    echo "Usage: $0 {start|stop}"
    exit 1
    ;;
esac

exit 0
