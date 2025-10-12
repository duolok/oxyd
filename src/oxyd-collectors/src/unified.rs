use async_trait::async_trait;
use oxyd_domain::{
    traits::{Collector, ProcessManager},
    models::SystemMetrics,
    errors::CollectorError,
};
use std::sync::Arc;
use chrono::Utc;

use crate::{CpuCollector, MemoryCollector, ProcessCollector};

pub struct UnifiedCollector {
    cpu_collector: CpuCollector,
    memory_collector: MemoryCollector,
    process_collector: ProcessCollector,
}

impl UnifiedCollector {
    pub fn new(process_manager: Arc<dyn ProcessManager>, per_core_cpu: bool) -> Self {
        Self {
            cpu_collector: CpuCollector::new(per_core_cpu),
            memory_collector: MemoryCollector::new(),
            process_collector: ProcessCollector::new(process_manager),
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

        // Get uptime
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
            let (cpu_result, memory_result, process_result) = tokio::join!(
            self.cpu_collector.collect(),
            self.memory_collector.collect(),
            self.process_collector.collect()
        );
        

        // Get system info
        let system_info = self.get_system_info().await;

        // Combine results
        let cpu_metrics = cpu_result?.cpu;
        let memory_metrics = memory_result?.memory;
        let process_metrics = process_result?.processes;

        Ok(SystemMetrics {
            timestamp: Utc::now(),
            system_info,
            cpu: cpu_metrics,
            memory: memory_metrics,
            disks: vec![], // TODO: Implement disk collector
            network: oxyd_domain::models::NetworkMetrics {
                interfaces: vec![],
                stats: vec![],
                total_bytes_sent: 0,
                total_bytes_received: 0,
                active_connections: vec![],
            },
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
