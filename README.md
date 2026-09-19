# Proposito

Implementar una libreria procesadora de trazas modbus para entrenamiento supervisado y no supervisado, que sera utilizada en una fpga PYNQ Z1 para la implementacion de un firewall inteligente de trazas modbus.

El proyecto consiste de multiples directorios, donde cada uno cumple un rol diferente.
Actualmente, firebrust/ posee las funciones necesarias para procesar las tramas, escrita en Rust.

Posteriormente se llamaran los archivos necesarios en un proyecto de python que usar jupyter notebooks,
debido a la necesidad del funcionamiento de la PYNQ

## Estado

- Actualizacion de firebrust a version 0.3.0
- Libreria modbus de rust esta finalizada. (cuestionable)
- Librerias llamables por python: se ha utilizado uv y maturin para gestionar los paquetes y se han implementado dos, una para una sola traza y otra para un arreglo de multiples trazas.

## Jupyter

1. Instalar un ambiente de entorno de python, idealmente usando uv. En la carpeta raiz, firerust/ ejecutar ``uv venv``, luego .``venv/Scrips/activate`` y finalmente ``uv pip install -r requirements.txt``. Deberia instalar las librerias, maturin, pandas, wheels, jupyter y dependencias.
2. Para habilitar jupyter lab, ejecuta ``uv run --with jupyter jupyter lab``

## Exportando libreria a python y PYNQ Z1

Para exportar la libreria se debe instalar los paquetes necesarios de python, para ello se recomienda usar uv y haber realizado el paso 1 de Jupyter

1. En Windows, ejecutar ``rustup target add armv7-unknown-linux-gnueabihf``, esto permitira generar los binarios para la arquitectura arm de la PYNQ. Prerequisito: tener Rust instalado.
2. Ubicarse en firerust/firebrust/ y ejecutar ``maturin build --release --target armv7-unknown-linux-gnueabihf --zig -i python``, esto generara un archivo en *firebrust/target/wheels/firebrust-0.x.0-XXXXXXXXXX.whl*
3. Usando el archivo .whl, subirlo como en la carpeta de jupyter notebook.

```python
# Reemplazar con el nombre del archivo generado
!pip install firebrust-0.2.0-cp38-abi3-manylinux_2_17_armv7l.manylinux2014_armv7l.whl

import modbus_parser
```

## Más información

Debido a la complejidad del proyecto, se estará documentando más informacion en la carpeta docs.

