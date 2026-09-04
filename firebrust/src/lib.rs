use chrono::prelude;

const MAX_MODBUS_BYTES: usize = 256; // max bytes in a modbus frame
const MINIMUM_MODBUS_BYTES: usize = 4; // min bytes in a modbus frame
const COMMON_MODBUS_BYTES_LENGTH: usize = 8;

const MAX_WRITE_MULTIPLE_COILS_BYTES: u8 = 246; // coils bytes
const MAX_WRITE_MULTIPLE_REGISTERS: u8 = 125; // registers 
const READ_WRITE_MULTIPLE_REGISTERS_MAX_READ: u8 = 121; // max read registers
const READ_WRITE_MULTIPLE_REGISTERS_MAX_WRITE: u8 = 125; // max write registers

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RawTraces {
    pub traces: Vec<u8>,
}

struct ModbusInstruction {
    pub slave_address: u8,
    pub function_code: u8,
    pub crc: u16,
}

// Option types will either cointain a value or None, which is useful for
// optional fields in the ProcessedTraces struct
// Later and extra function will be processed to transform None types to -1

// Derivations like debug, clone, copy, etc. are useful for testing and debugging
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ProcessedTraces {
    // timestamp: u64,
    slave_address: u8,
    function_code: u8,
    function_name: String,

    // Unit 1
    address_unit_1: Option<i16>,
    quantity_unit_1: Option<i16>,
    count_unit_1: Option<i16>,

    // Register units
    register_units: RegisterUnits,

    // Unit 2
    address_unit_2: Option<i16>,
    quantity_unit_2: Option<i16>,

    // extras
    crc_calculated: u16,
}

