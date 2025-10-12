use async_trait::async_trait;
use oxyd_domain::{traits::Collector, models::SystemMetrics, errors::CollectorError};

pub struct DiskCollector {
    // Track mount points to monitor
    mount_points: Vec<String>,
}

impl DiskCollector {
    pub fn new() -> Self {
        Self {
            mount_points: vec!["/".to_string()],
        }
    }
}

#[async_trait]
impl Collector for DiskCollector {
    fn id(&self) -> &str {
        "disk"
    }

    async fn collect(&self) -> Result<SystemMetrics, CollectorError> {
        // TODO: Implement by reading from /proc/diskstats and /proc/mounts
        unimplemented!("Disk collector not yet implemented")
    }

    fn is_available(&self) -> bool {
        std::path::Path::new("/proc/diskstats").exists()
    }
}
