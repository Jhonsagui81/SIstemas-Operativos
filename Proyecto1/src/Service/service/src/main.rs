use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::process::Command;
use std::env;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use reqwest::blocking::Client;
use serde_json;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local};
use ctrlc;

//Creacion de variable global para almacenar id de contenedor de logs
static LOG_CONTAINER_ID: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

// CREACIÓN DE STRUCT

/* 
    El #[derive (macro...)] es una característica de Rust que permite a los desarrolladores
    agregar funcionalidades a sus estructuras de datos. En este caso, estamos agregando
    la capacidad de serializar y deserializar la estructura de datos a JSON que es parte de la librería
    serde.
*/

#[derive(Debug, Serialize, Deserialize)]
struct SystemInfo {
    #[serde(rename = "TotalRAM")]
    total_ram: u64,
    #[serde(rename = "FreeRAM")]
    free_ram: u64,
    #[serde(rename = "UsedRAM")]
    used_ram: u64,
    #[serde(rename = "Processes")]
    processes: Vec<Process>
}

/* 
    Además de esto, estamos implementando los traits Eq, Ord y PartialOrd para poder comparar
    los procesos en base a su uso de CPU y memoria.

    La estructura de datos representa un proceso en el sistema operativo, con los siguientes campos:
    - pid: El identificador del proceso.
    - name: El nombre del proceso.
    - cmd_line: La línea de comandos que se utilizó para ejecutar el proceso.
    - memory_usage: La cantidad de memoria que está utilizando el proceso.
    - cpu_usage: El porcentaje de uso de CPU que está utilizando el proceso.

    Serde nos deja implementar macros a acada campo de la estructura de datos para poder renombrar
    los campos en el JSON que se genere.
*/
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Process {
    #[serde(rename = "PID")]
    pid: u32,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Cmdline")]
    cmd_line: String,
    #[serde(rename = "Vsz")]
    vsz: f64,
    #[serde(rename = "Rss")]
    rss: f64,
    #[serde(rename = "MemoryUsage")]
    memory_usage: f64,
    #[serde(rename = "CPUUsage")]
    cpu_usage: f64,
}

#[derive(Debug, Serialize,Deserialize, Clone)]
struct LogProcess {
    pid: u32,
    container_id: String,
    name: String,
    vsz: f64,
    rss: f64,
    memory_usage: f64,
    cpu_usage: f64,
    action: String,
    timestamp: String
}

#[derive(Debug, Serialize,Deserialize, Clone)]
struct LogMemory {
    total_ram: u64,
    free_ram: u64,
    used_ram: u64,
    timestamp: String
}
// IMPLEMENTACIÓN DE MÉTODOS

/* 
    Función para sobreescribir el campo cmd_line de cada proceso por el id del contenedor.
*/
impl Process {
    fn get_container_id(&self) -> &str {
        let parts: Vec<&str> = self.cmd_line.split_whitespace().collect();
        for (i, part) in parts.iter().enumerate() {
            if *part == "-id" {
                if let Some(id) = parts.get(i + 1) {
                    return id;
                }
            }
        }
        "N/A"
    }
}

// IMPLEMENTACIÓN DE TRAITS

/* 
    Contamos con 2 ordenamientos, el Ord y el PartialOrd. El primero es para poder comparar
    los procesos en base a su uso de CPU y memoria, mientras que el segundo es para poder
    comparar los procesos en base a su uso de CPU y memoria de manera parcial.

    ¿Por qué de manera parcial si todos los valores existen? 
        - Porque en el caso de que haya un valor NaN, la comparación no se puede hacer de manera total.
        - Por ejemplo, si un proceso tiene un uso de memoria de 10 y otro de NaN, no se puede comparar
          de manera total, pero sí de manera parcial.
        - Al manejar números decimales pueden existir valores NaN, por lo que es importante manejarlos.
*/

/* 
    Este trait no lleva ninguna implementación, pero es necesario para poder comparar ya que debe satisfacer
    la propiedad de reflexividad, es decir, que un proceso es igual a sí mismo.
*/
impl Eq for Process {}  


