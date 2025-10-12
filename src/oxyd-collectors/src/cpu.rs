use oxyd_domain::{CollectorError, CpuCore, CpuMetrics, CpuStates, metrics::LoadAverage, SystemMetrics};
use oxyd_domain::Collector;
use async_trait::async_trait;
use tokio::fs;
use chrono::Utc;

pub struct CpuCollector {
    per_core: bool,
    previous_stats: std::sync::Arc<tokio::sync::Mutex<Option<Vec<CpuStates>>>>,
}

impl CpuCollector {
    pub fn new(per_core: bool) -> Self {
        Self {
            per_core,
            previous_stats: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
        }
    }

    async fn read_cpu_stats(&self) -> Result<Vec<CpuStates>, CollectorError> {
        let content = fs::read_to_string("/proc/stat")
            .await
            .map_err(|e| CollectorError::AccessError("/proc/stat".to_string(), e.to_string()))?;

        let mut stats: Vec<CpuStates> = Vec::new();

        for line in content.lines() {
            if line.starts_with("cpu") && line.chars().nth(3).map_or(false, |c| c.is_whitespace() || c.is_numeric()) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 0 {
                    stats.push(CpuStates {
                        user: parts[1].parse().unwrap_or(0),
                        nice: parts[2].parse().unwrap_or(0),
                        system: parts[3].parse().unwrap_or(0),
                        idle: parts[4].parse().unwrap_or(0),
                        iowait: parts[5].parse().unwrap_or(0),
                        irq: parts[6].parse().unwrap_or(0),
                        softirq: parts[7].parse().unwrap_or(0),
                        steal: parts.get(8).and_then(|s| s.parse().ok()).unwrap_or(0),
                        guest: parts.get(9).and_then(|s| s.parse().ok()).unwrap_or(0),
                        guest_nice: parts.get(10).and_then(|s| s.parse().ok()).unwrap_or(0),
                    })
                }
            }
        }

        Ok(stats)
    }

    async fn calculate_usage(&self, current: &CpuStates, previous: &CpuStates) -> f32 {
        let prev_total = previous.user + previous.nice + previous.system + previous.idle
            + previous.iowait + previous.irq + previous.softirq + previous.steal;
        let curr_total = current.user + current.nice + current.system + current.idle
            + current.iowait + current.irq + current.softirq + current.steal;

        let prev_idle = previous.idle + previous.iowait;
        let curr_idle = current.idle + current.iowait;

        let total_diff = curr_total.saturating_sub(prev_total);
        let idle_diff = curr_idle.saturating_sub(prev_idle);

        if total_diff == 0 {
            return 0.0;
        }

        let usage_diff = total_diff.saturating_sub(idle_diff);
        (usage_diff as f32 / total_diff as f32) * 100.0
    }

    async fn read_load_average(&self) -> Result<LoadAverage, CollectorError> {
        let content = fs::read_to_string("/proc/loadavg")
            .await
            .map_err(|e| CollectorError::AccessError("/proc/loadavg".to_string(), e.to_string()))?;

        let parts: Vec<&str> = content.split_whitespace().collect();
        
        Ok(LoadAverage {
            one_minute: parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0.0),
            five_minutes: parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0.0),
            fifteen_minutes: parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0.0),
        })
    }
}

#[async_trait]
impl Collector for CpuCollector {
    fn id(&self) -> &str {
        "cpu"
    }

    async fn collect(&self) -> Result<SystemMetrics, CollectorError> {
        let current_stats = self.read_cpu_stats().await?;
        let load_avg = self.read_load_average().await?;

        let mut previous_lock = self.previous_stats.lock().await;

        let (overall_usage, cores) = if let Some(prev_stats) = previous_lock.as_ref() {
            let overall = if !current_stats.is_empty() && !prev_stats.is_empty(){ 
                self.calculate_usage(&current_stats[0], &prev_stats[0]).await
            } else {
                0.0
            };

            let mut core_metrics = Vec::new();
            if self.per_core {
                for (i, (curr, prev)) in current_stats.iter().skip(1).zip(prev_stats.iter().skip(1)).enumerate() {
                    let usage = self.calculate_usage(curr, prev).await;
                    core_metrics.push(CpuCore {
                        id: i,
                        usage_percent: usage,
                        frequency_mhz: 0.0,
                        temperature_celsius: None,
                        states: curr.clone()
                    });
                }
            } 
            (overall, core_metrics)
        } else {
            (0.0, Vec::new())   
        };


        *previous_lock = Some(current_stats);

        Ok(SystemMetrics {
            timestamp: Utc::now(),
            system_info: Default::default(),
            cpu: CpuMetrics {
                overall_usage_percent: overall_usage,
                cores,
                load_average: load_avg,
                context_switches: 0,
                interrupts: 0,
            },
            memory: Default::default(),
            disks: vec![],
            network: Default::default(),
            processes: Default::default(),
        })
    }

    fn is_available(&self) -> bool {
        std::path::Path::new("/proc/stat").exists()
    }
}
