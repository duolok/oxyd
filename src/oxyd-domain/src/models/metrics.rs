use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::{
    SystemInfo, CpuCore, MemoryInfo, DiskInfo, DiskIoStats,
    NetworkInterface, NetworkStats
};

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemMetrics {
    pub timestamp: DateTime<Utc>,
    pub system_info: SystemInfo,
    pub cpu: CpuMetrics,
    pub memory: MemoryInfo,
    pub disks: Vec<DiskMetrics>,
    pub network: NetworkMetrics,
    pub processes: ProcessMetrics,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CpuMetrics {
    pub overall_usage_percent: f32,
    pub cores: Vec<CpuCore>,
    pub load_average: LoadAverage,
    pub context_switches: u64,
    pub interrupts: u64,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LoadAverage {
    pub one_minute: f32,
    pub five_minutes: f32,
    pub fifteen_minutes: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskMetrics {
    pub info: DiskInfo,
    pub io_stats: DiskIoStats,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub interfaces: Vec<NetworkInterface>,
    pub stats: Vec<NetworkStats>,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub active_connections: Vec<NetworkConnection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnection {
    pub protocol: Protocol,
    pub local_address: String,
    pub local_port: u16,
    pub remote_address: Option<String>,
    pub remote_port: Option<u16>,
    pub state: ConnectionState,
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Protocol {
    Tcp,
    Udp,
    Unix,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionState {
    Established,
    Listen,
    TimeWait,
    CloseWait,
    SynSent,
    SynReceived,
    Closing,
    Closed,
    Unknown,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ProcessMetrics {
    pub total_count: usize,
    pub running_count: usize,
    pub sleeping_count: usize,
    pub stopped_count: usize,
    pub zombie_count: usize,
}
