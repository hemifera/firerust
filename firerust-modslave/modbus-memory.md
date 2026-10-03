# Screen dashboard status

MODO: SLAVE (TEXTO A MOSTRAR)
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