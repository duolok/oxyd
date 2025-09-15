use async_trait::async_trait;
use crate::models::{SystemMetrics};
use crate::errors::CollectorError;

#[async_trait]
pub trait Collector: Send + Sync {
    // Unique identifier for the collector
    fn id(&self) -> &str;
    
    // Collect metrics from the system
    async fn collect(&self) -> Result<SystemMetrics, CollectorError>;
    
    // Check if collector is available on this system
    fn is_available(&self) -> bool;
    
    // Get collection interval in milliseconds
    fn interval_ms(&self) -> u64 {
        1000
    }
}

#[async_trait]
pub trait ProcessManager: Send + Sync {
    async fn list_processes(&self) -> Result<Vec<u32>, crate::errors::ProcessError>;
}

#[async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
}