impl Default for ProcessedTraces {
    fn default() -> Self {
        Self {
            slave_address: 1,
            function_code: 1,
            function_name: get_modbus_function_name(1).into(),

            address_unit_1: Some(-1),
            quantity_unit_1: Some(-1),
            count_unit_1: Some(-1),

            register_units: RegisterUnits::default(),

            address_unit_2: Some(-1),
            quantity_unit_2: Some(-1),

            crc_calculated: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RegisterUnits {
    mininum_value_register: Option<i64>,
    maximum_value_register: Option<i64>,
    median_value_register: Option<i64>,
    total_value_register: Option<i64>,
    zeros_count_register: Option<i64>,
}

impl Default for RegisterUnits {
    fn default() -> Self {
        Self {
            mininum_value_register: Some(-1),
            maximum_value_register: Some(-1),
            median_value_register: Some(-1),
            total_value_register: Some(-1),
            zeros_count_register: Some(-1),
        }
    }
}

// RawTraces tiene una serie de funciones, y secuencias a cumplir
// Iiciarlmente
impl RawTraces {
    // Devuelve una traza modbus preparada para entranamiento
    // Se tiene tres estados, un None, y cuando es Some, se pueden tener datos con -1
    // indicando que no son relevantes
    fn process(&self) -> Option<ProcessedTraces> {
        // Valida que la traza modbus es apropiada y retorna una tupla con (validez, dirección del esclavo, código de función y crc calculado)
        let validation_result: ModbusInstruction = trace_validation(&self.traces)?;

        let traces_length = &self.traces.len();

        let mut initial_trace: Option<ProcessedTraces> = Some(ProcessedTraces {
            slave_address: validation_result.slave_address,
            function_code: validation_result.function_code,
            function_name: get_modbus_function_name(validation_result.function_code).into(),
            crc_calculated: validation_result.crc,
            ..Default::default()
        });

        match validation_result.function_code {
            1 | 2 | 3 | 4 | 5 | 6 | 15 | 16 | 23 => {
                if !traces_length < COMMON_MODBUS_BYTES_LENGTH {
                    return None;
                }

                if let Some(initial_trace) = initial_trace.as_mut() {
                    // Asigna adress unit para los bits 2 y 3 del self.traces
                    initial_trace.address_unit_1 =
                        bytes_to_u16(&self.traces[2..4]).map(|num: u16| num as i16);

                    // Asigna quantity unit para los bit 4 y 5 excepto cuando el codigo de funcion es 6
                    if !validation_result.function_code.eq(&6) {
                        initial_trace.quantity_unit_1 =
                            bytes_to_u16(&self.traces[4..6]).map(|num: u16| num as i16);
                    }

                    match initial_trace.function_code {
                        6 => {
                            if !traces_length.eq(&8) {
                                return None;
                            }

                            let reg_unit = bytes_to_u16(&self.traces[4..6])?;

                            initial_trace.register_units = RegisterUnits {
                                mininum_value_register: Some(reg_unit as i64),
                                maximum_value_register: Some(reg_unit as i64),
                                median_value_register: Some(reg_unit as i64),
                                total_value_register: Some(reg_unit as i64),
                                zeros_count_register: Some(reg_unit.count_zeros() as i64),
                            }
                        }
                        15 | 16 | 23 => {}
                        _ => {}
                    }
                }
            }

            7 | 11 | 12 => {
                if !traces_length.eq(&MINIMUM_MODBUS_BYTES) {
                    return None;
                }
            }

            // Funciones a no filtrar
            8 | 17 | 20 | 21 | 22 | 24 => {}

            _ => {
                return None;
            }
        }

        initial_trace
    }
}

// Convierte un slice de bytes en un u16, si el slice tiene exactamente 2 bytes.
// Retorna None si no es así.
fn bytes_to_u16(bytes: &[u8]) -> Option<u16> {
    if bytes.len() != 2 {
        return None;
    }
    Some(((bytes[0] as u16) << 8) | (bytes[1] as u16))
}

// Convierte un slice de bytes en un vector de u16, si el slice tiene una longitud par.
// Retorna None si no es así.
fn vec_to_u16(vec: &[u8]) -> Option<Vec<u16>> {
    if !vec.len().is_multiple_of(2) {
        return None;
    }
    Some(vec.chunks(2).filter_map(bytes_to_u16).collect())
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

pub fn calculate_register_units(vec: &mut [u16]) -> (u64, u64, u64, u64, u64) {
    if vec.is_empty() {
        return (0, 0, 0, 0, 0);
    }

    // 1. Calcular min, max, suma y ceros en un solo recorrido (O(N))
    let mut min = vec[0];
    let mut max = vec[0];
    let mut sum: u64 = 0;
    let mut zeros_count: u64 = 0;

    for &num in vec.iter() {
        if num < min {
            min = num;
        }
        if num > max {
            max = num;
        }
        sum += num as u64;
        zeros_count += num.count_zeros() as u64;
    }

    // 2. Calcular la mediana in-place con Quickselect (0 memoria extra)
    let len = vec.len();
    let mid = len / 2;

    let median = if len.is_multiple_of(2) {
        // select_nth_unstable particiona el slice. Nos devuelve el segundo valor central
        // y nos garantiza que todos los elementos en `left_half` son menores o iguales.
        let (_, &mut mid2, left_half) = vec.select_nth_unstable(mid);

        // El primer valor central es simplemente el máximo de la partición izquierda
        let &mid1 = left_half.iter().max().unwrap();

        (mid1 as u64 + mid2 as u64) / 2
    } else {
        // Para longitud impar, solo extraemos el centro directo
        let (_, &mut mid_val, _) = vec.select_nth_unstable(mid);
        mid_val as u64
    };

    (min as u64, max as u64, median, sum, zeros_count)
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

// Devuelve una tupla con (validez, dirección del esclavo,
// codigo de funcion y crc calculado) si la traza es válida,
// o (false, 0, 0) si no lo es
fn trace_validation(vec: &[u8]) -> Option<ModbusInstruction> {
    // Validar que el vector de trazas no esté vacío
    if vec.is_empty() {
        return None;
    }

    // Valida que la cantidad de bytes en la traza esté dentro del rango permitido (4 a 256 bytes)
    // 4 Es la minima, no posee data bytes
    // FC 07 (Read Exception Status) 4 bytes
    // FC 11 (Report Slave ID)
    // FC 12 (Get Comm Event Counter)
    if !(MINIMUM_MODBUS_BYTES..=MAX_MODBUS_BYTES).contains(&vec.len()) {
        return None;
    }

    // Validar que el primer byte (dirección del esclavo) esté en el rango válido (1-247)
    let slave_address = vec[0];
    if !(1..=247).contains(&slave_address) {
        return None;
    }

    // Validar que el segundo byte (código de función) esté en el rango válido (1-24)
    let function_code = vec[1];
    if !(1..=24).contains(&function_code) {
        return None;
    }

    // Validar que los últimos dos bytes sean un CRC válido
    let crc_received = ((vec[vec.len() - 2] as u16) << 8) | (vec[vec.len() - 1] as u16);
    let crc_calculated = calculate_crc(&vec[..vec.len() - 2]);
    if crc_received != crc_calculated {
        return None;
    }

    Some(ModbusInstruction {
        slave_address,
        function_code,
        crc: crc_calculated,
    })
}
