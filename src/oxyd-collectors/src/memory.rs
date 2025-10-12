use oxyd_domain::{CollectorError, SystemMetrics};
use oxyd_domain::Collector;
use async_trait::async_trait;
use tokio::fs;
use chrono::Utc;

pub struct MemoryCollector;

impl MemoryCollector {
    pub fn new() -> Self {
        Self
    }

    async fn parse_meminfo(&self) -> Result<oxyd_domain::models::MemoryInfo, CollectorError> {
        let content = fs::read_to_string("/proc/meminfo")
            .await
            .map_err(|e| CollectorError::AccessError("/proc/meminfo".to_string(), e.to_string()))?;

        let mut total = 0u64;
        let mut free = 0u64;
        let mut available = 0u64;
        let mut cached = 0u64;
        let mut buffers = 0u64;
        let mut swap_total = 0u64;
        let mut swap_free = 0u64;

        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }

            let value = parts[1].parse::<u64>().unwrap_or(0) * 1024; 

            match parts[0] {
                "MemTotal:" => total = value,
                "MemFree:" => free = value,
                "MemAvailable:" => available = value,
                "Cached:" => cached = value,
                "Buffers:" => buffers = value,
                "SwapTotal:" => swap_total = value,
                "SwapFree:" => swap_free = value,
                _ => {}
            }
        }

        let used = total.saturating_sub(free);
        let swap_used = swap_total.saturating_sub(swap_free);
        let usage_percent = if total > 0 {
            (used as f32 / total as f32) * 100.0
        } else {
            0.0
        };
        let swap_usage_percent = if swap_total > 0 {
            (swap_used as f32 / swap_total as f32) * 100.0
        } else {
            0.0
        };

        Ok(oxyd_domain::models::MemoryInfo {
            total_bytes: total,
            used_bytes: used,
            free_bytes: free,
            available_bytes: available,
            cached_bytes: cached,
            buffers_bytes: buffers,
            swap_total_bytes: swap_total,
            swap_used_bytes: swap_used,
            swap_free_bytes: swap_free,
            usage_percent,
            swap_usage_percent,
        })
    }
}

#[async_trait]
impl Collector for MemoryCollector {
    fn id(&self) -> &str {
        "memory"
    }

    async fn collect(&self) -> Result<SystemMetrics, CollectorError> {
        let memory = self.parse_meminfo().await?;

        Ok(SystemMetrics {
            timestamp: Utc::now(),
            system_info: Default::default(),
            cpu: Default::default(),
            memory,
            disks: vec![],
            network: Default::default(),
            processes: Default::default(),
        })
    }

    fn is_available(&self) -> bool {
        std::path::Path::new("/proc/meminfo").exists()
    }
}
