#include <Arduino.h>
#include <SPI.h>
#include <Wire.h>
#include <Adafruit_GFX.h>
#include <Adafruit_SSD1306.h>
#include "DisplayManager.h"
#include "ModbusManager.h"

#define SCREEN_WIDTH 128
#define SCREEN_HEIGHT 64
#define OLED_RESET -1
#define SCREEN_ADDRESS 0x3C
#define I2C_SDA 33
#define I2C_SCL 32

#define BTN_PAGE 26

const int RX_PIN = 23;
const int TX_PIN = 22;
const int EN_PIN = 21;
const int MODBUS_SPEED = 9600;

Adafruit_SSD1306 oled(SCREEN_WIDTH, SCREEN_HEIGHT, &Wire, OLED_RESET);
DisplayManager ui(&oled);
ModbusManager vfdModbus;

// Interfaz de pantalla

// Variables Pág 1
uint8_t SLAVE_ADDR = 1;
// Top header en pantalla
// // MODO: SLAVE | SLAVE ADDR: {SLAVE_ADDR}

// // Header en pantalla
bool RUNNING_SYMBOL = true; // Indica signo mas (True) o menos (False) segun la direccion del giro
float CURRENT_FREQ = 0.0;
float SET_FREQ = 0.0;
float STEP_FREQ = 0.01;
float RAMP_TIME = 5.0; // tiempo para que CURRENT_FREQ alcance SET_FREQ

// Variables de Límites de Frecuencia (restringen el SET_FREQ)
float FREQ_MIN = 0.00;
float FREQ_MAX = 60.00; // Puedes ajustarlo hasta 600.00 Hz si lo deseas

// Mostrar en pantalla la siguinte linea
// // RUN FREQ: (+/-) 040.00 Hz
// // // +/- corrensponde a RUNNING_SYMBOL
// // // 040.00 corresponde a CURRENT_FREQ, debe mostrar 3 digitos enteros y dos digitos decimal

const char *VFD_STATES[6] = {
    "STOPPED",     // 0: Detenido
    "RUNNING FWD", // 1: Marcha adelante
    "RUNNING REV", // 2: Marcha atrás
    "STOPPING",    // 3: Frenando por rampa
    "FAST STOP",   // 4: Parada rápida / inercia
    "FAULT STOP"   // 5: Detenido por fallo/alarma
};

uint8_t currentStateIndex = 0;

// Variables Pág 2
uint8_t modbusBaudIndex = 3;
uint8_t modbusDelayMs = 10;
float outVoltage = 0.0;
float outCurrent = 0.0;

// Variables Pág 3
int currentFault = -1;
int currentWarning = 2;

// Control de Páginas y Botón
volatile uint8_t currentPage = 1;
volatile unsigned long lastInterruptTime = 0;

// Rutina de Servicio de Interrupción (ISR) para el botón con Pull-Down
void IRAM_ATTR handleButtonPress()
{
  unsigned long interruptTime = millis();

  // Antirrebote por software de 200 ms dentro de la interrupción
  if (interruptTime - lastInterruptTime > 200)
  {
    currentPage++;
    if (currentPage > 3)
    {
      currentPage = 1;
    }
    lastInterruptTime = interruptTime;
  }
}

// Control de Tiempos Loop
unsigned long previousMillis = 0;
const long intervalCycle = 10000;
unsigned long lastUpdateTime = 0;

void setup()
{
  Serial.begin(115200);
  pinMode(BTN_PAGE, INPUT);
  attachInterrupt(digitalPinToInterrupt(BTN_PAGE), handleButtonPress, RISING);
  Wire.begin(I2C_SDA, I2C_SCL);

  if (!oled.begin(SSD1306_SWITCHCAPVCC, SCREEN_ADDRESS))
  {
    Serial.println(F("SSD1306 allocation failed"));
    for (;;)
      ;
  }

  oled.clearDisplay();
  oled.setTextSize(1);
  oled.setTextColor(SSD1306_WHITE);
  oled.setCursor(0, 0);
  oled.println(F("Arrancando sistema..."));
  oled.display();
  delay(1000);

  vfdModbus.begin(MODBUS_SPEED, SLAVE_ADDR, RX_PIN, TX_PIN, EN_PIN);

  previousMillis = millis();
  lastUpdateTime = millis();
}

