#include "DisplayManager.h"

DisplayManager::DisplayManager(Adafruit_SSD1306 *disp)
{
    display = disp;
}

void DisplayManager::drawHeader(uint8_t slaveAddr, uint8_t page)
{
    display->clearDisplay();
    display->setTextSize(1);
    display->setCursor(0, 0);
    display->printf("MOD:SLV | ID:%d   P%d", slaveAddr, page);
    display->drawLine(0, 9, 127, 9, SSD1306_WHITE);
}

void DisplayManager::renderPage1(uint8_t slaveAddr, bool runSym, float curFreq, float setFreq, float minF, float maxF, const char *stateStr)
{
    drawHeader(slaveAddr, 1);

    display->setCursor(0, 12);
    display->printf("RUN: (%s) %06.2f Hz", runSym ? "+" : "-", curFreq);

    display->setCursor(0, 24);
    display->printf("SET: %06.2f Hz", setFreq);

    display->setCursor(0, 36);
    display->printf("LIM: %06.2f-%06.2f", minF, maxF);

    display->drawLine(0, 47, 127, 47, SSD1306_WHITE);

    display->setCursor(0, 52);
    display->print(F("STA: "));
    display->print(stateStr);

    display->display();
}

void DisplayManager::renderPage2(uint8_t slaveAddr, uint8_t baudIdx, uint8_t delayMs, float voltage, float current)
{
    drawHeader(slaveAddr, 2);

    display->setCursor(0, 14);
    display->printf("BAUD: [%d] %s", baudIdx, baudRates[baudIdx]);

    display->setCursor(0, 26);
    display->printf("DELAY: %d ms", delayMs);

    display->setCursor(0, 40);
    display->printf("V-OUT: %6.2f V", voltage);

    display->setCursor(0, 52);
    display->printf("I-OUT: %4.2f A", current);

    display->display();
}

void DisplayManager::renderPage3(uint8_t slaveAddr, int faultCode, int warningCode)
{
    drawHeader(slaveAddr, 3);

    display->setCursor(0, 15);
    display->print(F("--- DIAGNOSTICS ---"));

    display->setCursor(0, 30);
    display->print(F("FAULT: "));
    if (faultCode >= 0 && faultCode <= 38)
    {
        display->printf("ERR-%02d", faultCode);
    }
    else
    {
        display->print(F("NONE"));
    }

    display->setCursor(0, 45);
    display->print(F("WARN:  "));
    if (warningCode == 1 || warningCode == 2 || warningCode == 5)
    {
        display->printf("W-%02d", warningCode);
    }
    else
    {
        display->print(F("NONE"));
    }

    display->display();
}