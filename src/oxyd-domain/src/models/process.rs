use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Process {
    pub pid: u32,
    pub ppid: Option<u32>,
    pub name: String,
    pub command: String,
    pub arguments: Vec<String>,
    pub executable_path: Option<String>,
    pub working_dir: Option<String>,
    pub state: ProcessState,
    pub user: String,
    pub group: String,
    pub priority: i32,
    pub nice: i32,
    pub threads: u32,
    pub start_time: DateTime<Utc>,
    pub cpu_usage_percent: f32,
    pub memory_usage_bytes: u64,
    pub memory_usage_percent: f64,
    pub virtual_memory_bytes: u64,
    pub disk_write_bytes: u64,
    pub disk_read_bytes: u64,
    pub open_files: u32,
    pub open_connections: u32,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessState {
    Running,
    Sleeping,
    Waiting,
    Zombie,
    Stopped,
    Idle,
    Dead,
    Unknown,
}

pub enum ProcessSignal {
    Kill,       // SIGKILL (9)
    Terminate,  // SIGTERM (15)
    Stop,       // SIGTOP (19)
    Continue,   // SIGCONT (18)
    Interrupt,  // SIGINT (2)
    Quit,       // SIGQUIT (3)
    Hangup,     // SIGHUP (1)
    Custom(i32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessAction {
    Kill,
    Terminate,
    Suspend,
    Resume,
    SetPriority(i32),
    SetNice(i32),
    SetAffinity(Vec<usize>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessActionResult {
    pub pid: u32,
    pub action: ProcessAction,
    pub success: bool,
    pub message: Option<String>,
    pub timestamp: DateTime<Utc>,
}