void loop()
{
  vfdModbus.task();

  uint16_t dirCmd = 1;

  vfdModbus.syncFromModbus(SET_FREQ, FREQ_MIN, FREQ_MAX, modbusBaudIndex, dirCmd);

  if (SET_FREQ > FREQ_MAX)
    SET_FREQ = FREQ_MAX;
  if (SET_FREQ < FREQ_MIN)
    SET_FREQ = FREQ_MIN;

  int currentDir = RUNNING_SYMBOL ? 1 : -1; // 1: FWD, -1: REV
  int targetDir = (dirCmd == 2) ? -1 : 1;
  if (dirCmd == 0)
    targetDir = 0;

  static enum
  {
    NORMAL,
    DECEL_FOR_REVERSE,
    ACCEL_NEW_DIR
  } motorState = NORMAL;
  float targetMag = SET_FREQ;
  uint16_t statusValue = 0x0000;

  if (dirCmd == 0)
  {
    targetMag = 0.0;
  }

  switch (motorState)
  {
  case NORMAL:
    if (dirCmd != 0 && targetDir != currentDir && CURRENT_FREQ > 0.0)
    {
      // Se pidió cambiar de dirección en marcha -> Iniciar rampa descendente a 0
      motorState = DECEL_FOR_REVERSE;
    }
    else
    {
      if (CURRENT_FREQ == 0.0 && (dirCmd == 0 || SET_FREQ == 0.0))
      {
        statusValue = 0x0000; // STOPPED
        currentStateIndex = 0;
      }
      else
      {
        statusValue = (currentDir == 1) ? 0x0001 : 0x0002;
        currentStateIndex = (currentDir == 1) ? 1 : 2;
      }
    }
    break;

  case DECEL_FOR_REVERSE:
    targetMag = 0.0;
    // Mientras reduce, mantiene el estado del sentido actual en ADDR_STATUS
    statusValue = (currentDir == 1) ? 0x0001 : 0x0002;
    currentStateIndex = (currentDir == 1) ? 1 : 2;

    if (CURRENT_FREQ <= 0.01)
    {
      CURRENT_FREQ = 0.0;
      // Al llegar a cero exacto, ADDR_STATUS pasa momentáneamente a 0x0000
      statusValue = 0x0000;
      currentStateIndex = 0;

      // Cambiar el sentido de giro y signo en pantalla
      currentDir = -currentDir;
      RUNNING_SYMBOL = (currentDir == 1);

      // Pasar a acelerar en el nuevo sentido
      motorState = ACCEL_NEW_DIR;
    }
    break;

  case ACCEL_NEW_DIR:
    targetMag = SET_FREQ;
    statusValue = (currentDir == 1) ? 0x0001 : 0x0002;
    currentStateIndex = (currentDir == 1) ? 1 : 2;

    if (CURRENT_FREQ >= SET_FREQ || SET_FREQ == 0.0 || dirCmd == 0)
    {
      motorState = NORMAL;
    }
    break;
  }

  // Lógica de Rampa y Control Físico del Motor
  unsigned long currentMillis = millis();
  float dt = (currentMillis - lastUpdateTime) / 1000.0;
  lastUpdateTime = currentMillis;
  if (dt > 0.1)
    dt = 0.1;

  if (CURRENT_FREQ < targetMag)
  {
    CURRENT_FREQ += ((FREQ_MAX - FREQ_MIN) / RAMP_TIME) * dt;
    if (CURRENT_FREQ > targetMag)
      CURRENT_FREQ = targetMag;
  }
  else if (CURRENT_FREQ > targetMag)
  {
    CURRENT_FREQ -= ((FREQ_MAX - FREQ_MIN) / RAMP_TIME) * dt;
    if (CURRENT_FREQ < targetMag)
      CURRENT_FREQ = targetMag;
  }

  // Simulación física de valores eléctricos
  if (statusValue == 0x0000 && CURRENT_FREQ == 0.0)
  {
    outVoltage = 0.0;
    outCurrent = 0.0;
  }
  else
  {
    outVoltage = (CURRENT_FREQ / 60.0) * 225.12;
    if (outVoltage < 15.0)
      outVoltage = 15.0;
    if (CURRENT_FREQ > 0.0)
    {
      outCurrent = 0.1 + ((CURRENT_FREQ / 60.0) * 3.0);
    }
    else
    {
      outCurrent = 0.0;
    }
  }
  if (outCurrent > 3.20)
    outCurrent = 3.20;

  // 6. Actualizar las variables calculadas de nuevo hacia la memoria del Esclavo Modbus
  vfdModbus.syncToModbus(CURRENT_FREQ, outVoltage, outCurrent, statusValue);

  // Actualización visual por medio de la clase modular según la página actual
  switch (currentPage)
  {
  case 1:
    ui.renderPage1(SLAVE_ADDR, RUNNING_SYMBOL, CURRENT_FREQ, SET_FREQ, FREQ_MIN, FREQ_MAX, VFD_STATES[currentStateIndex]);
    break;
  case 2:
    ui.renderPage2(SLAVE_ADDR, modbusBaudIndex, modbusDelayMs, outVoltage, outCurrent);
    break;
  case 3:
    ui.renderPage3(SLAVE_ADDR, currentFault, currentWarning);
    break;
  }

  delay(10);
}