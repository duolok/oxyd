use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub hostname: String,
    pub kernel_version: String,
    pub os_version: String,
    pub architecture: String,
    pub boot_time: DateTime<Utc>,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub model: String,
    pub vendor: String,
    pub cores: usize,
    pub threads: usize,
    pub frequency_mhz: f64,
    pub cache_size_kb: u64,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuCore {
    pub id: usize,
    pub usage_percent: f32,
    pub frequency_mhz: f64,
    pub temperature_celsius: Option<f32>,
    pub states: CpuStates,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CpuStates {
    pub user: u64,
    pub nice: u64,
    pub system: u64,
    pub idle: u64,
    pub iowait: u64,
    pub irq: u64,
    pub softirq: u64,
    pub steal: u64,
    pub guest: u64,
    pub guest_nice: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub available_bytes: u64,
    pub cached_bytes: u64,
    pub buffers_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
    pub swap_free_bytes: u64,
    pub usage_percent: f32,
    pub swap_usage_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub device: String,
    pub mount_point: String,
    pub filesystem: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskIoStats {
    pub device: String,
    pub read_bytes: u64,
    pub write_bytes: u64,
    pub read_count: u64,
    pub write_count: u64,
    pub read_time_ms: u64,
    pub write_time_ms: u64,
    pub busy_time_ms: u64,
    pub io_in_progress: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub mac_address: String,
    pub ip_addresses: Vec<IpAddress>,
    pub is_up: bool,
    pub is_loopback: bool,
    pub speed_mbps: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpAddress {
    pub address: String,
    pub netmask: String,
    pub version: IpVersion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpVersion {
    V4,
    V6,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub interface: String,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub errors_sent: u64,
    pub errors_received: u64,
    pub drop_sent: u64,
    pub drop_received: u64,
}
