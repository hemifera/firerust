use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use rayon::prelude::*;

const MAX_MODBUS_BYTES: usize = 256; // max bytes in a modbus frame
const MINIMUM_MODBUS_BYTES: usize = 4; // min bytes in a modbus frame
const COMMON_MODBUS_BYTES_LENGTH: usize = 8;

const MAX_WRITE_MULTIPLE_COILS_BYTES: u8 = 246; // coils bytes
const MAX_WRITE_MULTIPLE_REGISTERS: u8 = 125; // registers
const READ_WRITE_MULTIPLE_REGISTERS_MIN_BYTES: u8 = 13; // min instrucion registers
const READ_WRITE_MULTIPLE_REGISTERS_MAX_READ: u8 = 125; // max read registers
const READ_WRITE_MULTIPLE_REGISTERS_MAX_WRITE: u8 = 121; // max write registers

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RawTraces {
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
pub struct ProcessedTraces {
    // timestamp: u64,
    pub slave_address: u8,
    pub function_code: u8,
    pub function_name: String,

    // Unit 1
    pub address_unit_1: Option<i32>,
    pub quantity_unit_1: Option<i32>,
    pub count_unit_1: Option<i32>,

    // Register units
    pub register_units: RegisterUnits,

    // Unit 2
    pub address_unit_2: Option<i32>,
    pub quantity_unit_2: Option<i32>,

    // extras
    pub crc_calculated: u16,
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
pub struct RegisterUnits {
    pub mininum_value_register: Option<i64>,
    pub maximum_value_register: Option<i64>,
    pub mean_value_register: Option<i64>,
    // Trimmed mean
    pub tmean_value_register: Option<i64>,
    pub median_value_register: Option<i64>,
    pub std_value_register: Option<i64>,
    pub total_value_register: Option<i64>,
    pub zeros_count_register: Option<i64>,

    // Quartiles
    pub q1_value_register: Option<i64>,
    pub q3_value_register: Option<i64>,
    pub iqr_value_register: Option<i64>,
    // median absolute deviation
    pub mad_value_register: Option<i64>,
    // shanon entropy
    pub entropy_value_register: Option<i64>,

    pub mode_value_register: Option<i64>,
    pub mode_freq_value_register: Option<i64>,

    pub skewness_value_register: Option<i64>,
    pub kurtosis_value_register: Option<i64>,

    // pub normalized_histogram_value_register: Option<i64>,

    // Absolute Diferences | x_i+1 - x_i ... |
    pub max_diff_value_register: Option<i64>,
    pub avg_diff_value_register: Option<i64>,
    pub median_diff_value_register: Option<i64>,
    pub mad_diff_value_register: Option<i64>,
}

impl Default for RegisterUnits {
    fn default() -> Self {
        Self {
            mininum_value_register: Some(-1),
            maximum_value_register: Some(-1),
            mean_value_register: Some(-1),
            tmean_value_register: Some(-1),
            median_value_register: Some(-1),
            std_value_register: Some(-1),
            total_value_register: Some(-1),
            zeros_count_register: Some(-1),

            q1_value_register: Some(-1),
            q3_value_register: Some(-1),
            iqr_value_register: Some(-1),
            mad_value_register: Some(-1),
            entropy_value_register: Some(-1),

            mode_value_register: Some(-1),
            mode_freq_value_register: Some(-1),

            skewness_value_register: Some(-1),
            kurtosis_value_register: Some(-1),

            // normalized_histogram_value_register: Some(-1),
            max_diff_value_register: Some(-1),
            avg_diff_value_register: Some(-1),
            median_diff_value_register: Some(-1),
            mad_diff_value_register: Some(-1),
        }
    }
}

// RawTraces tiene la funcion process, que la prepara para un ProcessedTraces
impl RawTraces {
    // Devuelve una traza modbus preparada para entranamiento
    // Se tiene tres estados, un None, y cuando es Some, se pueden tener datos con -1
    // indicando que no son relevantes
    pub fn process(&self) -> Option<ProcessedTraces> {
        // Valida que la traza modbus es apropiada y retorna una tupla con (validez, dirección del esclavo,
        // código de función y crc calculado)
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
                        bytes_to_u16(&self.traces[2..4]).map(|num: u16| num as i32);

                    // Asigna quantity unit para los bit 4 y 5 excepto cuando el codigo de funcion es 6
                    if !validation_result.function_code.eq(&6) {
                        initial_trace.quantity_unit_1 =
                            bytes_to_u16(&self.traces[4..6]).map(|num: u16| num as i32);
                    }

                    match initial_trace.function_code {
                        6 => {
                            if !traces_length.eq(&8) {
                                return None;
                            }

                            let quantity_unit = bytes_to_u16(&self.traces[4..6])?;

                            initial_trace.quantity_unit_1 = Some(quantity_unit as i32);
                        }
                        15 => {
                            if !(10..=MAX_MODBUS_BYTES).contains(traces_length) {
                                return None;
                            }

                            // MAX_WRITE_MULTIPLE_COILS_BYTES - 9
                            let byte_count = self.traces[6] as i32;

                            // Se asegure que el byte count cumpla con
                            // la cantidad de bytes por el tipo de instruccion
                            // 7: bytes till byte count byte
                            // N data bytes
                            // 2 byte crc
                            if *traces_length != 7 + (byte_count as usize) + 2 {
                                return None;
                            }

                            initial_trace.count_unit_1 = Some(byte_count);

                            let payload_u8 = &self.traces[7..traces_length - 2];
                            // MAX_WRITE_MULTIPLE_COILS_BYTES - 9, se refiere a los bytes de
                            // informacion menos el maximo de bytes de coils
                            let mut static_buff =
                                [0_u16; MAX_WRITE_MULTIPLE_COILS_BYTES as usize - 9];

                            for (i, &byte) in payload_u8.iter().enumerate() {
                                static_buff[i] = byte as u16;
                            }

                            let slice_util = &mut static_buff[..payload_u8.len()];
                            // let converted_regs = vec_to_u16(&self.traces[7..traces_length-2])?;
                            let reg_unit = calculate_register_units(slice_util);

                            initial_trace.register_units = reg_unit;
                        }
                        16 => {
                            if !(10..=MAX_MODBUS_BYTES).contains(traces_length) {
                                return None;
                            }

                            // Quantity of registers
                            let registers_count =
                                bytes_to_u16(&self.traces[4..6]).map(|num: u16| num as i32)?;
                            // Byte Couunt
                            let byte_count = self.traces[6] as i32;

                            // Validacion de cantidad de registros con cantidad ee bytes
                            if 2 * registers_count != byte_count {
                                return None;
                            }

                            // Comprobacion de traza total
                            if *traces_length != 7 + (byte_count as usize) + 2 {
                                return None;
                            }

                            initial_trace.count_unit_1 = Some(registers_count);

                            if !self.traces[7..traces_length - 2].len().is_multiple_of(2) {
                                return None;
                            }

                            let payload = &self.traces[7..*traces_length - 2];
                            let reg_len = payload.len() / 2;

                            let mut static_buff = [0_u16; MAX_WRITE_MULTIPLE_REGISTERS as usize];

                            if reg_len > static_buff.len() {
                                return None;
                            }

                            for (i, par_bytes) in payload.as_chunks::<2>().0.iter().enumerate() {
                                if let Some(valor) = bytes_to_u16(par_bytes) {
                                    static_buff[i] = valor;
                                }
                            }

                            let slice_util = &mut static_buff[..reg_len];
                            let reg_units = calculate_register_units(slice_util);

                            initial_trace.register_units = reg_units;
                            initial_trace.count_unit_1 = Some(byte_count);
                            initial_trace.quantity_unit_1 = Some(registers_count);
                        }
                        23 => {
                            if !(READ_WRITE_MULTIPLE_REGISTERS_MIN_BYTES as usize
                                ..=MAX_MODBUS_BYTES)
                                .contains(traces_length)
                            {
                                return None;
                            }

                            if initial_trace.quantity_unit_1?
                                > (READ_WRITE_MULTIPLE_REGISTERS_MAX_READ as i32)
                            {
                                return None;
                            }

                            initial_trace.address_unit_2 =
                                bytes_to_u16(&self.traces[6..8]).map(|num: u16| num as i32);

                            // cantidad de registros a escribir
                            let write_quantity =
                                bytes_to_u16(&self.traces[8..10]).map(|num: u16| num as i32)?;

                            let write_registers_count = self.traces[10] as i32;

                            // Modbus specification: The byte count specifies the number of bytes to follow in the write data field.
                            if 2 * write_quantity != write_registers_count {
                                return None;
                            }

                            // Se asegura que la longitud de la traza coincida con la cantidad minima de la traza
                            // y la cantidad de registros indicados en Write Byte Count
                            if *traces_length != 11 + (write_registers_count as usize) + 2 {
                                return None;
                            }

                            // Se asegura que todos los registros pertenecientes a Write Registers Value sean pares
                            if !self.traces[11..traces_length - 2].len().is_multiple_of(2) {
                                return None;
                            }

                            // Se toma en todas las trazas pertenecientes a Write Registers Value,
                            // se valida que la longitud de estos no se exceda y se concatenan en pares para formar u16

                            let payload = &self.traces[11..*traces_length - 2];
                            let reg_len = payload.len() / 2;

                            let mut static_buff =
                                [0_u16; READ_WRITE_MULTIPLE_REGISTERS_MAX_WRITE as usize];

                            if reg_len > static_buff.len() {
                                return None;
                            }

                            for (i, par_bytes) in payload.as_chunks::<2>().0.iter().enumerate() {
                                if let Some(valor) = bytes_to_u16(par_bytes) {
                                    static_buff[i] = valor;
                                }
                            }

                            let slice_util = &mut static_buff[..reg_len];
                            // se alimentan los u16 para obtener el register units
                            let reg_units = calculate_register_units(slice_util);
                            initial_trace.register_units = reg_units;

                            // Se asignan los proximos campos
                            initial_trace.count_unit_1 = Some(write_registers_count);
                            initial_trace.quantity_unit_2 = Some(write_quantity);
                        }
                        _ => {}
                    }
                }
            }

            7 | 11 | 12 => {
                // Utilizan la longitud de trazas minima
                // si es diferente hay informacion de mas y no es valida
                if !traces_length.eq(&MINIMUM_MODBUS_BYTES) {
                    return None;
                }
            }

            // Funciones a no filtrar
            // No importa la longitud, son ignoradas
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
    // Intenta convertir el slice de longitud variable en un arreglo fijo [u8; 2]
    let arreglo_fijo: [u8; 2] = bytes.try_into().ok()?;

    // from_be_bytes hace el bit-shift (Big Endian) de forma ultra optimizada
    Some(u16::from_be_bytes(arreglo_fijo))
}

pub fn calculate_crc(data: &[u8]) -> u16 {
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

pub fn calculate_register_units(vec: &mut [u16]) -> RegisterUnits {
    if vec.is_empty() {
        return RegisterUnits::default();
    }

    let (max_diff, avg_diff, med_diff, mad_diff) = calculate_diff_stats(vec);

    vec.sort_unstable();

    let len = vec.len();
    let min = vec[0] as i64;
    let max = vec[len - 1] as i64;

    let mut sum: i64 = 0;
    let mut zeros: i64 = 0;
    for &val in vec.iter() {
        sum += val as i64;
        if val == 0 {
            zeros += 1;
        }
    }

    let mean = sum / len as i64;
    let median = vec[len / 2] as i64;
    let q1 = vec[len / 4] as i64;
    let q3 = vec[(len * 3) / 4] as i64;
    let iqr = q3 - q1;

    let (mode, mode_freq) = calculate_mode(vec);
    let mad = calculate_mad(vec, median);
    let tmean = calculate_trimmed_mean(vec, len);
    let (std, skewness, kurtosis) = calculate_moments(vec, mean as f64);
    let entropy = calculate_entropy(vec);

    RegisterUnits {
        mininum_value_register: Some(min),
        maximum_value_register: Some(max),
        mean_value_register: Some(mean),
        tmean_value_register: Some(tmean),
        median_value_register: Some(median),
        std_value_register: Some(std),
        total_value_register: Some(sum),
        zeros_count_register: Some(zeros),
        q1_value_register: Some(q1),
        q3_value_register: Some(q3),
        iqr_value_register: Some(iqr),
        mad_value_register: Some(mad),
        entropy_value_register: Some(entropy),
        mode_value_register: Some(mode),
        mode_freq_value_register: Some(mode_freq),
        skewness_value_register: Some(skewness),
        kurtosis_value_register: Some(kurtosis),
        // normalized_histogram_value_register: Some(0), // Requiere definir lógica de bins
        max_diff_value_register: max_diff,
        avg_diff_value_register: avg_diff,
        median_diff_value_register: med_diff,
        mad_diff_value_register: mad_diff,
    }
}

fn calculate_diff_stats(vec: &[u16]) -> (Option<i64>, Option<i64>, Option<i64>, Option<i64>) {
    if vec.len() < 2 {
        return (None, None, None, None);
    }

    let mut diffs: Vec<i64> = vec
        .windows(2)
        .map(|w| (w[1] as i64 - w[0] as i64).abs())
        .collect();

    let sum: i64 = diffs.iter().sum();
    let avg = sum / diffs.len() as i64;
    let max = *diffs.iter().max().unwrap_or(&0);

    diffs.sort_unstable();
    let median = diffs[diffs.len() / 2];

    let mut mad_diffs: Vec<i64> = diffs.iter().map(|&x| (x - median).abs()).collect();
    mad_diffs.sort_unstable();
    let mad = mad_diffs[mad_diffs.len() / 2];

    (Some(max), Some(avg), Some(median), Some(mad))
}

fn calculate_mode(vec: &[u16]) -> (i64, i64) {
    let mut max_count = 0;
    let mut mode = vec[0];
    let mut current_count = 1;

    for i in 1..vec.len() {
        if vec[i] == vec[i - 1] {
            current_count += 1;
        } else {
            if current_count > max_count {
                max_count = current_count;
                mode = vec[i - 1];
            }
            current_count = 1;
        }
    }
    if current_count > max_count {
        mode = vec[vec.len() - 1];
        max_count = current_count;
    }

    (mode as i64, max_count as i64)
}

fn calculate_mad(vec: &[u16], median: i64) -> i64 {
    let mut deviations: Vec<i64> = vec.iter().map(|&x| (x as i64 - median).abs()).collect();
    deviations.sort_unstable();
    deviations[deviations.len() / 2]
}

fn calculate_trimmed_mean(vec: &[u16], len: usize) -> i64 {
    let trim_count = len / 10; // 10% trim en cada extremo
    if trim_count * 2 >= len {
        return vec[len / 2] as i64;
    }

    let trimmed = &vec[trim_count..(len - trim_count)];
    let sum: i64 = trimmed.iter().map(|&x| x as i64).sum();
    sum / trimmed.len() as i64
}

fn calculate_moments(vec: &[u16], mean: f64) -> (i64, i64, i64) {
    let len = vec.len() as f64;
    let mut var_sum = 0.0;
    let mut skew_sum = 0.0;
    let mut kurt_sum = 0.0;

    for &val in vec {
        let diff = val as f64 - mean;
        var_sum += diff.powi(2);
        skew_sum += diff.powi(3);
        kurt_sum += diff.powi(4);
    }

    let variance = var_sum / len;
    let std = variance.sqrt();

    let skewness = if std > 0.0 {
        skew_sum / (len * std.powi(3))
    } else {
        0.0
    };
    let kurtosis = if std > 0.0 {
        kurt_sum / (len * std.powi(4))
    } else {
        0.0
    };

    (std as i64, skewness as i64, kurtosis as i64)
}

fn calculate_entropy(vec: &[u16]) -> i64 {
    let len = vec.len() as f64;
    let mut entropy = 0.0;

    let mut current_count = 1;
    for i in 1..vec.len() {
        if vec[i] == vec[i - 1] {
            current_count += 1;
        } else {
            let p = current_count as f64 / len;
            entropy -= p * p.log2();
            current_count = 1;
        }
    }
    let p = current_count as f64 / len;
    entropy -= p * p.log2();

    (entropy * 1000.0) as i64 // Escalado para conservar decimales en i64
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
    let crc_received = u16::from_le_bytes([vec[&vec.len() - 2], vec[&vec.len() - 1]]);
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

#[pyfunction]
#[pyo3(name = "process_single_trace")]
pub fn process_trace_for_python<'py>(
    py: Python<'py>,
    raw_bytes: &[u8],
) -> PyResult<Option<Bound<'py, PyDict>>> {
    let raw = RawTraces {
        traces: raw_bytes.to_vec(),
    };

    match raw.process() {
        Some(processed_trace) => {
            let dict = PyDict::new(py);

            dict.set_item("slave_address", processed_trace.slave_address)?;
            dict.set_item("function_code", processed_trace.function_code)?;
            dict.set_item("function_name", processed_trace.function_name)?;

            dict.set_item("address_unit_1", processed_trace.address_unit_1)?;
            dict.set_item("quantity_unit_1", processed_trace.quantity_unit_1)?;
            dict.set_item("count_unit_1", processed_trace.count_unit_1)?;

            dict.set_item(
                "mininum_value_register",
                processed_trace.register_units.mininum_value_register,
            )?;
            dict.set_item(
                "maximum_value_register",
                processed_trace.register_units.maximum_value_register,
            )?;
            dict.set_item(
                "mean_value_register",
                processed_trace.register_units.mean_value_register,
            )?;
            dict.set_item(
                "tmean_value_register",
                processed_trace.register_units.tmean_value_register,
            )?;
            dict.set_item(
                "median_value_register",
                processed_trace.register_units.median_value_register,
            )?;
            dict.set_item(
                "std_value_register",
                processed_trace.register_units.std_value_register,
            )?;
            dict.set_item(
                "total_value_register",
                processed_trace.register_units.total_value_register,
            )?;
            dict.set_item(
                "zeros_count_register",
                processed_trace.register_units.zeros_count_register,
            )?;
            dict.set_item(
                "q1_value_register",
                processed_trace.register_units.q1_value_register,
            )?;
            dict.set_item(
                "q3_value_register",
                processed_trace.register_units.q3_value_register,
            )?;
            dict.set_item(
                "iqr_value_register",
                processed_trace.register_units.iqr_value_register,
            )?;
            dict.set_item(
                "mad_value_register",
                processed_trace.register_units.mad_value_register,
            )?;
            dict.set_item(
                "entropy_value_register",
                processed_trace.register_units.entropy_value_register,
            )?;

            dict.set_item(
                "mode_value_register",
                processed_trace.register_units.mode_value_register,
            )?;
            dict.set_item(
                "mode_freq_value_register",
                processed_trace.register_units.mode_freq_value_register,
            )?;

            dict.set_item(
                "skewness_value_register",
                processed_trace.register_units.skewness_value_register,
            )?;
            dict.set_item(
                "kurtosis_value_register",
                processed_trace.register_units.kurtosis_value_register,
            )?;
            dict.set_item(
                "max_diff_value_register",
                processed_trace.register_units.max_diff_value_register,
            )?;

            dict.set_item(
                "avg_diff_value_register",
                processed_trace.register_units.avg_diff_value_register,
            )?;
            dict.set_item(
                "median_diff_value_register",
                processed_trace.register_units.median_diff_value_register,
            )?;

            dict.set_item(
                "mad_diff_value_register",
                processed_trace.register_units.mad_diff_value_register,
            )?;

            dict.set_item("address_unit_2", processed_trace.address_unit_2)?;
            dict.set_item("quantity_unit_2", processed_trace.quantity_unit_2)?;

            dict.set_item("crc_calculated", processed_trace.crc_calculated)?;

            Ok(Some(dict))
        }
        None => Ok(None),
    }
}

#[pymodule]
fn modbus_parser(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(process_trace_for_python, m)?)?;
    m.add_function(wrap_pyfunction!(process_batch_traces, m)?)?;
    Ok(())
}

