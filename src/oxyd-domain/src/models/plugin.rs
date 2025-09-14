use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub dependencies: Vec<PluginDependency>,
    pub capabilities: Vec<PluginCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub name: String,
    pub version: String,
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginCapability {
    Collector,
    Visualizer,
    Exporter,
    Alert,
    ProcessControl,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginState {
    Loaded,
    Active,
    Inactive,
    Failed,
    Unloaded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMessage {
    pub plugin_id: String,
    pub message_type: MessageType,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Request,
    Response,
    Event,
    Error,
}
