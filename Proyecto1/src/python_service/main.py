from fastapi import FastAPI # type: ignore
import os
import pandas as pd # type: ignore
import matplotlib.pyplot as plt # type: ignore
import json
from typing import List
from models.models import LogProcess
from models.models import LogMemory
from datetime import datetime
from matplotlib.dates import DateFormatter # type: ignore
import matplotlib.dates as mdates # type: ignore

app = FastAPI()


@app.get("/")
def read_root():
    return {"Hello": "World mundo "}


@app.post("/logs")
def get_logs(logs_proc: List[LogProcess]):
    logs_file = 'logs/logs.json'
    
    # Checamos si existe el archivo logs.json
    if os.path.exists(logs_file):
        # Leemos el archivo logs.json
        with open(logs_file, 'r') as file:
            existing_logs = json.load(file)
    else:
        # Sino existe, creamos una lista vacía
        existing_logs = []

    # Agregamos los nuevos logs a la lista existente
    new_logs = [log.dict() for log in logs_proc]
    existing_logs.extend(new_logs)

    # Escribimos la lista de logs en el archivo logs.json
    with open(logs_file, 'w') as file:
        json.dump(existing_logs, file, indent=4)

    return {"received": True}

@app.post("/logsmemory")
def get_logs_memory(logs_memory: List[LogMemory]):
    logs_file = 'logs/logsmemory.json'

    # Checamos si existe el archivo logs.json
    if os.path.exists(logs_file):
        # Leemos el archivo logs.json
        with open(logs_file, 'r') as file:
            existing_logs = json.load(file)
    else:
        # Sino from datetime import datetimeexiste, creamos una lista vacía
        existing_logs = []

    # Agregamos los nuevos logs a la lista existente
    new_logs = [log.dict() for log in logs_memory]
    existing_logs.extend(new_logs)

    # Escribimos la lista de logs en el archivo logs.json
    with open(logs_file, 'w') as file:
        json.dump(existing_logs, file, indent=4)

    return {"received": True}


@app.get("/logs/all")
def get_all_logs():
    logs_file = 'logs/logs.json'
    
    # Checamos si existe el archivo logs.json
    if os.path.exists(logs_file):
        # Leemos el archivo logs.json
        with open(logs_file, 'r') as file:
            existing_logs = json.load(file)
        return existing_logs
    else:
        # Si no existe, retornamos un mensaje
        return {"message": "No logs found"}
    
@app.get("/logs/grafica")
def generar_grafica():
    # Cargar los datos del JSON
    with open('logs/logsmemory.json', 'r') as f:
        data = json.load(f)

    timestamps = [datetime.strptime(entry['timestamp'], "%d/%m/%Y %H:%M:%S") for entry in data]
    free_ram = [entry['free_ram'] for entry in data]
    used_ram = [entry['used_ram'] for entry in data]

    # Crear la gráfica
    plt.figure(figsize=(12, 6))
    plt.plot(timestamps, free_ram, label='Free RAM')
    plt.plot(timestamps, used_ram, label='Used RAM')
    plt.xlabel('Timestamp')
    plt.ylabel('RAM (bytes)')
    plt.title('Uso de RAM a lo largo del tiempo')
    plt.legend()

    # Formatear las etiquetas del eje x
    plt.gca().xaxis.set_major_formatter(DateFormatter('%d/%m %H:%M'))
    plt.gca().xaxis.set_major_locator(mdates.HourLocator(interval=1))
    plt.gcf().autofmt_xdate()

    # Guardar la gráfica en una ruta específica
    ruta_guardado1 = '/home/jhonatan/Documentos/1_USAC/8Semestre/1.sopes1/Lab/Proyecto1/src/python_service/images/Memory.png'
    plt.savefig(ruta_guardado1)

    # # Verificar si la imagen se guardó correctamente
    # if os.path.exists(ruta_guardado):
    #     return {"message": f"Imagen guardada en {ruta_guardado}"}
    # else:
    #     return {"message": "Error al guardar la imagen"}


    # Cargar los datos del JSON
    with open('logs/logs.json', 'r') as f:
        data = json.load(f)

    container_ids = [entry['container_id'][:4] for entry in data]
    memory_usage = [entry['memory_usage'] for entry in data]
    cpu_usage = [entry['cpu_usage'] for entry in data]

    # Crear la gráfica
    fig, ax = plt.subplots(figsize=(12, 6))
    bar_width = 0.35
    index = range(len(container_ids))

    bar1 = ax.bar(index, memory_usage, bar_width, label='Memory Usage')
    bar2 = ax.bar([i + bar_width for i in index], cpu_usage, bar_width, label='CPU Usage')

    ax.set_xlabel('Container ID')
    ax.set_ylabel('Usage')
    ax.set_title('Memory and CPU Usage by Container')
    ax.set_xticks([i + bar_width / 2 for i in index])
    ax.set_xticklabels(container_ids, rotation=45, ha='right')
    ax.legend()

    # Guardar la gráfica en una ruta específica
    ruta_guardado = '/home/jhonatan/Documentos/1_USAC/8Semestre/1.sopes1/Lab/Proyecto1/src/python_service/images/Processes.png'
    plt.savefig(ruta_guardado)

    # Verificar si la imagen se guardó correctamente
    if os.path.exists(ruta_guardado) and os.path.exists(ruta_guardado1):
        return {"message": f"Imagenes guardadas"}
    else:
        return {"message": "Error al guardar la imagen"}