/* 
    Ord Trait:
    Funcionalidad: Proporciona una comparación total para dos instancias de Process. 
    Devuelve un std::cmp::Ordering que puede ser Less, Greater o Equal.
    Ejecución: Si partial_cmp devuelve Some(Ordering), sort usará el resultado de cmp para ordenar los elementos. 
    La implementación de cmp en Process compara primero el uso de CPU y, si es igual, compara el uso de memoria.
    
    ¿Qué significa esto?
        - Permite comparar procesos basándose en su uso de CPU y memoria.
        - Si el uso de CPU de un proceso es mayor que el de otro, el proceso con mayor uso de CPU es considerado mayor.
        - Si el uso de CPU de ambos procesos es igual, se comparan en base a su uso de memoria.
        - Si tanto el uso de CPU como el de memoria son iguales, los procesos se consideran iguales.

    Detalles de implementación:
        - Se utiliza unwrap_or para devolver std::cmp::Ordering::Equal en caso de que haya un valor NaN.
        - El método then_with se usa para comparar en base a la memoria si el uso de CPU es igual.
        - Los || no son necesarios aquí ya que unwrap_or maneja los valores NaN.

    TODO? :Se pueden agregar más condiciones para comparar en base a otros campos si es necesario.
*/
// impl Ord for Process {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         self.cpu_usage.partial_cmp(&other.cpu_usage).unwrap_or(std::cmp::Ordering::Equal)
//             .then_with(|| self.memory_usage.partial_cmp(&other.memory_usage).unwrap_or(std::cmp::Ordering::Equal))
//     }
// }


impl Ord for Process {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.cpu_usage.partial_cmp(&other.cpu_usage)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| self.memory_usage.partial_cmp(&other.memory_usage)
                             .unwrap_or(std::cmp::Ordering::Equal))
            .then_with(|| self.vsz.partial_cmp(&other.vsz)
                             .unwrap_or(std::cmp::Ordering::Equal))
            .then_with(|| self.rss.partial_cmp(&other.rss)
                             .unwrap_or(std::cmp::Ordering::Equal))
    }
}
/* 
    PartialOrd Trait:

    Funcionalidad: Permite la comparación parcial de dos instancias de Process. Devuelve un Option<std::cmp::Ordering>, 
    que puede ser Some(Ordering) si la comparación es válida o None si no lo es (por ejemplo, si hay un valor NaN).
    Ejecución: La función sort primero intentará usar partial_cmp para comparar los elementos. Si partial_cmp devuelve None, la comparación falla.
    
    ¿Qué significa esto?
        - La comparación puede fallar si hay un valor NaN.
        - Por ejemplo, si un proceso tiene un uso de memoria de 10 y otro tiene NaN, la comparación fallará.

    Detalles de implementación:
        - Se delega la comparación al método cmp del trait Ord, envolviendo el resultado en Some.
*/
impl PartialOrd for Process {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}


// FUNCIONES

/* 
    Función para matar un contenedor de Docker.
    - id: El identificador del contenedor que se quiere matar.
    - Regresa un std::process::Output que contiene la salida del comando que se ejecutó.
*/
fn kill_container(id: &str) -> std::process::Output {
    let  output = std::process::Command::new("sudo")
        .arg("docker")
        .arg("stop")
        .arg(id)
        .output()
        .expect("failed to execute process");

    println!("Matando contenedor con id: {}", id);

    output
}

fn obtener_fecha_hora() -> String {
    let ahora: DateTime<Local> = Local::now();
    ahora.format("%d/%m/%Y %H:%M:%S").to_string()
}

