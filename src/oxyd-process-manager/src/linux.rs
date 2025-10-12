use oxyd_domain:: {
    traits::ProcessManager,
    errors::ProcessError,
    models::{Process, ProcessState, ProcessAction, ProcessSignal, ProcessActionResult }
};
use async_trait::async_trait;
use tokio::fs;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use std::path::Path;
use std::sync::Arc;
use std::collections::HashMap;
use chrono::Utc;

pub struct LinuxProcessManager {
    protected_processes: Vec<String>,
    cpu_cache: Arc<Mutex<HashMap<u32, CpuMeasurement>>>,
}

impl LinuxProcessManager {
    pub fn new() -> Self{ 
        Self {
            protected_processes: vec![
                String::from("systemd"),
                String::from("kernel"),
                String::from("init"),
            ],
            cpu_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_config(protected_processes: Vec<String>) -> Self {
        Self {
            protected_processes,
            cpu_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ProcessManager for LinuxProcessManager {
    async fn list_processes(&self) -> Result<Vec<u32>, ProcessError> {
        let mut pids = Vec::new();

        let mut entries = fs::read_dir("/proc").await
            .map_err(|e| ProcessError::ListFailed(format!("Failed to read /proc: {}", e)))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| ProcessError::ListFailed(format!("Failed to read entry: {}", e)))? {

            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            // Check if the directory name is a number (PID)
            if let Ok(pid) = file_name_str.parse::<u32>() {
                // Verify it's actually a process directory
                let stat_path = format!("/proc/{}/stat", pid);
                if Path::new(&stat_path).exists() {
                    pids.push(pid);
                }
            }
        }

        Ok(pids)
    } 

    async fn get_process(&self, pid: u32) -> Result<Process, ProcessError> {
        let process_path = format!("/proc/{}", pid);        

        if !Path::new(&process_path).exists() {
            return Err(ProcessError::NotFound(pid));
        }

        let stat_path = format!("{}/stat", process_path);
        let stat_content = fs::read_to_string(&stat_path).await
            .map_err(|e| ProcessError::ReadFailed(pid, format!("Failed to read stat: {}", e)))?;

        let stat_fields = parse_stat(&stat_content)?;

        let comm_path = format!("{}/comm", process_path);
        let name = fs::read_to_string(&comm_path).await
            .unwrap_or_else(|_| stat_fields.comm.clone())
            .trim()
            .to_string();

        let cmdline_path = format!("{}/cmdline", process_path);
        let (command, arguments) = match fs::read(&cmdline_path).await {
            Ok(data) => {
                let parts: Vec<String> = data.split(|&b| b == 0)
                    .filter(|s| !s.is_empty())
                    .map(|s| String::from_utf8_lossy(s).to_string())
                    .collect();

                if parts.is_empty() {
                    (format!("[{}]", name), vec![])
                } else {
                    let cmd = parts[0].clone();
                    let args = parts.into_iter().skip(1).collect();
                    (cmd, args)
                }

            }
            Err(_) => (format!("[{}]", name), vec![]),
        };

        let exe_path = format!("{}/exe", process_path);
        let executable_path = fs::read_link(&exe_path).await
            .ok()
            .map(|p| p.to_string_lossy().to_string());

        let cwd_path = format!("{}/cwd", process_path);
        let working_dir = fs::read_link(&cwd_path).await
            .ok()
            .map(|p| p.to_string_lossy().to_string());

        let status_path = format!("{}/status", process_path);
        let status_info = fs::read_to_string(&status_path).await
            .map_err(|e| ProcessError::ReadFailed(pid, format!("Failed to read status: {}", e)))?;

        let status_fields = parse_status(&status_info);

        let fd_path = format!("{}/fd", process_path);
        let open_files = match fs::read_dir(&fd_path).await {
            Ok(mut entries) => {
                let mut count = 0;
                while let Ok(Some(_)) = entries.next_entry().await {
                    count += 1
                }
                count
            }
            Err(_) => 0,
        };

        let tcp_path = format!("{}/net/tcp", process_path);
        let tcp6_path = format!("{}/net/tcp6", process_path);
        let udp_path = format!("{}/net/udp", process_path);
        let udp6_path = format!("{}/net/udp6", process_path);

        let open_connections = count_connections(&tcp_path).await
        + count_connections(&tcp6_path).await
        + count_connections(&udp_path).await
        + count_connections(&udp6_path).await;

        let state = match stat_fields.state {
            'R' => ProcessState::Running,
            'S' => ProcessState::Sleeping,
            'D' => ProcessState::Waiting,
            'Z' => ProcessState::Zombie,
            'T' => ProcessState::Stopped,
            't' => ProcessState::Stopped, // tracing stop
            'W' => ProcessState::Sleeping, // paging (obsolete)
            'X' | 'x' => ProcessState::Dead,
            'K' => ProcessState::Waiting, // wakekill
            'P' => ProcessState::Waiting, // parked
            'I' => ProcessState::Idle,
            _ => ProcessState::Unknown,
        };

        let cpu_usage_percent = calculate_cpu_usage_cached(
            pid,
            &stat_fields,
            &self.cpu_cache
        ).await;


        let boot_time = get_boot_time().await?;
        let ticks_per_second = 100;
        let start_time = boot_time + chrono::Duration::seconds((stat_fields.starttime / ticks_per_second) as i64);

        Ok(Process {
            pid,
            ppid: if stat_fields.ppid > 0 { Some(stat_fields.ppid) } else { None },
            name,
            command,
            arguments,
            executable_path,
            working_dir,
            state,
            user: status_fields.uid.clone(),
            group: status_fields.gid.clone(),
            priority: stat_fields.priority,
            nice: stat_fields.nice,
            threads: status_fields.threads,
            start_time,
            cpu_usage_percent,
            memory_usage_bytes: status_fields.rss_bytes,
            memory_usage_percent: calculate_memory_percent(status_fields.rss_bytes).await,
            virtual_memory_bytes: status_fields.vm_size,
            disk_write_bytes: status_fields.write_bytes,
            disk_read_bytes: status_fields.read_bytes,
            open_files,
            open_connections,
        })
    }

    async fn kill_process(&self, pid: u32) -> Result<Process, ProcessError> {
        let process : Process = self.get_process(pid).await?;

        if self.protected_processes.contains(&process.name) {
            return Err(ProcessError::PermissionDenied(pid));
        }

        let output = tokio::process::Command::new("kill")
            .arg("-9")  
            .arg(pid.to_string())
            .output()
            .await
            .map_err(|e| ProcessError::ActionFailed(
                "kill".to_string(), 
                pid, 
                e.to_string()
            ))?; 

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ProcessError::ActionFailed(
                "kill".to_string(),
                pid,
                stderr.to_string()
            ));
        }

        Ok(process)
    }

    async fn send_signal(&self, pid:u32, signal: ProcessSignal) -> Result<ProcessActionResult, ProcessError> {
        let signal_name = match signal {
            ProcessSignal::Kill => "SIGKILL",
            ProcessSignal::Terminate => "SIGTERM",
            ProcessSignal::Stop => "SIGSTOP",
            ProcessSignal::Continue => "SIGCONT",
            ProcessSignal::Interrupt => "SIGINT",
            ProcessSignal::Quit => "SIGQUIT",
            ProcessSignal::Hangup => "SIGHUP",
            ProcessSignal::Custom(_num) => "CUSTOM",
        };

        let process: Process = self.get_process(pid).await?;
        if self.protected_processes.contains(&process.name) {
            return Err(ProcessError::PermissionDenied(pid));
        }

        let output = tokio::process::Command::new("kill")
            .arg(format!("-{}", signal_name))
            .arg(pid.to_string())
            .output()
            .await
            .map_err(|e| ProcessError::ActionFailed(
                "kill".to_string(), 
                pid, 
                e.to_string()
            ))?; 

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ProcessError::ActionFailed(
                "kill".to_string(),
                pid,
                stderr.to_string()
            ));
        }

        Ok(ProcessActionResult {
            pid,
            action: ProcessAction::Terminate,
            success: true,
            message: Some(format!("Sent {} to process {}", signal_name, pid)),
            timestamp: Utc::now(),
        })
    }


