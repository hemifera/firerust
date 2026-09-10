import pandas as pd
import my_rust_parser
from datetime import datetime

input_file = "./data/input/raw-20260823_001733.csv"
fecha_actual = datetime.now().strftime("%Y%m%d_%H%M%S")
output_file = f"./data/output/processed_traces_{fecha_actual}.csv"

df_raw = pd.read_csv(input_file)
df_raw = df_raw.dropna(subset=['hex_data', 'timestamp'])

# Convertimos ambas columnas a listas nativas de Python
timestamps_list = df_raw['timestamp'].astype(str).tolist()
hex_list = df_raw['hex_data'].tolist()

# Rust procesa, filtra errores y devuelve el diccionario estructurado
resultado_dict = my_rust_parser.process_batch_traces(timestamps_list, hex_list)

# Armamos el DataFrame (ahora ya incluye la llave 'timestamp')
df_procesado = pd.DataFrame(resultado_dict)

# Convertimos los strings de timestamp a datetime de Pandas y calculamos delta_t
df_procesado['timestamp'] = pd.to_datetime(df_procesado['timestamp'])
df_procesado['delta_t'] = df_procesado['timestamp'].diff().dt.total_seconds().fillna(0.0)

# Opcional: mover delta_t al inicio junto al timestamp
cols = ['timestamp', 'delta_t'] + [c for c in df_procesado.columns if c not in ['timestamp', 'delta_t']]
df_procesado = df_procesado[cols]

df_procesado.to_csv(output_file, index=False)
print(f"\nArchivo generado: {output_file} con {len(df_procesado)} registros.")