fn analyzer( system_info:  SystemInfo) {
    let id_log = LOG_CONTAINER_ID.lock().unwrap().clone();
    let _id_contenedor_logs = match id_log {
        Some(id) => id,
        None => "No encontrado".to_string(),
    };

    let mut log_memory_list: Vec<LogMemory> = Vec::new();
    let log_memory: LogMemory = LogMemory {
        total_ram: system_info.total_ram,
        free_ram: system_info.free_ram,
        used_ram: system_info.used_ram,
        timestamp: obtener_fecha_hora(),
    }; 
    log_memory_list.push(log_memory);

    println!("Memoria");
    println!("Total RAM: {}", system_info.total_ram);
    println!("Free RAM: {}", system_info.free_ram);
    println!("Used RAM: {}", system_info.used_ram);
    println!("------------------------------\n");

    // Creamos un vector vacío para guardar los logs de los procesos.
    let mut log_proc_list: Vec<LogProcess> = Vec::new();


    /* 
        Creamos un vector vacío para guardar los logs del sistema.
        En este caso, no se guardará nada, pero se puede modificar para guardar
        información del sistema.
    */
    let mut processes_list: Vec<Process> = system_info.processes;


    processes_list.retain(|process| {
        // Truncar el ID largo del contenedor en el JSON para que coincida con el ID corto
        let truncated_id = &process.get_container_id()[.._id_contenedor_logs.len()];
        truncated_id != _id_contenedor_logs
    });
    /* 
        Cuando llamas a la función sort en un vector de Process, se ejecutarán los traits 
        Ord y PartialOrd en el siguiente orden y con la siguiente funcionalidad:


        La función sort del vector llama internamente a partial_cmp para comparar los elementos.
        partial_cmp delega la comparación a cmp del trait Ord.


        Comparación con cmp:

        cmp compara primero el uso de CPU (cpu_usage).
        Si el uso de CPU es igual, compara el uso de memoria (memory_usage).
        Si ambos son iguales, devuelve Ordering::Equal.
        Funcionalidad de los Traits
        PartialOrd: Permite la comparación parcial, necesaria para manejar casos donde los valores pueden ser NaN.
        Ord: Proporciona una comparación total, necesaria para ordenar completamente los elementos del vector.

        Cuando llamas a processes_list.sort(), el método sort usará partial_cmp y cmp para comparar y 
        ordenar los procesos en el vector processes_list basándose en el uso de CPU y memoria.
    */
    processes_list.sort();


    // Dividimos la lista de procesos en dos partes iguales.
    let len = processes_list.len();
    let split_index = (len + 1) / 2; 
    let (lowest_list, highest_list) = processes_list.split_at(split_index);


    // Hacemos un print de los contenedores de bajo consumo en las listas.
    println!("Bajo consumo");
    for process in lowest_list {
        println!("PID: {}, Name: {}, container ID: {}, Vsz: {},Rss: {}, Memory Usage: {}, CPU Usage: {}", process.pid, process.name, process.get_container_id(), process.vsz, process.rss, process.memory_usage, process.cpu_usage);
    }

    println!("------------------------------\n");

    println!("Alto consumo");
    for process in highest_list {
        println!("PID: {}, Name: {}, container ID: {}, Vsz: {},Rss: {}, Memory Usage: {}, CPU Usage: {}", process.pid, process.name, process.get_container_id(), process.vsz, process.rss, process.memory_usage, process.cpu_usage);
    }

    println!("------------------------------\n");

    /* 
        En la lista de bajo consumo, matamos todos los contenedores excepto los 3 primeros.
        antes 
        | 1 | 2 | 3 | 4 | 5 |

        después
        | 1 | 2 | 3 |
    */

    if lowest_list.len() > 3 {
        // Iteramos sobre los procesos en la lista de bajo consumo.
        for process in lowest_list.iter().skip(3) {
            let log_process = LogProcess {
                pid: process.pid,
                container_id: process.get_container_id().to_string(),
                name: process.name.clone(),
                vsz: process.vsz,
                rss: process.rss,
                memory_usage: process.memory_usage,
                cpu_usage: process.cpu_usage,
                action: "stop low".to_string(),
                timestamp: obtener_fecha_hora()
            };
    
            log_proc_list.push(log_process.clone());

            // Matamos el contenedor.
            let _output = kill_container(&process.get_container_id());

        }
    } 

    /* 
        En la lista de alto consumo, matamos todos los contenedores excepto los 2 últimos.
        antes 
        | 1 | 2 | 3 | 4 | 5 |

        después
                    | 4 | 5 |
    */
    if highest_list.len() > 2 {
        // Iteramos sobre los procesos en la lista de alto consumo.
        for process in highest_list.iter().take(highest_list.len() - 2) {
            let log_process = LogProcess {
                pid: process.pid,
                container_id: process.get_container_id().to_string(),
                name: process.name.clone(),
                vsz: process.vsz,
                rss: process.rss,
                memory_usage: process.memory_usage,
                cpu_usage: process.cpu_usage,
                action: "stop high".to_string(),
                timestamp: obtener_fecha_hora()
            };
    
            log_proc_list.push(log_process.clone());

            // Matamos el contenedor.
            let _output = kill_container(&process.get_container_id());

        }
    }

    // TODO: ENVIAR LOGS AL CONTENEDOR REGISTRO


    //LOGS DE PROCESO
    let client = Client::new();
    let res = client
        .post("http://localhost:8000/logs") // Replace with your endpoint URL
        .json(&log_proc_list)
        .send()
        .unwrap();

    // Handle the response
    let response_text = res.text().unwrap();
    // println!("Response from Python: {}", response_text);

    
    //LOGS DE MEMORIA
    let client = Client::new();
    let res = client
        .post("http://localhost:8000/logsmemory") // Replace with your endpoint URL
        .json(&log_memory_list)
        .send()
        .unwrap();

    // Handle the response
    let response_text = res.text().unwrap();

    // Hacemos un print de los contenedores que matamos.
    println!("Contenedores matados");
    for process in log_proc_list {
        println!("PID: {}, Name: {}, Container ID: {}, Memory Usage: {}, CPU Usage: {} ", process.pid, process.name, process.container_id,  process.memory_usage, process.cpu_usage);
    }

    println!("------------------------------\n\n");

    
}