    async fn send_priority(&self, pid: u32, priority: i32) -> Result<ProcessActionResult, ProcessError> {
        Ok(ProcessActionResult {
            pid,
            action: ProcessAction::SetPriority(priority),
            success: true,
            message: Some(format!("Set priority of process {} to {}", pid, priority)),
            timestamp: Utc::now(),
        })
    }

    async fn suspend_process(&self, pid: u32) -> Result<ProcessActionResult, ProcessError> {
        self.send_signal(pid, ProcessSignal::Stop).await
    }

    async fn continue_process(&self, pid: u32) -> Result<ProcessActionResult, ProcessError> {
        self.send_signal(pid, ProcessSignal::Continue).await
    }
}

#[derive(Debug, Clone)]
struct CpuMeasurement {
    process_time: u64,
    system_time: u64,
    timestamp: std::time::Instant,
}

#[derive(Debug, Clone)]
struct StatFields {
    comm: String,
    state: char,
    ppid: u32,
    priority: i32,
    nice: i32,
    starttime: u64,
    utime: u64,      // User mode time
    stime: u64,      // Kernel mode time
    cutime: u64,     // Children user time
    cstime: u64,     // Chil
}

// Parse /proc/[pid]/stat format
// Format: pid (comm) state ppid pgrp session tty_nr tpgid flags ...
fn parse_stat(stat_content: &str) -> Result<StatFields, ProcessError> {
    let start = stat_content.find('(')
        .ok_or_else(|| ProcessError::ParseError("Invalid stat format".to_string()))?;
    let end = stat_content.rfind(')')
        .ok_or_else(|| ProcessError::ParseError("Invalid stat format".to_string()))?;

    let comm = stat_content[start + 1..end].to_string();
    let after_comm = &stat_content[end + 2..]; // Skip ') '

    let fields: Vec<&str> = after_comm.split_whitespace().collect();

    if fields.len() < 20 {
        return Err(ProcessError::ParseError("Insufficient stat fields".to_string()));
    }

   Ok(StatFields {
        comm,
        state: fields[0].chars().next().unwrap_or('?'),
        ppid: fields[1].parse().unwrap_or(0),
        priority: fields[15].parse().unwrap_or(20),
        nice: fields[16].parse().unwrap_or(0),
        starttime: fields[19].parse().unwrap_or(0),
        utime: fields[11].parse().unwrap_or(0),   
        stime: fields[12].parse().unwrap_or(0),   
        cutime: fields[13].parse().unwrap_or(0),  
        cstime: fields[14].parse().unwrap_or(0), 
    }) 
}

