use oxyd_domain::{
    traits::ProcessManager,
    errors::ProcessError,
    models::{Process, ProcessState, ProcessAction, ProcessSignal, ProcessActionResult}
};
use async_trait::async_trait;
use tokio::fs;
use tokio::sync::Mutex;
use std::path::Path;
use std::sync::Arc;
use std::collections::HashMap;
use chrono::Utc;

use super::cpu::{CpuMeasurement, calculate_cpu_usage_cached};
use super::parsers::{parse_stat, parse_status};
use super::helpers::{count_connections, get_boot_time, calculate_memory_percent};

pub struct LinuxProcessManager {
    protected_processes: Vec<String>,
    cpu_cache: Arc<Mutex<HashMap<u32, CpuMeasurement>>>,
}

impl LinuxProcessManager {
    pub fn new() -> Self {
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

            if let Ok(pid) = file_name_str.parse::<u32>() {
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

        // Read and parse stat
        let stat_path = format!("{}/stat", process_path);
        let stat_content = fs::read_to_string(&stat_path).await
            .map_err(|e| ProcessError::ReadFailed(pid, format!("Failed to read stat: {}", e)))?;
        let stat_fields = parse_stat(&stat_content)?;

        // Get process name
        let comm_path = format!("{}/comm", process_path);
        let name = fs::read_to_string(&comm_path).await
            .unwrap_or_else(|_| stat_fields.comm.clone())
            .trim()
            .to_string();

        // Get command and arguments
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

        // Get executable and working directory
        let exe_path = format!("{}/exe", process_path);
        let executable_path = fs::read_link(&exe_path).await
            .ok()
            .map(|p| p.to_string_lossy().to_string());

        let cwd_path = format!("{}/cwd", process_path);
        let working_dir = fs::read_link(&cwd_path).await
            .ok()
            .map(|p| p.to_string_lossy().to_string());

        // Parse status
        let status_path = format!("{}/status", process_path);
        let status_info = fs::read_to_string(&status_path).await
            .map_err(|e| ProcessError::ReadFailed(pid, format!("Failed to read status: {}", e)))?;
        let status_fields = parse_status(&status_info);

        // Count file descriptors
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

        // Count network connections
        let tcp_path = format!("{}/net/tcp", process_path);
        let tcp6_path = format!("{}/net/tcp6", process_path);
        let udp_path = format!("{}/net/udp", process_path);
        let udp6_path = format!("{}/net/udp6", process_path);

        let open_connections = count_connections(&tcp_path).await
            + count_connections(&tcp6_path).await
            + count_connections(&udp_path).await
            + count_connections(&udp6_path).await;

        // Map state
        let state = match stat_fields.state {
            'R' => ProcessState::Running,
            'S' => ProcessState::Sleeping,
            'D' => ProcessState::Waiting,
            'Z' => ProcessState::Zombie,
            'T' => ProcessState::Stopped,
            't' => ProcessState::Stopped,
            'W' => ProcessState::Sleeping,
            'X' | 'x' => ProcessState::Dead,
            'K' => ProcessState::Waiting,
            'P' => ProcessState::Waiting,
            'I' => ProcessState::Idle,
            _ => ProcessState::Unknown,
        };

        // Calculate CPU usage
        let cpu_usage_percent = calculate_cpu_usage_cached(
            pid,
            &stat_fields,
            &self.cpu_cache
        ).await;

        // Calculate start time
        let boot_time = get_boot_time().await?;
        let ticks_per_second = 100;
        let start_time = boot_time + chrono::Duration::seconds(
            (stat_fields.starttime / ticks_per_second) as i64
        );

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
        let process = self.get_process(pid).await?;

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

    async fn send_signal(&self, pid: u32, signal: ProcessSignal) -> Result<ProcessActionResult, ProcessError> {
        let signal_name = match signal {
            ProcessSignal::Kill => "SIGKILL",
            ProcessSignal::Terminate => "SIGTERM",
            ProcessSignal::Stop => "SIGSTOP",
            ProcessSignal::Continue => "SIGCONT",
            ProcessSignal::Interrupt => "SIGINT",
            ProcessSignal::Quit => "SIGQUIT",
            ProcessSignal::Hangup => "SIGHUP",
            ProcessSignal::Custom(_) => "CUSTOM",
        };

        let process = self.get_process(pid).await?;
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
