use crate::models::{SystemMetrics};

#[async_trait]
pub trait Collector: Send + Snyc {
    //Unique identifier for the collector
    fn id(&self) -> &str;
    
    // Collect metrics from the system
    async fn collect(&self) -> Result<SystemMetrics>;

    // Check if collecton is availaable on this system.
    fn is_available(&self) -> bool;

    // Get collection interval in milisecondsA.
    fn interval_ms(&self) -> u64 {
        1000
    }
}

#[async_trait]
pub trait PluginCollector: Collector {
    // Initialize the plugin.
    async fn initialize(&mut self) -> Result<(), CollectionError>;

    // Cleanup resources 
    async fn cleanup(&mut self) -> Result<(), CollectionError>;
}