struct StatusFields {
    uid: String,
    gid: String,
    threads: u32,
    vm_size: u64,
    rss_bytes: u64,
    read_bytes: u64,
    write_bytes: u64,
}

fn parse_status(status_content: &str) -> StatusFields {
    let mut uid = String::from("0");
    let mut gid = String::from("0");
    let mut threads = 1;
    let mut vm_size = 0;
    let mut rss_bytes = 0;
    let read_bytes = 0;
    let write_bytes = 0;

    for line in status_content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }

        match parts[0] {
            "Uid:" => {
                if parts.len() > 1 {
                    uid = parts[1].to_string();
                }
            }
            "Gid:" => {
                if parts.len() > 1 {
                    gid = parts[1].to_string();
                }
            }
            "Threads:" => {
                threads = parts[1].parse().unwrap_or(1);
            }
            "VmSize:" => {
                if parts.len() > 1 {
                    vm_size = parts[1].parse::<u64>().unwrap_or(0) * 1024; // Convert KB to bytes
                }
            }
            "VmRSS:" => {
                if parts.len() > 1 {
                    rss_bytes = parts[1].parse::<u64>().unwrap_or(0) * 1024; // Convert KB to bytes
                }
            }
            _ => {}
        }
    }

    // Try to read IO stats from /proc/[pid]/io
    // This is done separately as it may not be available

    StatusFields {
        uid,
        gid,
        threads,
        vm_size,
        rss_bytes,
        read_bytes,
        write_bytes,
    }
}

async fn count_connections(path: &str) -> u32 {
    match fs::read_to_string(path).await {
        Ok(content) => {
            content.lines().count() as u32
        }
        Err(_) => 0,
    }
}

async fn get_boot_time() -> Result<chrono::DateTime<Utc>, ProcessError> {
    let stat_content = fs::read_to_string("/proc/stat").await
        .map_err(|e| ProcessError::ReadFailed(0, format!("Failed to read /proc/stat: {}", e)))?;

    for line in stat_content.lines() {
        if line.starts_with("btime") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let boot_timestamp = parts[1].parse::<i64>()
                    .map_err(|_| ProcessError::ParseError("Invalid btime".to_string()))?;
                return Ok(chrono::DateTime::from_timestamp(boot_timestamp, 0)
                    .unwrap_or_else(|| Utc::now()));
            }
        }
    }
    Err(ProcessError::ParseError("Could not find btime".to_string()))
}

async fn calculate_memory_percent(rss_bytes: u64) -> f64 {
    match fs::read_to_string("/proc/meminfo").await {
        Ok(content) => {
            for line in content.lines() {
                if line.starts_with("MemTotal:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(total_kb) = parts[1].parse::<u64>() {
                            let total_bytes = total_kb * 1024;
                            return (rss_bytes as f64 / total_bytes as f64) * 100.0;
                        }
                    }
                }
            }
            0.0
        }
        Err(_) => 0.0,
    }
}

