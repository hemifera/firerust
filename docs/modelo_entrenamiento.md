# Modelo de entrenamiento

El proyecto consiste en un firewall con arquitectura bump-in-the-wire para comunicacion Modbus RTU. No debe cambiar la infraestructura existente, sino actuar como guardia de seguridad o portero que regula la entrada de instrucciones de un maestro a esclavo Modbus.

En este avance, se ha desarrollado una estructura de modelado de filtrado de datos, que todavia se encuentra en progreso su desarrollo final.

Las trazas Modbus, se pueden categorizar según el código de función, y el tipo de dato que recibe. En este momento la especificacion del protocol Modbus tiene apartado alrededor de 18 codigos de funciones diferentes, de los cuales, para este proyecto son relevantes 9 de estos: estos leen o cambian registros o bobinas del estado de un equipo que permite actuar como esclavo modbus. Estos corresponden a los codigos de funcion 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0xF, 0x10, 0x17.

Actualmente, cuando la PYNQ recibe trazas Modbus, actua como un tunel que las reenvia al esclavo pero al mismo tiempo va tomando un registro de estas. Primero guarda 1000 trazas en memoria, las guarda en un archivo csv y luego continua almacenando las trazas hasta que se decide dejar de tomar trazas. Este puede ser considerado el **periodo de adquisicion de datos** y la PYNQ en ese momento no hace nada mas que adquirir datos y reenviarlos.

Luego al llegar a la fase de entrenamiento, la PYNQ utiliza su utilidad de FPGA y actua como tunel bidireccional. En este periodo el procesador ARM se desconecta de la interfaz de la FPGA y comienza a realiza su entrenamiento. El entrenamiento tienen diferentes fases y son explicadas en la siguientes secciones.

## Procesado de datos

Los archivos inciales de adquisicion de datos, son archivos .csv que sólo contienen dos columnas, un timestamp (ejemplo 2026-09-13 05:28:13.490) y hex_data (ejemplo 01 10 0B 13 00 03 06 00 C9 00 74 00 F0 AE 70).

Los datos de hex_data pueden tener un tamaño fijo o variable maximo de 256 bytes según el codigo de función. Si son las funciones en el rango de 0x01 hasta 0x06 tienen un tamaño fijo de 8 bytes. Mientras que los codigo de funcion 0x0F, 0x10 y 0x17 comparten tener un tamaño variado de datos.

Para esto, se ha considerado utilizar herramientas estadísticas para que los datos con rango variado tengan una huella única que pueda ser reconocida por un modelo de entrenamiento no supervisado. Los datos que varian de tamaño tiene una peculiaridad compartida, siempre tienen una direccion inicial de memoria, luego una cantidad que le dice cuantos registros contiguos a la direccion inicial deben ser cambiados o leidos. Debido a este comportamiendo, se ha elegido utilizar herramientas estadísticas para encontrar una huella de reconocimiento.

Sabiendo esto, se prepara un dataframe de tamaño fijo que realiza diferentes transformaciones y validaciones. La estructura del DataFrame actualmente sigue cambiando por las dificultades de analisis todavía en desarrollo. Mientras el archivo modbus.xlsx tiene la primera estructura inicial, esta esta por desactualizada, y se puede encontrar las más reciente en firebrust/src/lib.rs en la Struct ProcessedTraces y tambien la Struct RegisterUnits.

La Struct incluye los siguientes campos:

- slave_address
- function_code
- function_name,
- address_unit_1
- quantity_unit_1
- count_unit_1
- register_units: Struct RegisterUnits
- address_unit_2
- quantity_unit_2
- crc_calculated

Se aclara que RegisterUnits, es la Struct que se encuentra cambiando actualmente, y a este momento, cuando se tiene un rango variado de datos se tienen los distintos campos de entrenamiento:

- mininum_value
- maximum_value
- mean_value
- trimmed_mean_value
- median_value
- standard_deviation_value
- total_sum_value
- binary_zeros_count
- first_quartile_value
- third_quartile_value
- interquartile_range_value
- median absolute deviation
- median_absolute_deviation_value
- shanon_entropy
- mode_value
- mode_freq_value
- skewness_value
- kurtosis_value

Una vez se tienen todos estos datos, se procesan usando el lenguaje de programacion Rust, que simplifica considerablemente el tiempo de procesado de datos y los devuelve como la lista de datos fija que puede ser tratada como DataFrame dentrode python.

Una vez con los datos con Python, lo primero que se hace es categorizar las trazas en codigos de funcion y al mismo tiempo se calcula la diferencia de tiempo de cada traza segun la ultima traza enviada; en este caso es la ultima traza Modbus enviada, no la ultima por código de función.

Con los datos separados por codigo de funcion, se hace un analisis de correlacion, y si se encuentra una correlacion alta en los datos, se realiza un entrenamiento no supervisado secuencial por ventanas de trazas Modbus. El numero de trazas que se toman son N contiguas, donde N corresponde al valor de autocorrelacion encontrado que sea alto.

Hasta este punto el avance se alcanzado hasta este punto.
Se han encontrado situaciones donde no existe autocorrelacion entre los datos, y se tiene que tomar diferentes medidas para tratar los datos.
Actualmente se ha estado considerando tambien implementar un sistema automatizado de reglas, tomando inspiracion en los firewall tradicionales.