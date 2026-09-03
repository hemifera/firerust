struct RawTraces {
    pub traces: Vec<u8>,
}

struct ProcessedTraces {
    timestamp: u64,
    slave_address: u8,
    function_code: u8,
    function_name: String,

    // Unit 1
    address_unit_1: u16,
    quantity_unit_1: u16,
    count_unit_1: Option<i16>,

    // Register units
    mininum_value_register: Option<i64>,
    maximum_value_register: Option<i64>,
    median_value_register: Option<i64>,
    total_value_register: Option<i64>,
    zeros_count_register: Option<i64>,

    // Unit 2
    address_unit_2: Option<i16>,
    quantity_unit_2: Option<i16>,

    // extras
    crc_calculated: u16,
}

impl RawTraces {
    fn calculate_register_units(vec: &Vec<u16>) -> (u64, u64, u64, u64, u64) {
        // Si el vector está vacío, retornamos tupla de ceros para evitar errores (panics)
        if vec.is_empty() {
            return (0, 0, 0, 0, 0);
        }

        // 1. Clonar y ordenar para obtener el mínimo, máximo y la mediana
        let mut sorted = vec.clone();
        sorted.sort_unstable();

        let min = sorted[0] as u64;
        let max = *sorted.last().unwrap() as u64;

        // 2. Calcular la mediana (manejando cantidad de elementos pares e impares)
        let len = sorted.len();
        let median = if len.is_multiple_of(2) {
            let mid1 = sorted[len / 2 - 1] as u64;
            let mid2 = sorted[len / 2] as u64;
            (mid1 + mid2) / 2
        } else {
            sorted[len / 2] as u64
        };

        // 3. Calcular la suma total y la cantidad de ceros en binario
        let mut sum: u64 = 0;
        let mut zeros_count: u64 = 0;

        for &num in vec {
            sum += num as u64;
            // count_zeros() cuenta los bits en '0' de los 16 bits que componen al u16
            zeros_count += num.count_zeros() as u64;
        }

        (min, max, median, sum, zeros_count)
    }

    // Devuelve una tupla con (validez, dirección del esclavo,
    // codigo de funcion y crc calculado) si la traza es válida,
    // o (false, 0, 0) si no lo es
    fn trace_validation(&self) -> (bool, u8, u8, u16) {
        // Validar que el vector de trazas no esté vacío
        if self.traces.is_empty() {
            return (false, 0, 0, 0);
        }

        // Validar que la longitud del vector sea al menos 5 bytes (1 byte de dirección + 1 byte de función + 2 bytes de CRC)
        if self.traces.len() < 5 {
            return (false, 0, 0, 0);
        }

        // Validar que el primer byte (dirección del esclavo) esté en el rango válido (1-247)
        let slave_address = self.traces[0];
        if !(1..=247).contains(&slave_address) {
            return (false, 0, 0, 0);
        }

        // Validar que el segundo byte (código de función) esté en el rango válido (1-24)
        let function_code = self.traces[1];
        if !(1..=24).contains(&function_code) {
            return (false, 0, 0, 0);
        }

        // Validar que los últimos dos bytes sean un CRC válido
        let crc_received = ((self.traces[self.traces.len() - 2] as u16) << 8)
            | (self.traces[self.traces.len() - 1] as u16);
        let crc_calculated = Self::calculate_crc(&self.traces[..self.traces.len() - 2]);
        if crc_received != crc_calculated {
            return (false, 0, 0, 0);
        }

        (true, slave_address, function_code, crc_calculated)
    }

    fn calculate_crc(data: &[u8]) -> u16 {
        let mut crc: u16 = 0xFFFF;

        for &byte in data {
            crc ^= byte as u16;

            for _ in 0..8 {
                if (crc & 0x0001) != 0 {
                    crc >>= 1;
                    crc ^= 0xA001;
                } else {
                    crc >>= 1;
                }
            }
        }

        crc
    }

