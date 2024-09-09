use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct Process {
    PID: u32,
    Name: String,
    Cmdline: String,
    MemoryUsage: f32,
    CPUUsage: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct SysInfo {
    Processes: Vec<Process>,
}

async fn read_sysinfo() -> Result<SysInfo, Box<dyn std::error::Error>> {
    let path = "/proc/sysinfo";
    let content = fs::read_to_string(path)?;
    let sysinfo: SysInfo = serde_json::from_str(&content)?;
    Ok(sysinfo)
}

fn identify_consumption(processes: &[Process]) {
    let mut high_consumption: Vec<&Process> = Vec::new();
    let mut low_consumption: Vec<&Process> = Vec::new();

    for process in processes {
        if process.MemoryUsage > 0.1 || process.CPUUsage > 0.1 {
            high_consumption.push(process);
        } else {
            low_consumption.push(process);
        }
    }

    println!("High Consumption:");
    for process in high_consumption {
        println!(
            "PID: {}, Memory Usage: {:.2}%, CPU Usage: {:.2}%", 
            process.PID, process.MemoryUsage * 100.0, process.CPUUsage * 100.0
        );
    }

    println!("\nLow Consumption:");
    for process in low_consumption {
        println!(
            "PID: {}, Memory Usage: {:.2}%, CPU Usage: {:.2}%", 
            process.PID, process.MemoryUsage * 100.0, process.CPUUsage * 100.0
        );
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sysinfo = read_sysinfo().await?;
    identify_consumption(&sysinfo.Processes);
    Ok(())
}
