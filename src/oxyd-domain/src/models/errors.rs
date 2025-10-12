use thiserror::Error;

#[derive(Error, Debug)]
pub enum OxydError {
    #[error("Collector  error: {0}")]
    Collector(#[from] CollectorError),

    #[error("Process manager error: {0}")]
    ProcessManager(#[from] ProcessError),

    #[error("Plugin error: {0}")]
    PluginError(#[from] PluginError),

    #[error("Configuration error: {0}")]
    ConfigError(#[from] ConfigError),

    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

#[derive(Error, Debug)]
pub enum CollectorError {
    #[error("Failed to read system information: {0}")]
    SystemInfoError(String),

    #[error("Failed to access {0}: {1}")]
    AccessError(String, String),

    #[error("Parse error for {0}: {1}")]
    ParseError(String, String),

    #[error("Collector {0} not available on this system")]
    NotAvailable(String),

    #[error("Timeout while collecting {0}")]
    Timeout(String),
}

#[derive(Error, Debug)]
pub enum ProcessError {
    #[error("Process {0} not found")]
    NotFound(u32),

    #[error("Permission denied for process {0}")]
    PermissionDenied(u32),

    #[error("Invalid signal: {0}")]
    InvalidSignal(String),

    #[error("Failed to {0} process {1}: {2}")]
    ActionFailed(String, u32, String),

    #[error("Failed to list processes: {0}")]
    ListFailed(String)
}

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Plugin {0} not found")]
    NotFound(String),
    
    #[error("Failed to load plugin {0}: {1}")]
    LoadError(String, String),
    
    #[error("Plugin {0} initialization failed: {1}")]
    InitError(String, String),
    
    #[error("Plugin {0} crashed: {1}")]
    CrashError(String, String),
    
    #[error("Invalid plugin configuration: {0}")]
    ConfigError(String),
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file not found: {0}")]
    FileNotFound(String),
    
    #[error("Invalid configuration: {0}")]
    Invalid(String),
    
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
}

#[derive(Error, Debug)]
pub enum UiError {
    #[error("Terminal error: {0}")]
    TerminalError(String),
    
    #[error("Render error: {0}")]
    RenderError(String),
    
    #[error("Input error: {0}")]
    InputError(String),
}
