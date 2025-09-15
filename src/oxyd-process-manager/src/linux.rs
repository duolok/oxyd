use oxyd_domain:: {
    traits::ProcessManager,
    errors::ProcessError,
    models::{Process, ProcessState, ProcessAction, ProcessSignal, ProcessActionResult }
};
use async_trait::async_trait;
use chrono::Utc;

pub struct LinuxProcessManager {
    protected_processes: Vec<String>,
}

impl LinuxProcessManager {
    pub fn new() -> Self{ 
        Self {
            protected_processes: vec![
                String::from("systemd"),
                String::from("kernel"),
                String::from("init"),
            ]
        }
    }

    pub fn with_config(protected_processes: Vec<String>) -> Self {
        Self {
            protected_processes
        }
    }
}

#[async_trait]
impl ProcessManager for LinuxProcessManager {
    async fn list_processes(&self) -> Result<Vec<u32>, ProcessError> {
        Ok(vec![1, 2, 3, 5, 6, 7, 8])
    }


    async fn get_process(&self, pid: u32) -> Result<Process, ProcessError> {
     Ok(Process {
            pid,
            ppid: Some(1),
            name: format!("process_{}", pid),
            command: format!("/usr/bin/process_{}", pid),
            arguments: vec![],
            executable_path: Some(format!("/usr/bin/process_{}", pid)),
            working_dir: Some(String::from("/home/user")),
            state: ProcessState::Running,
            user: String::from("user"),
            group: String::from("users"),
            priority: 20,
            nice: 0,
            threads: 1,
            start_time: Utc::now(),
            cpu_usage_percent: 0.0,
            memory_usage_bytes: 1024 * 1024, 
            memory_usage_percent: 0.1,
            virtual_memory_bytes: 2 * 1024 * 1024, 
            disk_write_bytes: 0,
            disk_read_bytes: 0,
            open_files: 5,
            open_connections: 2,
        })}

    async fn kill_process(&self, _pid: u32) -> Result<Process, ProcessError> {
        panic!()
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
