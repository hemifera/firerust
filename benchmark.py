import pandas as pd
import my_rust_parser
import time

input_file = "./data/input/raw-20260823_001733.csv"
print(f"Cargando datos en memoria desde {input_file}...\n")

df_raw = pd.read_csv(input_file)
df_raw = df_raw.dropna(subset=['hex_data', 'timestamp'])

# ---------------------------------------------------------
# Prueba 1: Secuencial (Bucle For en Python)
# ---------------------------------------------------------
print("Ejecutando método 1 (Secuencial)...")
start_seq = time.perf_counter()

resultados_validos = []
for index, row in df_raw.iterrows():
    timestamp = row['timestamp']
    hex_string = row['hex_data']

    try:
        raw_bytes = bytes.fromhex(hex_string.replace(" ", ""))
        resultado = my_rust_parser.process_single_trace(raw_bytes)
        if resultado is not None:
            resultado['timestamp'] = timestamp
            resultados_validos.append(resultado)
    except Exception:
        pass

if resultados_validos:
    df_seq = pd.DataFrame(resultados_validos)
    df_seq['timestamp'] = pd.to_datetime(df_seq['timestamp'])
    df_seq['delta_t'] = df_seq['timestamp'].diff().dt.total_seconds().fillna(0.0)

end_seq = time.perf_counter()
time_seq = end_seq - start_seq


# ---------------------------------------------------------
# Prueba 2: Paralelo en Batch (Rust + Rayon)
# ---------------------------------------------------------
print("Ejecutando método 2 (Paralelo / Batch)...")
start_par = time.perf_counter()

timestamps_list = df_raw['timestamp'].astype(str).tolist()
hex_list = df_raw['hex_data'].tolist()

resultado_dict = my_rust_parser.process_batch_traces(timestamps_list, hex_list)
df_par = pd.DataFrame(resultado_dict)

if not df_par.empty:
    df_par['timestamp'] = pd.to_datetime(df_par['timestamp'])
    df_par['delta_t'] = df_par['timestamp'].diff().dt.total_seconds().fillna(0.0)

end_par = time.perf_counter()
time_par = end_par - start_par
print(f"Tiempo Secuencial : {time_seq:.4f} segundos")
print(f"Tiempo Paralelo   : {time_par:.4f} segundos")

print(f"Relacion tiempo secuencial / tiempo paralelo: {time_seq / time_par}")
