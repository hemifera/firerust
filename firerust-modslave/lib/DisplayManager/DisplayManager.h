#ifndef DISPLAY_MANAGER_H
#define DISPLAY_MANAGER_H

#include <Arduino.h>
#include <Wire.h>
#include <Adafruit_GFX.h>
#include <Adafruit_SSD1306.h>

class DisplayManager
{
private:
    Adafruit_SSD1306 *display;
    const char *baudRates[8] = {"1200", "2400", "4800", "9600", "19200", "38400", "57600", "115200"};

    void drawHeader(uint8_t slaveAddr, uint8_t page);

public:
    DisplayManager(Adafruit_SSD1306 *disp);

    void renderPage1(uint8_t slaveAddr, bool runSym, float curFreq, float setFreq, float minF, float maxF, const char *stateStr);
    void renderPage2(uint8_t slaveAddr, uint8_t baudIdx, uint8_t delayMs, float voltage, float current);
    void renderPage3(uint8_t slaveAddr, int faultCode, int warningCode);
};

#endif