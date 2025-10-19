use async_trait::async_trait;
use chrono::Utc;
use oxyd_domain::{
    errors::CollectorError,
    models::{NetworkStats, SystemMetrics},
    traits::Collector,
};
use tokio::fs;

pub struct NetworkCollector {}

impl NetworkCollector {
    pub fn new() -> Self {
        Self {}
    }

    async fn parse_net_dev(&self) -> Result<Vec<NetworkStats>, CollectorError> {
        let content = fs::read_to_string("/proc/net/dev")
            .await
            .map_err(|e| CollectorError::AccessError("/proc/net/dev".to_string(), e.to_string()))?;

        let mut stats = Vec::new();

        for line in content.lines().skip(2) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let interface = parts[0].trim_end_matches(':').to_string();

            if parts.len() >= 17 {
                stats.push(NetworkStats {
                    interface,
                    bytes_received: parts[1].parse().unwrap_or(0),
                    packets_received: parts[2].parse().unwrap_or(0),
                    errors_received: parts[3].parse().unwrap_or(0),
                    drop_received: parts[4].parse().unwrap_or(0),
                    bytes_sent: parts[9].parse().unwrap_or(0),
                    packets_sent: parts[10].parse().unwrap_or(0),
                    errors_sent: parts[11].parse().unwrap_or(0),
                    drop_sent: parts[12].parse().unwrap_or(0),
                });
            }
        }

        Ok(stats)
    }
}

#[async_trait]
impl Collector for NetworkCollector {
    fn id(&self) -> &str {
        "network"
    }

    async fn collect(&self) -> Result<SystemMetrics, CollectorError> {
        let stats = self.parse_net_dev().await?;

        let total_bytes_sent = stats.iter().map(|s| s.bytes_sent).sum();
        let total_bytes_received = stats.iter().map(|s| s.bytes_received).sum();

        Ok(SystemMetrics {
            timestamp: Utc::now(),
            system_info: Default::default(),
            cpu: Default::default(),
            memory: Default::default(),
            disks: vec![],
            network: oxyd_domain::models::NetworkMetrics {
                interfaces: vec![],
                stats,
                total_bytes_sent,
                total_bytes_received,
                active_connections: vec![],
            },
            processes: Default::default(),
        })
    }

    fn is_available(&self) -> bool {
        std::path::Path::new("/proc/net/dev").exists()
    }
}
