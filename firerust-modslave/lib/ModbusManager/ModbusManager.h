#ifndef MODBUS_MANAGER_H
#define MODBUS_MANAGER_H

#include <Arduino.h>
#include <ModbusRTU.h>

class ModbusManager
{
private:
    ModbusRTU mb;

    // Direcciones Modbus - Holding Registers (Lectura/Escritura)
    const uint16_t ADDR_SET_FREQ = 0x2001;
    const uint16_t ADDR_MAX_FREQ = 0x0001;
    const uint16_t ADDR_MIN_FREQ = 0x0002;
    const uint16_t ADDR_BAUD_IDX = 0x0003;
    const uint16_t ADDR_DIR_CMD = 0x0010; // Direccion de giro

    // Direcciones Modbus - Input Registers (Solo Lectura)
    const uint16_t ADDR_CUR_FREQ = 0x3001;
    const uint16_t ADDR_OUT_VOLT = 0x3002;
    const uint16_t ADDR_OUT_CURR = 0x3003;
    const uint16_t ADDR_STATUS = 0x3000;

public:
    // Configura el puerto serie y los registros Modbus
    void begin(int SERIAL_SPEED, uint8_t slaveId, int rxPin, int txPin, int enPin);

    // Mantiene la comunicación viva
    void task();

    // Sincroniza desde Modbus hacia el ESP32 (Lee comandos del Maestro)
    void syncFromModbus(float &setFreq, float &minFreq, float &maxFreq, uint8_t &baudIdx, uint16_t &dirCmd);

    // Sincroniza desde el ESP32 hacia Modbus (Actualiza estado al Maestro)
    void syncToModbus(float curFreq, float voltage, float current, uint8_t statusIdx);
};

#endif