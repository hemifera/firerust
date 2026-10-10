#include "ModbusManager.h"

void ModbusManager::begin(int SERIAL_SPEED, uint8_t slaveId, int rxPin, int txPin, int enPin)
{
    // Inicialización del Serial1 para Modbus
    Serial1.begin(SERIAL_SPEED, SERIAL_8O1, rxPin, txPin);

    // Configuración del esclavo Modbus con su pin de control (RTS) y el ID
    mb.begin(&Serial1, enPin);
    mb.server(slaveId);

    // 1. Holding Registers (Escritura/Lectura)
    mb.addHreg(ADDR_SET_FREQ, 0);
    mb.addHreg(ADDR_MAX_FREQ, 6000); // 60.00 Hz * 100
    mb.addHreg(ADDR_MIN_FREQ, 0);
    mb.addHreg(ADDR_BAUD_IDX, 3); // Baud 9600 por defecto
    mb.addHreg(ADDR_DIR_CMD, 1);

    // 2. Input Registers. Solo lectura
    mb.addIreg(ADDR_CUR_FREQ, 0);
    mb.addIreg(ADDR_OUT_VOLT, 0);
    mb.addIreg(ADDR_OUT_CURR, 0);
    mb.addIreg(ADDR_STATUS, 0);
}

void ModbusManager::task()
{
    mb.task(); // Procesa las peticiones entrantes del maestro Modbus
}

void ModbusManager::syncFromModbus(float &setFreq, float &minFreq, float &maxFreq, uint8_t &baudIdx, uint16_t &dirCmd)
{
    // Dividimos entre 100 para restaurar los decimales
    setFreq = mb.Hreg(ADDR_SET_FREQ) / 100.0;
    maxFreq = mb.Hreg(ADDR_MAX_FREQ) / 100.0;
    minFreq = mb.Hreg(ADDR_MIN_FREQ) / 100.0;
    baudIdx = mb.Hreg(ADDR_BAUD_IDX);
    dirCmd = mb.Hreg(ADDR_DIR_CMD);
}

void ModbusManager::syncToModbus(float curFreq, float voltage, float current, uint8_t statusVal)
{
    // Multiplicamos para enviar la precisión decimal requerida como número entero
    mb.Ireg(ADDR_CUR_FREQ, (uint16_t)(curFreq * 100));
    mb.Ireg(ADDR_OUT_VOLT, (uint16_t)(voltage * 10));  // Un decimal para el voltaje
    mb.Ireg(ADDR_OUT_CURR, (uint16_t)(current * 100)); // Dos decimales para la corriente
    mb.Ireg(ADDR_STATUS, statusVal);
}