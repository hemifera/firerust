# Modbus memory registers and screen status

El ESP32 simulará un variador de frecuencia basado en el VFD500 de la marca Veikong, se tomara de referencia las instrucciones del variador pero se cambiarán los datos.

Dentro del ESP32 se implementa una pantalla de estado, el modelo es la SSD1306, de resolución de 128x64 pixeles. Tambien se agrega un boton para cambiar de paginas, y las paginas son secciones de informacion especificas.

## Paginacion

### (1) Sección del estado de arranque del motor

1. Modalidad de comunicacion esclavo-maestro modbus
   
   **Texto a mostrar**: "MOD: SLAVE"
   
   **Explicación**: El ESP32 actuara como esclavo modbus en la comunicación, este solo podra recibir instrucciones, ya sea de lectura, escritura de su memoria.

2. Interfaz identificador de esclavo modbus
   
   **Texto a mostrar**: "ID: {modbus-id}"

   **Explicacion**: Se asigna la direccion de esclavo correspondiente al ESP32, modbus-id, sera definida posterior en programacion. Alta posibilidad que sea 1.

3. Paginación
   **Texto amostart**: "P{numero-pagina}"

   **Explicacion**: Muestra el número de pagina actual que se encuentra la interfaz.

Los numeros 1, 2 y 3 estaran ordenados en la misma linea, y seran parte de un encabezado que se mostrará en cualquiera de las 3 paginas.

4. Frecuencia actual del variador

   Texto a mostrar: (+/-) 000.00 Hz

   Explicacion: La porcion de "(+/-)" indica que el motor se encuentra girando en sentido comun (+) o sentido inverso (-). Su direccion corresponde a bobina en direccion 0x0006. El valor de "000.00 Hz" es el indicador de frecuencia actual, es un registro de solo lectura, ubicado en 0x0106.
   
   Direccion 0x0006: registro lectura y escritura. Los primeros 3 bytes son de 

### (2) Sección del estado
P30.01 ID: 1
P01.06 
RUNNING FREQ: (+-) 40.6 Hz
Rotation direction P00.08 (0 or 1)
P01.08 SET FREQ: 60.00Hz

0x6000


0,1,2,3,4,5
RUNNING FWD/REV // STOPPING/STOPPED/FAST STOP // FAULT STOP
LIMITS (0.00-600.00 Hz) 65535RPM, 100% (considers limits)
Maximum frequency P01.06 (Hz only up to 600.00Hz)
Lower limit frequency P01.14 (Hz only)

FAILURE (BLINKING)
FAILURE CODES (BLINKING) (0-38)

WARNING (1,2,5)

MODBUS BAUD RATE P30.2
0:1200 bps; 1:2400 bps
2:4800 bps; 3:9600 bps
4:19200 bps; 5:38400 bps
6:57600 bps; 7:115200 bps

Modbus response delay P30.04 1-20ms

Bus voltage r27.03 read only