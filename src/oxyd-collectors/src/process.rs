use async_trait::async_trait;
use oxyd_domain::{
    traits::{Collector, ProcessManager},
    models::{SystemMetrics, SystemInfo, ProcessMetrics, ProcessState},
    errors::CollectorError,
};
use std::sync::Arc;
use chrono::Utc;

pub struct ProcessCollector {
    process_manager: Arc<dyn ProcessManager>,
    max_processes: Option<usize>,
}

impl ProcessCollector {
    pub fn new(process_manager: Arc<dyn ProcessManager>) -> Self {
        Self {
            process_manager,
            max_processes: None,
        }
    }

    pub fn with_limit(process_manager: Arc<dyn ProcessManager>, limit: usize) -> Self {
        Self {
            process_manager,
            max_processes: Some(limit),
        }
    }

    async fn collect_process_metrics(&self) -> Result<ProcessMetrics, CollectorError> {
        let pids = self.process_manager
            .list_processes()
            .await
            .map_err(|e| CollectorError::SystemInfoError(e.to_string()))?;

        let mut running_count = 0;
        let mut sleeping_count = 0;
        let mut stopped_count = 0;
        let mut zombie_count = 0;

        // Collect detailed info for processes (can be limited)
        let process_limit = self.max_processes.unwrap_or(pids.len());
        
        for pid in pids.iter().take(process_limit) {
            if let Ok(process) = self.process_manager.get_process(*pid).await {
                match process.state {
                    ProcessState::Running => running_count += 1,
                    ProcessState::Sleeping | ProcessState::Waiting | ProcessState::Idle => {
                        sleeping_count += 1
                    }
                    ProcessState::Stopped => stopped_count += 1,
                    ProcessState::Zombie => zombie_count += 1,
                    _ => {}
                }
            }
        }

        Ok(ProcessMetrics {
            total_count: pids.len(),
            running_count,
            sleeping_count,
            stopped_count,
            zombie_count,
        })
    }
}

#[async_trait]
impl Collector for ProcessCollector {
    fn id(&self) -> &str {
        "process"
    }

    async fn collect(&self) -> Result<SystemMetrics, CollectorError> {
        let process_metrics = self.collect_process_metrics().await?;

        Ok(SystemMetrics {
            timestamp: Utc::now(),
            system_info: SystemInfo {
                hostname: hostname::get()
                    .ok()
                    .and_then(|h| h.into_string().ok())
                    .unwrap_or_else(|| "unknown".to_string()),
                kernel_version: String::from("unknown"),
                os_version: String::from("unknown"),
                architecture: std::env::consts::ARCH.to_string(),
                boot_time: Utc::now(),
                uptime_seconds: 0,
            },
            cpu: oxyd_domain::models::CpuMetrics {
                overall_usage_percent: 0.0,
                cores: vec![],
                load_average: oxyd_domain::models::metrics::LoadAverage {
                    one_minute: 0.0,
                    five_minutes: 0.0,
                    fifteen_minutes: 0.0,
                },
                context_switches: 0,
                interrupts: 0,
            },
            memory: oxyd_domain::models::MemoryInfo {
                total_bytes: 0,
                used_bytes: 0,
                free_bytes: 0,
                available_bytes: 0,
                cached_bytes: 0,
                buffers_bytes: 0,
                swap_total_bytes: 0,
                swap_used_bytes: 0,
                swap_free_bytes: 0,
                usage_percent: 0.0,
                swap_usage_percent: 0.0,
            },
            disks: vec![],
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
        std::path::Path::new("/proc").exists()
    }

    fn interval_ms(&self) -> u64 {
        1000
    }
}
