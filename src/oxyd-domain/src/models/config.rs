use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub collectors: CollectorConfig,
    pub ui: UIConfig,
    pub process_manager: ProcessManagerConfig,
    pub plugins: PluginConfig,
    pub alerts: AlertConfig,
}

#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct GeneralConfig {
    pub update_interval_ms: u64,
    pub history_size: usize,
    pub log_level: LogLevel,
    pub data_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Trace, 
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorConfig {
    pub enabled_collectors: Vec<String>,
    pub cpu: CpuCollectorConfig,
    pub memory: MemoryCollectorConfig,
    pub disk: DiskCollectorConfig,
    pub network: NetworkCollectorConfig,
    pub process: ProcessCollectorConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuCollectorConfig {
    pub enabled: bool,
    pub per_core: bool,
    pub collect_temperature: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCollelctorConfig {
    pub enabled: bool,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskCollectorConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkCollectorConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessCollectorConfig {
    pub enabled: bool,
    pub command_line_max_length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    pub theme: Theme,
    pub refresh_rate_ms: u64,
    pub show_help_on_start: bool,
    pub default_tab: TabType,
    pub chart_height: u16,
    pub process_table_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Theme {
    Default,
    Dark,
    Light,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TabType {
    Overview,
    Cpu,
    Memory,
    Disk,
    Network,
    Processes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessManagerConfig {
    pub allow_kill: bool,
    pub allow_priority_change: bool,
    pub require_sudo: bool,
    pub protected_processes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled: bool,
    pub plugin_dir: String,
    pub auto_load: bool,
    pub plugins: HashMap<String, PluginSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSettings {
    pub enabled: bool,
    pub config: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    pub enabled: bool,
    pub rules: Vec<AlertRule>,
    pub channels: Vec<AlertChannel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
    pub channels: Vec<String>,
    pub cooldown_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    CpuUsageAbove(f32),
    MemoryUsageAbove(f32),
    DiskUsageAbove(f32),
    ProcessCount(usize),
    ProcessNotRunning(String),
    NetworkTrafficAbove(u64),
    TemperatureAbove(f32),
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertChannel {
    Log,
    File(String),
    Command(String),
    Webhook(String),
    Email(EmailConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub smtp_server: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub from: String,
    pub to: Vec<String>,
}
