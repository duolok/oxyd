use async_trait::async_trait;
use oxyd_domain::{traits::Collector, models::SystemMetrics, errors::CollectorError};

pub struct NetworkCollector {
    interfaces: Vec<String>,
}

impl NetworkCollector {
    pub fn new() -> Self {
        Self {
            interfaces: Vec::new(),
        }
    }
}

#[async_trait]
impl Collector for NetworkCollector {
    fn id(&self) -> &str {
        "network"
    }

    async fn collect(&self) -> Result<SystemMetrics, CollectorError> {
        // TODO: Implement by reading from /proc/net/dev and /proc/net/tcp
        unimplemented!("Network collector not yet implemented")
    }

    fn is_available(&self) -> bool {
        std::path::Path::new("/proc/net/dev").exists()
    }
}