#[pyfunction]
#[pyo3(name = "process_batch_traces")]
pub fn process_batch_traces<'py>(
    py: Python<'py>,
    timestamps: Vec<String>,
    hex_strings: Vec<String>,
) -> PyResult<Bound<'py, PyDict>> {
    let processed_results: Vec<Option<(String, ProcessedTraces)>> = timestamps
        .into_par_iter()
        .zip(hex_strings.into_par_iter())
        .map(|(ts, hex_str)| {
            let cleaned: String = hex_str.chars().filter(|c| !c.is_whitespace()).collect();
            match hex::decode(cleaned) {
                Ok(bytes) => {
                    let raw = RawTraces { traces: bytes };
                    raw.process().map(|pt| (ts, pt))
                }
                Err(_) => None,
            }
        })
        .collect();

    let dict = PyDict::new(py);
    let timestamp_out = PyList::empty(py);
    let slave_address = PyList::empty(py);
    let function_code = PyList::empty(py);
    let function_name = PyList::empty(py);
    let address_unit_1 = PyList::empty(py);
    let quantity_unit_1 = PyList::empty(py);
    let count_unit_1 = PyList::empty(py);
    let mininum_value_register = PyList::empty(py);
    let maximum_value_register = PyList::empty(py);

    let mean_value_register = PyList::empty(py);
    let tmean_value_register = PyList::empty(py);

    let median_value_register = PyList::empty(py);
    let std_value_register = PyList::empty(py);

    let total_value_register = PyList::empty(py);
    let zeros_count_register = PyList::empty(py);

    let q1_value_register = PyList::empty(py);
    let q3_value_register = PyList::empty(py);
    let iqr_value_register = PyList::empty(py);
    let mad_value_register = PyList::empty(py);
    let entropy_value_register = PyList::empty(py);

    let mode_value_register = PyList::empty(py);
    let mode_freq_value_register = PyList::empty(py);

    let skewness_value_register = PyList::empty(py);
    let kurtosis_value_register = PyList::empty(py);

    let max_diff_value_register = PyList::empty(py);
    let avg_diff_value_register = PyList::empty(py);
    let median_diff_value_register = PyList::empty(py);
    let mad_diff_value_register = PyList::empty(py);

    let address_unit_2 = PyList::empty(py);
    let quantity_unit_2 = PyList::empty(py);
    let crc_calculated = PyList::empty(py);

    for result in processed_results.into_iter().flatten() {
        let (ts, trace) = result;

        timestamp_out.append(ts)?;
        slave_address.append(trace.slave_address)?;
        function_code.append(trace.function_code)?;
        function_name.append(trace.function_name)?;

        address_unit_1.append(trace.address_unit_1)?;
        quantity_unit_1.append(trace.quantity_unit_1)?;
        count_unit_1.append(trace.count_unit_1)?;

        mininum_value_register.append(trace.register_units.mininum_value_register)?;
        maximum_value_register.append(trace.register_units.maximum_value_register)?;
        mean_value_register.append(trace.register_units.mean_value_register)?;
        tmean_value_register.append(trace.register_units.tmean_value_register)?;
        median_value_register.append(trace.register_units.median_value_register)?;

        std_value_register.append(trace.register_units.std_value_register)?;
        // mean_value_register.append(trace.register_units.mean_value_register)?;

        total_value_register.append(trace.register_units.total_value_register)?;
        zeros_count_register.append(trace.register_units.zeros_count_register)?;

        q1_value_register.append(trace.register_units.q1_value_register)?;
        q3_value_register.append(trace.register_units.q3_value_register)?;
        iqr_value_register.append(trace.register_units.iqr_value_register)?;
        mad_value_register.append(trace.register_units.mad_value_register)?;
        entropy_value_register.append(trace.register_units.entropy_value_register)?;

        mode_value_register.append(trace.register_units.mode_value_register)?;
        mode_freq_value_register.append(trace.register_units.mode_freq_value_register)?;

        skewness_value_register.append(trace.register_units.skewness_value_register)?;
        kurtosis_value_register.append(trace.register_units.kurtosis_value_register)?;

        max_diff_value_register.append(trace.register_units.max_diff_value_register)?;
        avg_diff_value_register.append(trace.register_units.avg_diff_value_register)?;
        median_diff_value_register.append(trace.register_units.median_diff_value_register)?;
        mad_diff_value_register.append(trace.register_units.mad_diff_value_register)?;

        address_unit_2.append(trace.address_unit_2)?;
        quantity_unit_2.append(trace.quantity_unit_2)?;
        crc_calculated.append(trace.crc_calculated)?;
    }

    dict.set_item("timestamp", timestamp_out)?;
    dict.set_item("slave_address", slave_address)?;
    dict.set_item("function_code", function_code)?;
    dict.set_item("function_name", function_name)?;
    dict.set_item("address_unit_1", address_unit_1)?;
    dict.set_item("quantity_unit_1", quantity_unit_1)?;
    dict.set_item("count_unit_1", count_unit_1)?;
    dict.set_item("mininum_value_register", mininum_value_register)?;
    dict.set_item("maximum_value_register", maximum_value_register)?;
    dict.set_item("tmean_value_register", tmean_value_register)?;
    dict.set_item("mean_value_register", mean_value_register)?;
    dict.set_item("median_value_register", median_value_register)?;
    dict.set_item("std_value_register", std_value_register)?;

    dict.set_item("q1_value_register", q1_value_register)?;
    dict.set_item("q3_value_register", q3_value_register)?;
    dict.set_item("iqr_value_register", iqr_value_register)?;
    dict.set_item("mad_value_register", mad_value_register)?;
    dict.set_item("entropy_value_register", entropy_value_register)?;

    dict.set_item("mode_value_register", mode_value_register)?;
    dict.set_item("mode_freq_value_register", mode_freq_value_register)?;

    dict.set_item("skewness_value_register", skewness_value_register)?;
    dict.set_item("kurtosis_value_register", kurtosis_value_register)?;

    dict.set_item("max_diff_value_register", max_diff_value_register)?;
    dict.set_item("avg_diff_value_register", avg_diff_value_register)?;
    dict.set_item("median_diff_value_register", median_diff_value_register)?;
    dict.set_item("mad_diff_value_register", mad_diff_value_register)?;

    dict.set_item("total_value_register", total_value_register)?;
    dict.set_item("zeros_count_register", zeros_count_register)?;
    dict.set_item("address_unit_2", address_unit_2)?;
    dict.set_item("quantity_unit_2", quantity_unit_2)?;
    dict.set_item("crc_calculated", crc_calculated)?;

    Ok(dict)
}