/*  
    Función para leer el archivo proc
    - file_name: El nombre del archivo que se quiere leer.
    - Regresa un Result<String> que puede ser un error o el contenido del archivo.
*/
fn read_proc_file(file_name: &str) -> io::Result<String> {
    // Se crea un Path con el nombre del archivo que se quiere leer.
    let path  = Path::new("/proc").join(file_name);

    /* 
        Se abre el archivo en modo lectura y se guarda en la variable file.
        En caso de que haya un error al abrir el archivo, se regresa un error.
        El signo de interrogación es un atajo para regresar un error en caso de que haya uno.
    */
    let mut file = File::open(path)?;

    // Se crea una variable mutable content que se inicializa con un String vacío.
    let mut content = String::new();

    // Se lee el contenido del archivo y se guarda en la variable content.
    file.read_to_string(&mut content)?;


    // Se regresa el contenido del archivo.
    Ok(content)
}

/* 
    Función para deserializar el contenido del archivo proc a un vector de procesos.
    - json_str: El contenido del archivo proc en formato JSON.
    - Regresa un Result<> que puede ser un error o un SystemInfo.
*/
fn parse_proc_to_struct(json_str: &str) -> Result<SystemInfo, serde_json::Error> {
    // Se deserializa el contenido del archivo proc a un SystemInfo.
    let system_info: SystemInfo = serde_json::from_str(json_str)?;

    // Se regresa el SystemInfo.
    Ok(system_info)
}

fn start_log_container() -> Result<(), Box<dyn std::error::Error>> {
    // Ruta al directorio donde se encuentra el docker-compose.yml
    let python_service_dir = Path::new("../../../python_service");

    // Cambia el directorio de trabajo al de python_service
    env::set_current_dir(&python_service_dir)?;

    // Ejecuta docker-compose up en el directorio python_service
    let output = Command::new("docker-compose")
        .arg("up")
        .arg("-d")  // Modo detach para que se ejecute en segundo plano
        .output()?;

    if !output.status.success() {
        eprintln!("Error al levantar el contenedor de logs: {:?}", output);
        return Err("Error al iniciar el contenedor de logs".into());
    }

    println!("Contenedor de logs levantado");

    // Obtén el ID del contenedor de logs
    let container_id = get_log_container_id()?;

    *LOG_CONTAINER_ID.lock().unwrap() = Some(container_id);
    Ok(())
}

