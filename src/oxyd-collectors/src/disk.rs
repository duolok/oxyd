use async_trait::async_trait;
use oxyd_domain::{
    traits::Collector,
    models::{SystemMetrics, DiskMetrics, DiskInfo, DiskIoStats},
    errors::CollectorError,
};
use chrono::Utc;

pub struct DiskCollector {
    mount_points: Vec<String>,
}

impl DiskCollector {
    pub fn new() -> Self {
        Self {
            mount_points: vec!["/".to_string()],
        }
    }

    async fn get_disk_usage(&self, mount_point: &str) -> Result<DiskInfo, CollectorError> {
        // use statvfs syscall to get disk space
        let output = tokio::process::Command::new("df")
            .arg("-B1") // bytes
            .arg(mount_point)
            .output()
            .await
            .map_err(|e| CollectorError::SystemInfoError(e.to_string()))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = output_str.lines().collect();
        
        if lines.len() < 2 {
            return Err(CollectorError::ParseError("df".to_string(), "Invalid output".to_string()));
        }

        let parts: Vec<&str> = lines[1].split_whitespace().collect();
        if parts.len() < 6 {
            return Err(CollectorError::ParseError("df".to_string(), "Invalid format".to_string()));
        }

        let device = parts[0].to_string();
        let total_bytes: u64 = parts[1].parse().unwrap_or(0);
        let used_bytes: u64 = parts[2].parse().unwrap_or(0);
        let free_bytes: u64 = parts[3].parse().unwrap_or(0);
        let usage_percent = if total_bytes > 0 {
            (used_bytes as f32 / total_bytes as f32) * 100.0
        } else {
            0.0
        };

        Ok(DiskInfo {
            device,
            mount_point: mount_point.to_string(),
            filesystem: "unknown".to_string(),
            total_bytes,
            used_bytes,
            free_bytes,
            usage_percent,
        })
    }
}

#[async_trait]
impl Collector for DiskCollector {
    fn id(&self) -> &str {
        "disk"
    }

    async fn collect(&self) -> Result<SystemMetrics, CollectorError> {
        let mut disks = Vec::new();

        for mount_point in &self.mount_points {
            if let Ok(disk_info) = self.get_disk_usage(mount_point).await {
                disks.push(DiskMetrics {
                    info: disk_info,
                    io_stats: DiskIoStats {
                        device: "unknown".to_string(),
                        read_bytes: 0,
                        write_bytes: 0,
                        read_count: 0,
                        write_count: 0,
                        read_time_ms: 0,
                        write_time_ms: 0,
                        busy_time_ms: 0,
                        io_in_progress: 0,
                    },
                });
            }
        }

        Ok(SystemMetrics {
            timestamp: Utc::now(),
            system_info: Default::default(),
            cpu: Default::default(),
            memory: Default::default(),
            disks,
            network: Default::default(),
            processes: Default::default(),
        })
    }

    fn is_available(&self) -> bool {
        std::path::Path::new("/proc/diskstats").exists()
    }
}
