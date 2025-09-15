use async_trait::async_trait;
use std::result::Result;
use crate::models::{SystemMetrics};
use crate::errors::{CollectorError, ProcessError};
use crate::{PluginError, Process, ProcessActionResult, ProcessSignal};

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
    // List all process IDs.
    async fn list_processes(&self) -> Result<Vec<u32>, ProcessError>;

    // Get detailed information about a specific process.
    async fn get_process(&self, pid: u32) -> Result<Process, ProcessError>;

    // Send a signal to kill a process (SIGKILL).
    async fn kill_process(&self, pid: u32) -> Result<Process, ProcessError>;

    // Send a signal to a process.
    async fn send_signal(&self, pid:u32, signal: ProcessSignal) -> Result<ProcessActionResult, ProcessError>;

    // Set process priority.
    async fn send_priority(&self, pid: u32, priority: i32) -> Result<ProcessActionResult, ProcessError>;

    // Suspend a process (SIGSTOP).
    async fn suspend_process(&self, pid: u32) -> Result<ProcessActionResult, ProcessError>;

    // Resume a process (SIGCONT) 
    async fn continue_process(&self, pid: u32) -> Result<ProcessActionResult, ProcessError>;
}

#[async_trait]
pub trait Plugin: Send + Sync {
    // Plugin name.
    fn name(&self) -> &str;

    // Plugin version 
    fn version(&self) -> &str;

    // Initialize the plugin
    async fn initialize(&mut self) -> Result<(), PluginError>;

    // Cleanup after shutting down
    async fn cleanup(&mut self) -> Result<(), PluginError>;
}
