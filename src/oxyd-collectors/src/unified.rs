use async_trait::async_trait;
use oxyd_domain::{
    traits::{Collector, ProcessManager},
    models::SystemMetrics,
    errors::CollectorError,
};
use std::sync::Arc;
use chrono::Utc;

use crate::{CpuCollector, MemoryCollector, ProcessCollector, NetworkCollector, DiskCollector};

pub struct UnifiedCollector {
    cpu_collector: CpuCollector,
    memory_collector: MemoryCollector,
    process_collector: ProcessCollector,
    network_collector: NetworkCollector,  
    disk_collector: DiskCollector,        
}

impl UnifiedCollector {
    pub fn new(process_manager: Arc<dyn ProcessManager>, per_core_cpu: bool) -> Self {
        Self {
            cpu_collector: CpuCollector::new(per_core_cpu),
            memory_collector: MemoryCollector::new(),
            process_collector: ProcessCollector::new(process_manager),
            network_collector: NetworkCollector::new(),  
            disk_collector: DiskCollector::new(),        
        }
    }

    async fn get_system_info(&self) -> oxyd_domain::models::SystemInfo {
        use tokio::fs;

        let hostname = hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "unknown".to_string());

        let kernel_version = fs::read_to_string("/proc/version")
            .await
            .unwrap_or_else(|_| "unknown".to_string())
            .lines()
            .next()
            .unwrap_or("unknown")
            .to_string();

        let (boot_time, uptime) = match fs::read_to_string("/proc/uptime").await {
            Ok(content) => {
                let uptime_secs = content
                    .split_whitespace()
                    .next()
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0) as u64;
                
                let boot = Utc::now() - chrono::Duration::seconds(uptime_secs as i64);
                (boot, uptime_secs)
            }
            Err(_) => (Utc::now(), 0),
        };

        oxyd_domain::models::SystemInfo {
            hostname,
            kernel_version,
            os_version: std::env::consts::OS.to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            boot_time,
            uptime_seconds: uptime,
        }
    }
}

#[async_trait]
impl Collector for UnifiedCollector {
    fn id(&self) -> &str {
        "unified_system"
    }

    async fn collect(&self) -> Result<SystemMetrics, CollectorError> {
        let (cpu_result, memory_result, process_result, network_result, disk_result) = tokio::join!(
            self.cpu_collector.collect(),
            self.memory_collector.collect(),
            self.process_collector.collect(),
            self.network_collector.collect(),  
            self.disk_collector.collect()     
        );

        let system_info = self.get_system_info().await;

        let cpu_metrics = cpu_result?.cpu;
        let memory_metrics = memory_result?.memory;
        let process_metrics = process_result?.processes;
        let network_metrics = network_result?.network;  
        let disk_metrics = disk_result?.disks;         

        Ok(SystemMetrics {
            timestamp: Utc::now(),
            system_info,
            cpu: cpu_metrics,
            memory: memory_metrics,
            disks: disk_metrics,        
            network: network_metrics,  
            processes: process_metrics,
        })
    }

    fn is_available(&self) -> bool {
        self.cpu_collector.is_available()
            && self.memory_collector.is_available()
            && self.process_collector.is_available()
    }

    fn interval_ms(&self) -> u64 {
        1000
    }
}