fn get_log_container_id() -> Result<String, Box<dyn std::error::Error>> {
    // Ejecuta docker ps para obtener el ID del contenedor de logs
    let output = Command::new("docker")
        .arg("ps")
        .arg("--filter")
        .arg("name=python_container")  // Nombre del contenedor del gestor de logs
        .arg("--format")
        .arg("{{.ID}}")  // Solo obtenemos el ID del contenedor
        .output()?;
      // Capturamos la salida estándar y de error
      let stdout = std::str::from_utf8(&output.stdout)?;
      let _stderr = std::str::from_utf8(&output.stderr)?;
    if !output.status.success() {
        eprintln!("Error al obtener el ID del contenedor de logs: {:?}", output);
        return Err("Error al obtener el ID del contenedor de logs".into());
    }

    println!("Salida estándar de docker-compose:{}", stdout);
    println!("Contenedor de logs levantado");

    let container_id = String::from_utf8(output.stdout)?;
    Ok(container_id.trim().to_string())  // Devuelve el ID del contenedor sin espacios
}

fn crear_cronjob() {
    let output = Command::new("sh")
        .arg("-c")
        .arg("echo '* * * * * /home/jhonatan/Documentos/1_USAC/8Semestre/1.sopes1/Lab/Proyecto1/src/Script/conteiners.sh'  | crontab -")
        .output()
        .expect("Failed to create cronjob");

    if output.status.success() {
        println!("Cronjob creado exitosamente.");
    } else {
        eprintln!("Error al crear el cronjob: {:?}", output);
    }
}


fn eliminar_cronjob() {
    let output = Command::new("crontab")
        .arg("-r")
        .output()
        .expect("Failed to remove cronjob");

    if output.status.success() {
        println!("Cronjob eliminado exitosamente.");
    } else {
        eprintln!("Error al eliminar el cronjob: {:?}", output);
    }
}

fn main() {

    //Iniciar el script para crear contenedores
    crear_cronjob();
    // TODO: antes de iniciar el loop, ejecutar el docker-compose.yml y obtener el id del contenedor registro.
    // Ejecutar el comando docker-compose up
     // Ruta relativa al archivo docker-compose.yml desde tu archivo main.rs
    let _conteiner_id = start_log_container();
    

    // TODO: Utilizar algo para capturar la señal de terminación y matar el contenedor registro y cronjob.
    // Registrar un manejador para la señal Ctrl+C
    ctrlc::set_handler(move || {
        println!("Ctrl-C pressed. Exiting...");
        eliminar_cronjob();
        let client = Client::new();
        let res = client
            .get("http://localhost:8000/logs/grafica") // Replace with your endpoint URL
            .send()
            .unwrap();

    // Handle the response
    let response_text = res.text().unwrap();

        std::process::exit(0);
    }).expect("Error setting Ctrl-C handler");

    loop {
        // Creamos una estructura de datos SystemInfo con un vector de procesos vacío.
        let system_info: Result<SystemInfo, _>;

        // Leemos el contenido del archivo proc y lo guardamos en la variable json_str.
        let json_str = read_proc_file("sysinfo_202106003").unwrap();

        // Deserializamos el contenido del archivo proc a un SystemInfo.
        system_info = parse_proc_to_struct(&json_str);

        // Dependiendo de si se pudo deserializar el contenido del archivo proc o no, se ejecuta una u otra rama.
        match system_info {
            Ok( info) => {
                analyzer(info);
            }
            Err(e) => println!("Failed to parse JSON: {}", e),
        }

        // Dormimos el hilo principal por 10 segundos.
        std::thread::sleep(std::time::Duration::from_secs(10));
    }

}