async fn calculate_cpu_usage_cached(
    pid: u32,
    stat_fields: &StatFields,
    cache: &Arc<Mutex<HashMap<u32, CpuMeasurement>>>,
) -> f64 {
    let process_time = stat_fields.utime + stat_fields.stime;
    let system_time = match get_system_cpu_time().await {
        Ok(time) => time,
        Err(_) => return 0.0,
    };
    
    let now = std::time::Instant::now();
    let mut cache_lock = cache.lock().await;
    
    if let Some(prev) = cache_lock.get(&pid) {
        let time_delta = now.duration_since(prev.timestamp).as_secs_f64();
        
        // Return cached value if called too quickly (< 100ms)
        // This prevents returning 0.0 on rapid successive calls
        if time_delta < 0.1 {
            // Calculate from cached data
            if prev.system_time > 0 {
                let num_cpus = num_cpus::get() as f64;
                let process_delta = process_time.saturating_sub(prev.process_time) as f64;
                let system_delta = system_time.saturating_sub(prev.system_time) as f64;
                
                if system_delta > 0.0 {
                    let cpu_percent = (process_delta / system_delta) * 100.0 * num_cpus;
                    return cpu_percent.min(100.0 * num_cpus);
                }
            }
            return 0.0;
        }
        
        let process_delta = process_time.saturating_sub(prev.process_time) as f64;
        let system_delta = system_time.saturating_sub(prev.system_time) as f64;
        
        if system_delta > 0.0 {
            let num_cpus = num_cpus::get() as f64;
            let cpu_percent = (process_delta / system_delta) * 100.0 * num_cpus;
            
            // Update cache with new measurement
            cache_lock.insert(pid, CpuMeasurement {
                process_time,
                system_time,
                timestamp: now,
            });
            
            return cpu_percent.min(100.0 * num_cpus);
        }
    }
    
    // First measurement - store and return 0
    cache_lock.insert(pid, CpuMeasurement {
        process_time,
        system_time,
        timestamp: now,
    });
    
    0.0
}

async fn calculate_cpu_usage_instant(pid: u32) -> f64 {
    // Take first measurement
    let measurement1 = match take_cpu_measurement(pid).await {
        Ok(m) => m,
        Err(_) => return 0.0,
    };
    
    // Wait 100ms
    sleep(Duration::from_millis(100)).await;
    
    // Take second measurement
    let measurement2 = match take_cpu_measurement(pid).await {
        Ok(m) => m,
        Err(_) => return 0.0,
    };
    
    // Calculate deltas
    let process_delta = measurement2.process_time.saturating_sub(measurement1.process_time) as f64;
    let system_delta = measurement2.system_time.saturating_sub(measurement1.system_time) as f64;
    let time_delta = measurement2.timestamp.duration_since(measurement1.timestamp).as_secs_f64();
    
    if system_delta > 0.0 && time_delta > 0.0 {
        let num_cpus = num_cpus::get() as f64;
        let cpu_percent = (process_delta / system_delta) * 100.0 * num_cpus;
        return cpu_percent.min(100.0 * num_cpus);
    }
    
    0.0
}

async fn take_cpu_measurement(pid: u32) -> Result<CpuMeasurement, ProcessError> {
    let stat_path = format!("/proc/{}/stat", pid);
    let stat_content = fs::read_to_string(&stat_path).await
        .map_err(|e| ProcessError::ReadFailed(pid, format!("Failed to read stat: {}", e)))?;
    
    let stat_fields = parse_stat(&stat_content)?;
    let process_time = stat_fields.utime + stat_fields.stime;
    let system_time = get_system_cpu_time().await?;
    
    Ok(CpuMeasurement {
        process_time,
        system_time,
        timestamp: std::time::Instant::now(),
    })
}

// Get total system CPU time from /proc/stat
async fn get_system_cpu_time() -> Result<u64, ProcessError> {
    let stat_content = fs::read_to_string("/proc/stat").await
        .map_err(|e| ProcessError::ReadFailed(0, format!("Failed to read /proc/stat: {}", e)))?;
    
    // First line is "cpu" with aggregate times
    if let Some(first_line) = stat_content.lines().next() {
        if first_line.starts_with("cpu ") {
            let parts: Vec<&str> = first_line.split_whitespace().collect();
            
            // cpu user nice system idle iowait irq softirq steal guest guest_nice
            // Sum all times (indices 1-10)
            let total: u64 = parts.iter()
                .skip(1)
                .take(10)
                .filter_map(|s| s.parse::<u64>().ok())
                .sum();
            
            return Ok(total);
        }
    }
    
    Err(ProcessError::ParseError("Could not parse /proc/stat".to_string()))
}

fn get_num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}
