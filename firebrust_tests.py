import pandas as pd
# Maturin compiled lib
import my_rust_parser
from datetime import datetime

input_file = "./data/input/raw-20260823_001733.csv"
fecha_actual = datetime.now().strftime("%Y%m%d_%H%M%S")
output_file = f"./data/output/processed_traces_{fecha_actual}.csv"

print(f"Intentado leer: {input_file}...")

df_raw = pd.read_csv(input_file)

resultados_validos = []


for index, row in df_raw.iterrows():
    timestamp = row['timestamp']
    hex_string = row['hex_data']

    if pd.isna(hex_string):
        continue

    try:
        raw_bytes = bytes.fromhex(hex_string.replace(" ", ""))
        resultado = my_rust_parser.process_single_trace(raw_bytes)

        if resultado is not None:
            resultado['timestamp'] = timestamp
            resultados_validos.append(resultado)

    except Exception as e:
        pass # Manejo de errores omitido por brevedad

if resultados_validos:
    df_procesado = pd.DataFrame(resultados_validos)

    # 2. Convertir la columna string a un objeto DateTime real de Pandas
    df_procesado['timestamp'] = pd.to_datetime(df_procesado['timestamp'])

    # 3. Calcular delta_t (diferencia de tiempo con la fila anterior en segundos)
    # El primer valor será NaN, lo cual es correcto para Isolation Forest
    # (puedes llenarlo con 0.0 si el modelo lo exige)
    df_procesado['delta_t'] = df_procesado['timestamp'].diff().dt.total_seconds().fillna(0.0)

    # Reordenar columnas para ver timestamp y delta_t al principio
    columnas = ['timestamp', 'delta_t'] + [col for col in df_procesado.columns if col not in ['timestamp', 'delta_t']]
    df_procesado = df_procesado[columnas]

    df_procesado.to_csv(output_file, index=False)
    print(f"\nArchivo generado: {output_file}")
