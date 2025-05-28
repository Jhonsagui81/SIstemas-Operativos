# SISTEMAS OPERATIVOS 1
Jhonatan Alexander Aguilar Reyes - 202106003

# PROYECTO 2

# 🏅 Olimpiadas USAC - Sistema de Monitoreo en Tiem Real

## 📌 Descripción
Sistema distribuido en Kubernetes (GCP) para monitorear en tiempo real las competencias de Natación, Boxeo y Atletismo entre las facultades de Ingeniería y Agronomía. Utiliza Grafana para visualización y Kafka para streaming de datos.

---

## 🏗️ Arquitectura
![Arquitectura](./Proyecto2/img/image.png)
### Componentes Principales:
1. **Locust** 🌪️  
   - Generador de tráfico HTTP con Python.
   - Envía 10,000+ solicitudes/sec en formato JSON:
     ```json
     {
       "faculty": "Ingeniería|Agronomía",
       "discipline": 1|2|3  // 1: Natación, 2: Atletismo, 3: Boxeo
     }
     ```

2. **Servidores de Facultades** ⚙️  
   - **Ingeniería**: Contenedor en Go (Goroutines + Channels + gRPC).
   - **Agronomía**: Contenedor en Rust (Threads + gRPC).
   - Autoescalado horizontal (HPA) basado en carga.

3. **Servidores de Disciplinas** 🏊♂️🥊🏃♂️  
   - Contenedores en Go.
   - Algoritmo de probabilidad (lanzamiento de moneda).
   - Publica resultados en Kafka (tópicos: `winners` y `losers`).

4. **Kafka** 📡  
   - Tópicos: `winners` y `losers`.
   - Implementado con Strimzi.

5. **Consumidores + Redis** 💾  
   - Procesan mensajes en paralelo.
   - Almacenan datos estructurados en Redis (hashes).

6. **Grafana + Prometheus** 📊  
   - Dashboards en tiempo real:
     - Conteo por facultad/disciplina.
     - Monitoreo de cluster (CPU, memoria, pods).

---

## 🛠️ Tecnologías Clave
| Componente          | Tecnologías                                                                 |
|---------------------|-----------------------------------------------------------------------------|
| Infraestructura     | GCP, GKE, Kubernetes, Helm                                                 |
| Lenguajes           | Go (servidores), Rust (servidores), Python (Locust)                        |
| Comunicación        | gRPC, HTTP/HTTPS, Kafka                                                    |
| Almacenamiento      | Redis                                                                       |
| Monitoreo           | Prometheus, Grafana                                                        |
| Autoescalado        | Horizontal Pod Autoscaler (HPA)                                            |

---



# PROYECTO 1

# 📊 Proyecto 1 - Monitorización de Contenedores y Recursos


## 🌟 Descripción
Sistema integral para monitorear contenedores Docker y recursos del sistema (CPU/memoria), combinando un módulo en C, un gestor de contenedores en Rust y un servicio de visualización con Python. Ideal para análisis en tiempo real y gestión automatizada de entornos containerizados.

---

## 🧩 Componentes Principales

### 1. **Módulo en C** 🖥️  
Recolecta datos del sistema en formato JSON, incluyendo:  
- Uso de CPU y memoria RAM.  
- Información detallada de procesos.  
- Estadísticas en tiempo real para integración con otros servicios.

### 2. **Gestor de Contenedores en Rust** 🦀  
- **Autoescalado Inteligente**:  
  - Crea 10 contenedores cada minuto (4 imágenes diferentes).  
  - Ejecuta un analizador cada 10 segundos para optimizar recursos:  
    - Mantiene **2 contenedores de alto consumo**.  
    - Conserva **3 contenedores de bajo consumo**.  
    - Preserva el servicio de Python y el propio analizador.  
- **Orquestación**:  
  - Levanta un entorno con `docker-compose` para el servicio de visualización.  
  - Utiliza volúmenes para compartir datos entre componentes.

### 3. **Servicio de Visualización en Python** 🐍  
- **API REST con FastAPI**:  
  - Recibe logs estructurados (contenedores eliminados, métricas de RAM/CPU).  
  - Almacena datos históricos para análisis.  
- **Dashboard Interactivo**:  
  - Gráficas de uso de recursos.  
  - Historial de contenedores eliminados.  

---

## 🔄 Flujo de Trabajo
1. El **módulo en C** genera datos del sistema cada intervalo definido.  
2. El **gestor en Rust** automatiza la creación/eliminación de contenedores, priorizando eficiencia.  
3. Los logs y métricas se envían al **servicio Python**, que los procesa y visualiza.  
4. Los datos se persisten mediante volúmenes Docker para consultas históricas.  

---

## 🛠️ Tecnologías Utilizadas
| Categoría          | Herramientas                                                      |
|---------------------|-------------------------------------------------------------------|
| Lenguajes           | C (módulo del kernel), Rust (gestor), Python (API y gráficos)     |
| Contenedores        | Docker, Docker Compose                                            |
| Automatización      | Cronjobs (tareas programadas), Scripting                          |
| Visualización       | FastAPI, Bibliotecas de gráficos (Matplotlib/Plotly)              |
| Gestión de Recursos | Algoritmos de priorización (alto/bajo consumo)                    |

---

## 📈 Visualización de Datos
El servicio Python ofrece:  
- **Gráficas en Tiempo Real**:  
  - Uso de RAM y CPU durante la ejecución.  
  - Tendencia de contenedores creados vs. eliminados.  
- **Historial Consultable**:  
  - Exportación de logs en JSON.  
  - Filtros por fecha, tipo de contenedor o consumo.  

---

