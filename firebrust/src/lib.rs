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
    count_unit_1: Option<u16>,

    // Register units
    mininum_value_register: Option<u64>,
    maximum_value_register: Option<u64>,
    mediant_value_register: Option<u64>,
    total_value_register: Option<u64>,
    zeros_count_register: Option<u64>,

    // Unit 2
    address_unit_2: Option<u16>,
    quantity_unit_2: Option<u16>,

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

    // Devuelve una tupla con (validez, dirección del esclavo, codigo de funcion y crc calculado) si la traza es válida, o (false, 0, 0) si no lo es.
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
        // None

        let validation_result: (bool, u8, u8, u16) = self.trace_validation();
        if !validation_result.0 {
            return None;
        }

        let trace_len: usize = self.traces.len();
        let address_unit_1 = ((self.traces[2] as u16) << 8) | (self.traces[3] as u16);
        let quantity_unit_1 = ((self.traces[4] as u16) << 8) | (self.traces[5] as u16);

        match validation_result.2 {
            1 | 2 | 3 | 4 | 5 | 6 | 15 | 16 => {
                if trace_len < 7 {
                    return None;
                }
            }
            _ => return None,
        }

        Some(ProcessedTraces {
            timestamp: 0,
            slave_address: validation_result.1,
            function_code: validation_result.2,
            function_name: Self::get_modbus_function_name(validation_result.2).into(),
            address_unit_1: address_unit_1,
            quantity_unit_1: quantity_unit_1,
            count_unit_1: None,
            mininum_value_register: None,
            maximum_value_register: None,
            mediant_value_register: None,
            total_value_register: None,
            zeros_count_register: None,
            address_unit_2: None,
            quantity_unit_2: None,
            crc_calculated: validation_result.3,
        })
    }
}
