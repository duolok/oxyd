use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use oxyd_domain::{
    models::{Config, SystemMetrics},
    traits::{Collector, ProcessManager, Plugin},
};
use oxyd_process_manager::LinuxProcessManager;

pub struct Engine {
    collectors: Arc<RwLock<Vec<Box<dyn Collector>>>>,
    process_manager: Arc<dyn ProcessManager>,
    plugins: Arc<RwLock<Vec<Box<dyn Plugin>>>>,
    metrics_tx: broadcast::Sender<SystemMetrics>,
    config: Config,
}

impl Engine {
    pub fn new(config: Config) -> Self {
        let (metrics_tx, _) = broadcast::channel(100);
        
        let process_manager = LinuxProcessManager::with_config(
            config.process_manager.protected_processes.clone()
        );
        
        Self {
            collectors: Arc::new(RwLock::new(Vec::new())),
            process_manager: Arc::new(process_manager),
            plugins: Arc::new(RwLock::new(Vec::new())),
            metrics_tx,
            config,
        }
    }
    
    pub fn new_default() -> Self {
        let config = Config {
            general: oxyd_domain::models::GeneralConfig {
                update_interval_ms: 1000,
                history_size: 100,
                log_level: oxyd_domain::models::LogLevel::Info,
                data_dir: String::from("/tmp/oxyd"),
            },
            collectors: oxyd_domain::models::CollectorConfig {
                enabled_collectors: vec![String::from("cpu"), String::from("memory")],
                cpu: oxyd_domain::models::CpuCollectorConfig {
                    enabled: true,
                    per_core: true,
                    collect_temperature: false,
                },
                memory: oxyd_domain::models::MemoryCollectorConfig {
                    enabled: true,
                },
                disk: oxyd_domain::models::DiskCollectorConfig {
                    enabled: true,
                },
                network: oxyd_domain::models::NetworkCollectorConfig {
                    enabled: true,
                },
                process: oxyd_domain::models::ProcessCollectorConfig {
                    enabled: true,
                    command_line_max_length: 256,
                },
            },
            ui: oxyd_domain::models::UIConfig {
                theme: oxyd_domain::models::Theme::Dark,
                refresh_rate_ms: 500,
                show_help_on_start: true,
                default_tab: oxyd_domain::models::TabType::Overview,
                chart_height: 10,
                process_table_size: 20,
            },
            process_manager: oxyd_domain::models::ProcessManagerConfig {
                allow_kill: true,
                allow_priority_change: true,
                require_sudo: true,
                protected_processes: vec![
                    String::from("systemd"),
                    String::from("init"),
                    String::from("kernel"),
                ],
            },
            plugins: oxyd_domain::models::PluginConfig {
                enabled: false,
                plugin_dir: String::from("/usr/local/lib/oxyd/plugins"),
                auto_load: false,
                plugins: std::collections::HashMap::new(),
            },
            alerts: oxyd_domain::models::AlertConfig {
                enabled: false,
                rules: vec![],
                channels: vec![],
            },
        };
        
        Self::new(config)
    }
    
    pub async fn run(&self) {
        println!("Engine running...");
        println!("Process Manager initialized");
        
        match self.process_manager.list_processes().await {
            Ok(pids) => println!("Found {} processes", pids.len()),
            Err(e) => eprintln!("Error listing processes: {}", e),
        }
        
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            println!("Tick...");
        }
    }
    
    // Expose process manager for external use
    pub fn process_manager(&self) -> &Arc<dyn ProcessManager> {
        &self.process_manager
    }
    
    // Subscribe to metrics updates
    pub fn subscribe_metrics(&self) -> broadcast::Receiver<SystemMetrics> {
        self.metrics_tx.subscribe()
    }
}