    pub fn get_modbus_function_name(function_code: u8) -> &'static str {
        match function_code {
            1 => "Read Coils",
            2 => "Read Discrete Inputs",
            3 => "Read Holding Registers",
            4 => "Read Input Registers",
            5 => "Write Single Coil",
            6 => "Write Single Register",
            7 => "Read Exception Status",
            8 => "Diagnostics",
            11 => "Get Comm Event Counter",
            12 => "Get Comm Event Log",
            15 => "Write Multiple Coils",
            16 => "Write Multiple Registers",
            17 => "Report Server ID",
            20 => "Read File Record",
            21 => "Write File Record",
            22 => "Mask Write Register",
            23 => "Read Write Multiple Registers",
            24 => "Read FIFO Queue",
            // Aunque validaste el rango antes, Rust exige que el match cubra
            // todos los valores posibles de un u8 (0 a 255)
            _ => "Unknown Function",
        }
    }

    fn process(&self) -> Option<ProcessedTraces> {
        // Valida que la traza modbus es apropiada y retorna una tupla con (validez, dirección del esclavo, código de función y crc calculado)
        let validation_result: (bool, u8, u8, u16) = self.trace_validation();
        if !validation_result.0 {
            return None;
        }

        let trace_len: usize = self.traces.len();
        // address_unit_1 une self.traces[2] y self.traces[3] como un solo u16
        let address_unit_1 = ((self.traces[2] as u16) << 8) | (self.traces[3] as u16);
        let quantity_unit_1 = ((self.traces[4] as u16) << 8) | (self.traces[5] as u16);
        let count_unit_1: i16 = match validation_result.2 {
            15 | 16 => self.traces[6] as i16,
            _ => -1,
        };

        let mut mininum_value_register: i64 = -1;
        let mut maximum_value_register: i64 = -1;
        let mut median_value_register: i64 = -1;
        let mut total_value_register: i64 = -1;
        let mut zeros_count_register: i64 = -1;

        let mut address_unit_2: i16 = -1;
        let mut quantity_unit_2: i16 = -1;

        match validation_result.2 {
            6 => {
                let reg_units = ((self.traces[4] as u16) << 8) | (self.traces[5] as u16);
                let (min, max, median, total, zeros_count) =
                    Self::calculate_register_units(&vec![reg_units]);

                mininum_value_register = min as i64;
                maximum_value_register = max as i64;
                median_value_register = median as i64;
                total_value_register = total as i64;
                zeros_count_register = zeros_count as i64;
            }

            // Write multiple coils
            15 | 16 => {
                if self.traces.len() < 9 {
                    return None;
                }

                let byte_count = self.traces[6] as usize;

                // Se asegura que la trama de respuesta tenga la longitud correcta
                // según el byte count
                // // explicacion de longitud:
                // // slave address: 1 byte
                // // function code: 1 byte
                // // address unit 1: 2 bytes
                // // quantity unit 1: 2 bytes
                // // byte count: 1 byte
                // // data: N bytes
                // // crc: 2 bytes
                // // total = 1 + 1 + 2 + 2 + 1 + N + 2 = 9 + N
                if self.traces.len() != 9 + byte_count {
                    return None;
                }

                let quantity_outputs: u16 =
                    ((self.traces[4] as u16) << 8) | (self.traces[5] as u16);
                // N = Quantity of Outputs / 8, if the remainder is different of 0  N = N+1
                let q = quantity_outputs as usize; // Hacemos el cast una sola vez

                let n: usize = match validation_result.2 {
                    15 => q.div_ceil(8),
                    16 => q * 2,
                    _ => 0,
                };

                // !TODO ACTUALIZAR SEGUN N
                // reg_vector_units es un vector de u16 que contendrá los valores de los registros
                // empieza desde self.traces[7] hasta self.traces[7 + byte_count], y lo toma en pares de u8
                // es decir si tiene byte count 3, los pares irian como (self.traces[7],self.traces[8]), (self.traces[9],self.traces[10]), (self.traces[11],self.traces[12])
                let reg_vector_units = self.traces[7..7 + byte_count]
                    .chunks(2)
                    .filter_map(|chunk| {
                        if chunk.len() == 2 {
                            Some(((chunk[0] as u16) << 8) | (chunk[1] as u16))
                        } else {
                            None
                        }
                    })
                    .collect();
                let (min, max, median, total, zeros_count) =
                    Self::calculate_register_units(&reg_vector_units);

                mininum_value_register = min as i64;
                maximum_value_register = max as i64;
                median_value_register = median as i64;
                total_value_register = total as i64;
                zeros_count_register = zeros_count as i64;

                if validation_result.2 == 16 {
                    address_unit_2 = ((self.traces[7 + byte_count] as i16) << 8)
                        | (self.traces[8 + byte_count] as i16);
                    quantity_unit_2 = ((self.traces[9 + byte_count] as i16) << 8)
                        | (self.traces[10 + byte_count] as i16);
                }
            }
            23 => {
                if self.traces.len() < 13 {
                    return None;
                }

                let byte_count = self.traces[10] as usize;
                // slave address 1 byte
                // Function code 1byte -> function code
                // Read starting address 2bytes -> address unit 1
                // Quantity to read 2bytes -> quantity unit 1
                // Write starting addres 2 bytes -> address unit 2
                // Quantity to write 2bytes -> quantity unit 2
                // Write Byte count 1byte -> count unit 1
                // Write registers value N * 2 bytes -> vector units
                // crc 2 bytes
                // 1 + 1 + 2 + 2 +2 + 2 +1 + N*2 + 2 = 13 + N*2
                if self.traces.len() != 13 + byte_count {
                    return None;
                }
            }
            _ => {}
        }

        Some(ProcessedTraces {
            timestamp: 0,
            slave_address: validation_result.1,
            function_code: validation_result.2,
            function_name: Self::get_modbus_function_name(validation_result.2).into(),
            address_unit_1,
            quantity_unit_1,
            count_unit_1: Some(count_unit_1),
            mininum_value_register: Some(mininum_value_register),
            maximum_value_register: Some(maximum_value_register),
            median_value_register: Some(median_value_register),
            total_value_register: Some(total_value_register),
            zeros_count_register: Some(zeros_count_register),
            address_unit_2: Some(address_unit_2),
            quantity_unit_2: Some(quantity_unit_2),
            crc_calculated: validation_result.3,
        })
    }
}
