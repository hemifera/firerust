# Proposito

Implementar una libreria procesadora de trazas modbus para entrenamiento supervisado y no supervisado, que sera utilizada en una fpga PYNQ Z1 para la implementacion de un firewall inteligente de trazas modbus.

El proyecto consiste de multiples directorios, donde cada uno cumple un rol diferente.
Actualmente, firebrust/ posee las funciones necesarias para procesar las tramas, escrita en Rust.

Posteriormente se llamaran los archivos necesarios en un proyecto de python que usar jupyter notebooks,
debido a la necesidad del funcionamiento de la PYNQ

Estado:
- Actualizacion de firebrust a version 0.2.0
- Libreria modbus de rust esta finalizada. (cuestionable)
- Librerias llamables por python: se ha utilizado uv y maturin para gestionar los paquetes y se han implementado dos, una para una sola traza y otra para un arreglo de multiples trazas.
